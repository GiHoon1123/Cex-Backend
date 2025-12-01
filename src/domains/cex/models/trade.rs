use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// =====================================================
// Trade 모델
// =====================================================
// 역할: 체결 내역을 나타내는 데이터 모델
// 설명: 두 주문이 매칭되어 실제로 거래가 발생한 내역
// 
// 체결 과정:
// 1. 매수 주문과 매도 주문이 가격이 일치
// 2. 매칭 엔진이 두 주문을 매칭
// 3. 체결 발생 → Trade 레코드 생성
// 4. 각 사용자의 잔고 업데이트
// =====================================================

/// 체결 내역 정보 (데이터베이스에서 조회한 체결)
/// Trade information (trade retrieved from database)
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(as = Trade)]
pub struct Trade {
    /// Trade ID (BIGSERIAL, auto-generated)
    /// 체결 내역 ID (DB에서 자동 생성)
    pub id: u64,

    /// Buy order ID (who purchased)
    /// 매수 주문 ID (구매한 사람)
    pub buy_order_id: u64,

    /// Sell order ID (who sold)
    /// 매도 주문 ID (판매한 사람)
    pub sell_order_id: u64,

    /// Base asset (e.g., 'SOL', 'USDC')
    /// 기준 자산 (예: 'SOL', 'USDC')
    #[schema(example = "SOL")]
    pub base_mint: String,

    /// Quote currency (always 'USDT')
    /// 기준 통화 (항상 'USDT')
    #[schema(example = "USDT")]
    pub quote_mint: String,

    /// Trade price (execution price in USDT)
    /// 체결 가격 (USDT 기준 체결 가격)
    /// Example: 100.0 means 1 SOL = 100 USDT
    /// 예: 100.0은 1 SOL = 100 USDT를 의미
    #[schema(value_type = String, example = "100.0")]
    pub price: Decimal,

    /// Trade amount (executed amount in base asset)
    /// 체결 수량 (기준 자산 기준 체결된 수량)
    /// Example: 1.5 means 1.5 SOL was traded
    /// 예: 1.5는 1.5 SOL이 거래되었음을 의미
    #[schema(value_type = String, example = "1.0")]
    pub amount: Decimal,

    /// Trade timestamp
    /// 체결 발생 시간
    pub created_at: DateTime<Utc>,
}

// =====================================================
// 체결 내역 생성용 (Repository에서 사용)
// =====================================================
/// 체결 내역 생성 시 사용하는 내부 모델 (DB 저장용)
/// Internal model for creating trades (for database storage)
#[derive(Debug)]
pub struct TradeCreate {
    /// Buy order ID
    /// 매수 주문 ID
    pub buy_order_id: u64,

    /// Sell order ID
    /// 매도 주문 ID
    pub sell_order_id: u64,

    /// Buyer user ID
    /// 매수자 사용자 ID
    pub buyer_id: u64,

    /// Seller user ID
    /// 매도자 사용자 ID
    pub seller_id: u64,

    /// Base asset
    /// 기준 자산
    pub base_mint: String,

    /// Quote currency
    /// 기준 통화
    pub quote_mint: String,

    /// Trade price
    /// 체결 가격
    pub price: Decimal,

    /// Trade amount
    /// 체결 수량
    pub amount: Decimal,
}

// =====================================================
// 체결 내역 응답 (Trade Response)
// =====================================================
/// 체결 내역 목록 응답 모델
/// Response model for list of trades
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = TradesResponse)]
pub struct TradesResponse {
    /// List of trades
    /// 체결 내역 목록
    pub trades: Vec<Trade>,
}

