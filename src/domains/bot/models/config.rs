use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// 봇 설정
/// Bot Configuration
/// 
/// 바이낸스 오더북을 동기화할 때 사용하는 설정값들
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// 봇 1 (매수 전용) 이메일
    /// Bot 1 (Buy only) email
    pub bot1_email: String,
    
    /// 봇 2 (매도 전용) 이메일
    /// Bot 2 (Sell only) email
    pub bot1_password: String,
    
    /// 봇 1 비밀번호
    /// Bot 1 password
    pub bot2_email: String,
    
    /// 봇 2 비밀번호
    /// Bot 2 password
    pub bot2_password: String,
    
    /// 주문 수량 (고정)
    /// Fixed order quantity
    /// 
    /// 바이낸스 오더북의 각 호가에 대해 이 수량으로 주문을 생성합니다.
    /// 예: 1.0 SOL, 10.0 SOL 등
    pub order_quantity: Decimal,
    
    /// 오더북 깊이 (상위 N개)
    /// Orderbook depth (top N entries)
    /// 
    /// 바이낸스에서 받아올 오더북의 상위 N개 호가
    /// 예: 50개면 상위 50개 매수/매도 호가만 동기화
    pub orderbook_depth: usize,
    
    /// 바이낸스 WebSocket URL
    /// Binance WebSocket URL
    pub binance_ws_url: String,
    
    /// 바이낸스 심볼
    /// Binance symbol (e.g., "SOLUSDT")
    pub binance_symbol: String,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            bot1_email: "bot1@bot.com".to_string(),
            bot1_password: "123123".to_string(),
            bot2_email: "bot2@bot.com".to_string(),
            bot2_password: "123123".to_string(),
            order_quantity: Decimal::new(1, 0), // 1.0 SOL
            orderbook_depth: 200, // 상위 200개 호가 처리
            binance_ws_url: "wss://stream.binance.com:9443/ws/solusdt@depth20@100ms".to_string(),
            binance_symbol: "SOLUSDT".to_string(),
        }
    }
}

impl BotConfig {
    /// 환경변수에서 설정 로드
    /// Load configuration from environment variables
    /// 
    /// 환경변수가 없으면 기본값 사용
    pub fn from_env() -> Self {
        let order_quantity = std::env::var("BOT_ORDER_QUANTITY")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| Decimal::from_f64_retain(f).unwrap_or(Decimal::new(1, 0)))
            .unwrap_or(Decimal::new(1, 0));
        
        let orderbook_depth = std::env::var("BOT_ORDERBOOK_DEPTH")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(200); // 기본값: 200개
        
        Self {
            bot1_email: std::env::var("BOT1_EMAIL")
                .unwrap_or_else(|_| "bot1@bot.com".to_string()),
            bot1_password: std::env::var("BOT1_PASSWORD")
                .unwrap_or_else(|_| "123123".to_string()),
            bot2_email: std::env::var("BOT2_EMAIL")
                .unwrap_or_else(|_| "bot2@bot.com".to_string()),
            bot2_password: std::env::var("BOT2_PASSWORD")
                .unwrap_or_else(|_| "123123".to_string()),
            order_quantity,
            orderbook_depth,
            binance_ws_url: std::env::var("BINANCE_WS_URL")
                .unwrap_or_else(|_| "wss://stream.binance.com:9443/ws/solusdt@depth20@100ms".to_string()),
            binance_symbol: std::env::var("BINANCE_SYMBOL")
                .unwrap_or_else(|_| "SOLUSDT".to_string()),
        }
    }
}

