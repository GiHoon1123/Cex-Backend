use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use rust_decimal::Decimal;

// =====================================================
// Position 모델
// =====================================================
// 역할: 사용자의 특정 자산에 대한 포지션 정보 (평균 매수가, 손익, 수익률 등)
// 설명: 거래소에서 사용자가 보유한 자산의 투자 성과를 나타냄
// 
// 포지션 정보:
// - average_entry_price: 평균 매수가 (모든 매수 체결의 가중 평균)
// - current_market_price: 현재 시장 가격
// - unrealized_pnl: 미실현 손익 (현재 평가액 - 총 매수 금액)
// - unrealized_pnl_percent: 미실현 수익률 (%)
// 
// 예시:
// - SOL을 평균 100 USDT에 10개 매수
// - 현재 SOL 가격이 110 USDT
//   → average_entry_price: 100.0
//   → current_market_price: 110.0
//   → unrealized_pnl: 100.0 USDT (10개 × 10 USDT)
//   → unrealized_pnl_percent: 10.0%
// =====================================================

/// 사용자 자산 포지션 정보
/// User asset position information
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(as = AssetPosition)]
pub struct AssetPosition {
    /// Asset identifier (e.g., 'SOL', 'USDT')
    /// 자산 식별자
    #[schema(example = "SOL")]
    pub mint: String,

    /// Current balance (available + locked)
    /// 현재 보유 수량
    #[schema(value_type = String, example = "11.0")]
    pub current_balance: Decimal,

    /// Available balance
    /// 사용 가능 잔고
    #[schema(value_type = String, example = "10.0")]
    pub available: Decimal,

    /// Locked balance
    /// 잠긴 잔고
    #[schema(value_type = String, example = "1.0")]
    pub locked: Decimal,

    /// Average entry price (weighted average of all buy trades)
    /// 평균 매수가 (모든 매수 체결의 가중 평균)
    /// Unit: USDT per asset
    /// 단위: 자산 1개당 USDT
    #[schema(value_type = String, example = "100.5")]
    pub average_entry_price: Option<Decimal>,

    /// Total amount bought (sum of all buy trade amounts)
    /// 총 매수 수량 (모든 매수 체결 수량의 합)
    #[schema(value_type = String, example = "15.0")]
    pub total_bought_amount: Decimal,

    /// Total cost of all buy trades (sum of price × amount)
    /// 총 매수 금액 (모든 매수 체결 금액의 합)
    /// Unit: USDT
    #[schema(value_type = String, example = "1507.5")]
    pub total_bought_cost: Decimal,

    /// Current market price (latest trade price or orderbook mid price)
    /// 현재 시장 가격 (최근 체결가 또는 오더북 중간가)
    /// Unit: USDT per asset
    /// 단위: 자산 1개당 USDT
    #[schema(value_type = String, example = "110.0")]
    pub current_market_price: Option<Decimal>,

    /// Current value (current_market_price × current_balance)
    /// 현재 평가액 (현재 시장 가격 × 현재 보유 수량)
    /// Unit: USDT
    #[schema(value_type = String, example = "1210.0")]
    pub current_value: Option<Decimal>,

    /// Unrealized profit/loss (current_value - total_bought_cost)
    /// 미실현 손익 (현재 평가액 - 총 매수 금액)
    /// Positive: profit, Negative: loss
    /// 양수: 이익, 음수: 손실
    /// Unit: USDT
    #[schema(value_type = String, example = "702.5")]
    pub unrealized_pnl: Option<Decimal>,

    /// Unrealized profit/loss percentage
    /// 미실현 수익률 (%)
    /// Formula: (unrealized_pnl / total_bought_cost) × 100
    /// 공식: (미실현 손익 / 총 매수 금액) × 100
    #[schema(value_type = String, example = "46.6")]
    pub unrealized_pnl_percent: Option<Decimal>,

    /// Trade summary
    /// 거래 요약
    pub trade_summary: TradeSummary,
}

/// 거래 요약 정보
/// Trade summary information
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(as = TradeSummary)]
pub struct TradeSummary {
    /// Total number of buy trades
    /// 총 매수 횟수
    #[schema(example = 5)]
    pub total_buy_trades: u64,

    /// Total number of sell trades
    /// 총 매도 횟수
    #[schema(example = 2)]
    pub total_sell_trades: u64,

    /// Realized profit/loss from sell trades
    /// 매도로 인한 실현 손익
    /// Positive: profit, Negative: loss
    /// 양수: 이익, 음수: 손실
    /// Unit: USDT
    #[schema(value_type = String, example = "50.0")]
    pub realized_pnl: Decimal,
}

// =====================================================
// 포지션 응답 (Position Response)
// =====================================================

/// 특정 자산 포지션 응답 모델
/// Single asset position response model
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = AssetPositionResponse)]
pub struct AssetPositionResponse {
    /// Position information
    /// 포지션 정보
    pub position: AssetPosition,
}

/// 모든 자산 포지션 응답 모델
/// All assets positions response model
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = AllPositionsResponse)]
pub struct AllPositionsResponse {
    /// List of positions
    /// 포지션 목록
    pub positions: Vec<AssetPosition>,
}

