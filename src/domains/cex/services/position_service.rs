use std::sync::Arc;
use crate::shared::database::{Database, TradeRepository, UserBalanceRepository};
use crate::domains::cex::models::position::{AssetPosition, TradeSummary};
use crate::domains::cex::engine::{Engine, runtime::HighPerformanceEngine};
use anyhow::{Context, Result};
use rust_decimal::Decimal;

/// 거래소 포지션 서비스
/// Exchange Position Service
/// 
/// 역할:
/// - 사용자의 특정 자산 포지션 정보 계산 (평균 매수가, 손익, 수익률 등)
/// - 거래 내역 기반 통계 계산
/// - 현재 시장 가격 기반 평가액 계산
/// 
/// 포지션 계산 로직:
/// 1. 사용자의 매수 거래 내역을 조회하여 평균 매수가 계산 (DB)
/// 2. 현재 보유 수량 조회 (엔진 메모리 - 실시간 정확성 보장)
/// 3. 최근 체결가를 현재 시장 가격으로 사용 (DB 또는 엔진 메모리)
/// 4. 미실현 손익 = (현재 시장 가격 - 평균 매수가) × 보유 수량
/// 5. 수익률 = (미실현 손익 / 총 매수 금액) × 100
#[derive(Clone)]
pub struct PositionService {
    db: Database,
    engine: Arc<tokio::sync::Mutex<HighPerformanceEngine>>,
}

impl PositionService {
    /// 생성자
    /// Constructor
    /// 
    /// # Arguments
    /// * `db` - 데이터베이스 연결
    /// * `engine` - 체결 엔진 (메모리 잔고 조회용)
    /// 
    /// # Returns
    /// PositionService 인스턴스
    pub fn new(db: Database, engine: Arc<tokio::sync::Mutex<HighPerformanceEngine>>) -> Self {
        Self { db, engine }
    }

    /// 사용자의 특정 자산 포지션 정보 조회
    /// Get position information for user's specific asset
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `mint` - 자산 식별자 (예: "SOL", "USDT")
    /// 
    /// # Returns
    /// * `Ok(Some(AssetPosition))` - 포지션 정보 (자산을 보유한 경우)
    /// * `Ok(None)` - 포지션 정보 없음 (자산을 보유하지 않거나 매수 거래가 없는 경우)
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # 계산 로직
    /// 1. 사용자의 현재 잔고 조회 (available + locked)
    /// 2. 사용자의 매수 거래 통계 조회 (평균 매수가, 총 매수 수량, 총 매수 금액)
    /// 3. 사용자의 매도 거래 통계 조회 (매도 횟수, 실현 손익)
    /// 4. 최근 체결가 조회 (현재 시장 가격)
    /// 5. 미실현 손익 및 수익률 계산
    pub async fn get_position(
        &self,
        user_id: u64,
        mint: &str,
    ) -> Result<Option<AssetPosition>> {
        // 1. 현재 잔고 조회 (엔진 메모리에서 실시간 조회)
        let (available, locked) = {
            let engine_guard = self.engine.lock().await;
            engine_guard
                .get_balance(user_id, mint)
                .await
                .context("Failed to get balance from engine")?
        };

        let current_balance = available + locked;

        // 잔고가 없으면 포지션 정보도 없음
        if current_balance == Decimal::ZERO {
            return Ok(None);
        }

        // 2. 매수 거래 통계 조회 (평균 매수가 계산용)
        let trade_repo = TradeRepository::new(self.db.pool().clone());
        let buy_stats = trade_repo
            .get_buy_statistics(user_id, mint)
            .await
            .context("Failed to fetch buy statistics")?;

        // 매수 거래가 없으면 포지션 정보 없음 (보유 수량이 있어도 매수 내역이 없으면 계산 불가)
        let (total_bought_amount, total_bought_cost, average_entry_price) = match buy_stats {
            Some((amount, cost, avg_price)) => (amount, cost, Some(avg_price)),
            None => {
                // 매수 거래가 없지만 잔고가 있는 경우 (예: 입금만 하고 거래 안 함)
                // 기본값으로 반환
                return Ok(Some(AssetPosition {
                    mint: mint.to_string(),
                    current_balance,
                    available,
                    locked,
                    average_entry_price: None,
                    total_bought_amount: Decimal::ZERO,
                    total_bought_cost: Decimal::ZERO,
                    current_market_price: None,
                    current_value: None,
                    unrealized_pnl: None,
                    unrealized_pnl_percent: None,
                    trade_summary: TradeSummary {
                        total_buy_trades: 0,
                        total_sell_trades: 0,
                        realized_pnl: Decimal::ZERO,
                    },
                }));
            }
        };

        // 3. 매도 거래 통계 조회
        let (total_sell_trades, _total_sold_amount, total_sold_value) = trade_repo
            .get_sell_statistics(user_id, mint)
            .await
            .context("Failed to fetch sell statistics")?;

        // 4. 매수 거래 횟수 조회
        let total_buy_trades = trade_repo
            .get_buy_trade_count(user_id, mint)
            .await
            .context("Failed to fetch buy trade count")?;

        // 5. 최근 체결가 조회 (현재 시장 가격)
        let current_market_price = trade_repo
            .get_latest_price(mint)
            .await
            .context("Failed to fetch latest price")?;

        // 6. 손익 계산
        let (current_value, unrealized_pnl, unrealized_pnl_percent) = if let Some(market_price) = current_market_price {
            // 현재 평가액 = 현재 시장 가격 × 현재 보유 수량
            let value = market_price * current_balance;
            
            // 미실현 손익 = 현재 평가액 - (평균 매수가 × 현재 보유 수량)
            let avg_price = average_entry_price.unwrap();
            let pnl = value - (avg_price * current_balance);
            
            // 수익률 = (현재 가격 - 평균 매수가) / 평균 매수가 × 100
            // 이렇게 하면 초기 입금과 관계없이 정확한 수익률 계산 가능
            // 예: 평균 매수가 $100, 현재가 $110 → 수익률 = (110-100)/100 × 100 = 10%
            let pnl_percent = if !avg_price.is_zero() {
                Some(((market_price - avg_price) / avg_price) * Decimal::from(100))
            } else {
                None
            };

            (Some(value), Some(pnl), pnl_percent)
        } else {
            // 시장 가격 정보가 없으면 계산 불가
            (None, None, None)
        };

        // 7. 실현 손익 계산
        // 실현 손익 = 매도 금액 - (평균 매수가 × 매도 수량)
        // 단순화: 매도 금액에서 평균 매수가 기준 매도 수량 금액을 뺌
        // 주의: 정확한 실현 손익 계산을 위해서는 각 매도 거래의 매수가를 추적해야 하지만,
        // 현재는 평균 매수가 기준으로 계산
        let realized_pnl = if !total_sold_value.is_zero() && average_entry_price.is_some() {
            // 매도 수량에 대한 평균 매수가 기준 원가를 계산
            // 하지만 정확한 매도 수량을 알 수 없으므로, 매도 금액에서 추정 원가를 뺌
            // 단순화: 매도 금액 - (평균 매수가 × 매도 수량)
            // 하지만 매도 수량을 정확히 알 수 없으므로, 매도 금액만 반환 (임시)
            // TODO: 정확한 실현 손익 계산을 위해 매도 거래별 원가 추적 필요
            Decimal::ZERO // 임시로 0 반환
        } else {
            Decimal::ZERO
        };

        Ok(Some(AssetPosition {
            mint: mint.to_string(),
            current_balance,
            available,
            locked,
            average_entry_price,
            total_bought_amount,
            total_bought_cost,
            current_market_price,
            current_value,
            unrealized_pnl,
            unrealized_pnl_percent,
            trade_summary: TradeSummary {
                total_buy_trades,
                total_sell_trades,
                realized_pnl,
            },
        }))
    }

    /// 사용자의 모든 자산 포지션 정보 조회
    /// Get all positions for user
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// 
    /// # Returns
    /// * `Ok(Vec<AssetPosition>)` - 모든 자산 포지션 목록
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # Note
    /// - 잔고가 있는 자산만 반환
    /// - 매수 거래가 없는 자산도 포함 (평균 매수가 등이 None)
    pub async fn get_all_positions(&self, user_id: u64) -> Result<Vec<AssetPosition>> {
        // 1. 사용자의 모든 잔고 조회 (DB에서 자산 목록 가져오고, 각각 엔진 메모리에서 잔고 확인)
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());
        let db_balances = balance_repo
            .get_all_by_user(user_id)
            .await
            .context("Failed to fetch user balance list from database")?;

        // 2. 각 자산별로 포지션 정보 계산
        let mut positions = Vec::new();
        let engine_guard = self.engine.lock().await;

        for db_balance in db_balances {
            // USDT는 포지션 계산 대상이 아님 (기준 통화)
            if db_balance.mint_address == "USDT" {
                continue;
            }

            // 엔진 메모리에서 실시간 잔고 확인
            let (available, locked) = engine_guard
                .get_balance(user_id, &db_balance.mint_address)
                .await
                .unwrap_or((Decimal::ZERO, Decimal::ZERO)); // 실패 시 0으로 처리

            // 잔고가 있으면 포지션 정보 계산
            if available + locked > Decimal::ZERO {
                if let Some(position) = self.get_position(user_id, &db_balance.mint_address).await? {
                    positions.push(position);
                }
            }
        }

        Ok(positions)
    }
}

