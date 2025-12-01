use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use url::Url;
use chrono;

/// 바이낸스 오더북 업데이트 메시지
/// Binance orderbook update message
/// 
/// 바이낸스 WebSocket에서 받는 depth stream 형식
/// 
/// 참고: 바이낸스 depth stream은 항상 `e` 필드를 포함하지만,
/// 초기 스냅샷이나 다른 형식의 메시지가 올 수 있으므로
/// `e` 필드를 확인하여 "depthUpdate" 이벤트만 처리합니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceOrderbookUpdate {
    /// 이벤트 타입 (항상 "depthUpdate")
    /// 초기 스냅샷 등에서는 없을 수 있음
    #[serde(rename = "e")]
    pub event_type: Option<String>,
    
    /// 이벤트 시간 (밀리초)
    #[serde(rename = "E")]
    pub event_time: Option<u64>,
    
    /// 심볼 (예: "SOLUSDT")
    #[serde(rename = "s")]
    pub symbol: Option<String>,
    
    /// 첫 번째 업데이트 ID
    #[serde(rename = "U")]
    pub first_update_id: Option<u64>,
    
    /// 마지막 업데이트 ID
    /// 초기 스냅샷에서는 `lastUpdateId`로 올 수 있음
    #[serde(rename = "u", alias = "lastUpdateId")]
    pub last_update_id: Option<u64>,
    
    /// 매수 호가 (가격, 수량 쌍)
    /// Bids: [[price, quantity], ...]
    /// 주의: Binance는 Vec<Vec<String>> 형식을 사용합니다 (고정 배열이 아님)
    /// 실제 JSON에서는 "bids"로 나오지만, depthUpdate 이벤트에서는 "b"로 나옵니다.
    /// 두 가지 모두 지원하기 위해 alias 사용
    #[serde(rename = "bids", alias = "b")]
    pub bids: Option<Vec<Vec<String>>>,
    
    /// 매도 호가 (가격, 수량 쌍)
    /// Asks: [[price, quantity], ...]
    /// 주의: Binance는 Vec<Vec<String>> 형식을 사용합니다 (고정 배열이 아님)
    /// 실제 JSON에서는 "asks"로 나오지만, depthUpdate 이벤트에서는 "a"로 나옵니다.
    /// 두 가지 모두 지원하기 위해 alias 사용
    #[serde(rename = "asks", alias = "a")]
    pub asks: Option<Vec<Vec<String>>>,
}

/// 바이낸스 오더북 엔트리
/// Binance orderbook entry
/// 
/// 파싱된 오더북 항목 (가격, 수량)
#[derive(Debug, Clone)]
pub struct BinanceOrderbookEntry {
    /// 가격
    pub price: Decimal,
    
    /// 수량
    pub quantity: Decimal,
}

/// 바이낸스 WebSocket 클라이언트
/// Binance WebSocket Client
/// 
/// 역할:
/// - 바이낸스 depth stream WebSocket 연결
/// - 오더북 업데이트 수신 및 파싱
/// - 업데이트를 채널로 전송
/// 
/// 처리 흐름:
/// 1. 바이낸스 WebSocket 연결
/// 2. 오더북 업데이트 수신
/// 3. JSON 파싱
/// 4. 채널로 전송
pub struct BinanceClient {
    /// WebSocket URL
    ws_url: String,
    
    /// 오더북 업데이트 수신 채널 (송신자)
    /// Orderbook update receiver channel (sender)
    update_tx: Option<mpsc::UnboundedSender<BinanceOrderbookUpdate>>,
}

impl BinanceClient {
    /// 생성자
    /// Constructor
    /// 
    /// # Arguments
    /// * `ws_url` - 바이낸스 WebSocket URL
    /// 
    /// # Returns
    /// BinanceClient 인스턴스
    pub fn new(ws_url: String) -> Self {
        Self {
            ws_url,
            update_tx: None,
        }
    }

    /// 바이낸스 WebSocket 연결 시작
    /// Start Binance WebSocket connection
    /// 
    /// 백그라운드 태스크에서 WebSocket 연결을 유지하고,
    /// 오더북 업데이트를 채널로 전송합니다.
    /// 
    /// # Arguments
    /// * `update_tx` - 오더북 업데이트 전송 채널
    /// 
    /// # Returns
    /// * `Ok(())` - 연결 시작 성공
    /// * `Err` - 연결 실패
    /// 
    /// # 처리 과정
    /// 1. WebSocket URL 파싱
    /// 2. WebSocket 연결
    /// 3. 메시지 수신 루프
    /// 4. JSON 파싱 및 채널 전송
    pub async fn start(
        &mut self,
        update_tx: mpsc::UnboundedSender<BinanceOrderbookUpdate>,
    ) -> Result<()> {
        self.update_tx = Some(update_tx);
        
        let ws_url = self.ws_url.clone();
        let update_tx = self.update_tx.as_ref().unwrap().clone();
        
        // 백그라운드 태스크에서 WebSocket 연결 유지
        tokio::spawn(async move {
            if let Err(e) = Self::run_websocket(ws_url, update_tx).await {
                eprintln!("[Binance Client] WebSocket error: {}", e);
            }
        });
        
        Ok(())
    }

    /// WebSocket 실행 루프
    /// WebSocket run loop
    /// 
    /// 실제 WebSocket 연결 및 메시지 수신을 처리합니다.
    /// 
    /// # Arguments
    /// * `ws_url` - WebSocket URL
    /// * `update_tx` - 오더북 업데이트 전송 채널
    /// 
    /// # 처리 과정
    /// 1. WebSocket 연결
    /// 2. 메시지 수신 루프
    /// 3. JSON 파싱
    /// 4. 채널로 전송
    /// 5. 연결 끊어지면 재연결 시도
    async fn run_websocket(
        ws_url: String,
        update_tx: mpsc::UnboundedSender<BinanceOrderbookUpdate>,
    ) -> Result<()> {
        loop {
            // WebSocket 연결
            let url = Url::parse(&ws_url)
                .context("Failed to parse WebSocket URL")?;
            
            let (ws_stream, _) = connect_async(url)
                .await
                .context("Failed to connect to Binance WebSocket")?;
            
            // 연결 성공 (로그 제거 - 봇 동작은 조용히)
            
            let (mut _write, mut read) = ws_stream.split();
            
            // 메시지 수신 루프
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        // JSON 파싱
                        match serde_json::from_str::<BinanceOrderbookUpdate>(&text) {
                            Ok(update) => {
                                // "depthUpdate" 이벤트만 처리 (초기 스냅샷 등은 무시)
                                let bids_count = update.bids.as_ref().map(|b| b.len()).unwrap_or(0);
                                let asks_count = update.asks.as_ref().map(|a| a.len()).unwrap_or(0);
                                
                                if let Some(event_type) = &update.event_type {
                                    if event_type == "depthUpdate" {
                                        // bids나 asks가 있어야 처리
                                        if bids_count > 0 || asks_count > 0 {
                                            // 채널로 전송
                                            if update_tx.send(update).is_err() {
                                                // 수신자가 없으면 종료
                                                eprintln!("[Binance Client] Receiver dropped, stopping");
                                                return Ok(());
                                            }
                                        }
                                        // bids/asks가 비어있으면 무시 (정상 동작)
                                    }
                                } else {
                                    // event_type이 없으면 초기 스냅샷이거나 다른 형식
                                    // bids/asks가 있으면 처리 (초기 스냅샷)
                                    if bids_count > 0 || asks_count > 0 {
                                        // 초기 스냅샷도 처리 (depthUpdate로 변환)
                                        let mut snapshot_update = update.clone();
                                        snapshot_update.event_type = Some("depthUpdate".to_string());
                                        snapshot_update.event_time = Some(chrono::Utc::now().timestamp_millis() as u64);
                                        snapshot_update.symbol = Some("SOLUSDT".to_string());
                                        
                                        if update_tx.send(snapshot_update).is_err() {
                                            eprintln!("[Binance Client] Receiver dropped, stopping");
                                            return Ok(());
                                        }
                                    }
                                    // bids/asks가 비어있으면 무시 (정상 동작)
                                }
                            }
                            Err(_) => {
                                // 파싱 실패는 무시 (다른 형식의 메시지일 수 있음)
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        eprintln!("[Binance Client] WebSocket closed by server");
                        break; // 재연결 시도
                    }
                    Ok(Message::Ping(_data)) => {
                        // Ping에 대한 Pong 응답 (필요시)
                    }
                    Err(e) => {
                        eprintln!("[Binance Client] WebSocket error: {}", e);
                        break; // 재연결 시도
                    }
                    _ => {
                        // 다른 메시지 타입은 무시
                    }
                }
            }
            
            // 연결이 끊어졌으면 잠시 대기 후 재연결 시도
            eprintln!("[Binance Client] Reconnecting in 5 seconds...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    /// 바이낸스 오더북 업데이트를 파싱된 엔트리로 변환
    /// Parse Binance orderbook update to entries
    /// 
    /// # Arguments
    /// * `update` - 바이낸스 오더북 업데이트
    /// 
    /// # Returns
    /// * `(bids, asks)` - 파싱된 매수/매도 호가
    pub fn parse_orderbook_update(
        update: &BinanceOrderbookUpdate,
    ) -> Result<(Vec<BinanceOrderbookEntry>, Vec<BinanceOrderbookEntry>)> {
        // 매수 호가 파싱
        let bids = update
            .bids
            .as_ref()
            .map(|bids_vec| {
                bids_vec
                    .iter()
                    .filter_map(|entry| {
                        // entry는 Vec<String>이므로 길이가 2인지 확인
                        if entry.len() >= 2 {
                            let price = Decimal::from_str(&entry[0]).ok()?;
                            let quantity = Decimal::from_str(&entry[1]).ok()?;
                            Some(BinanceOrderbookEntry { price, quantity })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        // 매도 호가 파싱
        let asks = update
            .asks
            .as_ref()
            .map(|asks_vec| {
                asks_vec
                    .iter()
                    .filter_map(|entry| {
                        // entry는 Vec<String>이므로 길이가 2인지 확인
                        if entry.len() >= 2 {
                            let price = Decimal::from_str(&entry[0]).ok()?;
                            let quantity = Decimal::from_str(&entry[1]).ok()?;
                            Some(BinanceOrderbookEntry { price, quantity })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        Ok((bids, asks))
    }
}

