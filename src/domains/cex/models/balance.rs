use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// =====================================================
// UserBalance 모델
// =====================================================
// 역할: 사용자 자산 잔고를 나타내는 데이터 모델
// 설명: CEX 거래소에서 사용자가 보유한 각 자산의 잔고 관리
// 
// 잔고 구분:
// - available: 사용 가능한 잔고 (즉시 거래/출금 가능)
// - locked: 주문에 사용 중인 잔고 (주문 취소 또는 체결 시 해제)
// 
// 예시:
// - 사용자가 SOL 10개 보유, SOL 1개를 매도 주문 중
//   → available: 9.0, locked: 1.0
// - 주문이 체결되면
//   → available: 9.0, locked: 0.0
// =====================================================

/// 사용자 자산 잔고 정보 (데이터베이스에서 조회한 잔고)
/// User balance information (balance retrieved from database)
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(as = UserBalance)]
pub struct UserBalance {
    /// Balance record ID (BIGSERIAL, auto-generated)
    /// 잔고 레코드 ID (DB에서 자동 생성)
    pub id: u64,

    /// User ID (owner of this balance)
    /// 사용자 ID (이 잔고의 소유자)
    pub user_id: u64,

    /// Asset identifier (e.g., 'SOL', 'USDT', or SPL token mint address)
    /// 자산 식별자 (예: 'SOL', 'USDT', 또는 SPL 토큰 mint 주소)
    #[schema(example = "SOL")]
    pub mint_address: String,

    /// Available balance (can be used for trading/withdrawal immediately)
    /// 사용 가능 잔고 (거래/출금 즉시 사용 가능)
    /// Supports up to 9 decimal places
    /// 소수점 9자리까지 지원
    #[schema(value_type = String, example = "10.0")]
    pub available: Decimal,

    /// Locked balance (used in pending orders)
    /// 잠긴 잔고 (대기 중인 주문에 사용됨)
    /// Locked balance becomes available when order is filled or cancelled
    /// 주문이 체결되거나 취소되면 available로 전환됨
    #[schema(value_type = String, example = "1.0")]
    pub locked: Decimal,

    /// Created timestamp
    /// 잔고 레코드 생성 시간
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    /// 잔고 정보 마지막 업데이트 시간
    pub updated_at: DateTime<Utc>,
}

// =====================================================
// 잔고 응답 (Balance Response)
// =====================================================
/// 거래소 잔고 목록 응답 모델
/// Exchange balances list response model
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = ExchangeBalancesResponse)]
pub struct ExchangeBalancesResponse {
    /// List of balances
    /// 잔고 목록
    pub balances: Vec<UserBalance>,
}

/// 거래소 특정 자산 잔고 응답 모델
/// Exchange specific asset balance response model
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = ExchangeBalanceResponse)]
pub struct ExchangeBalanceResponse {
    /// Balance information
    /// 잔고 정보
    pub balance: UserBalance,
}

// =====================================================
// 잔고 생성/업데이트용 (Repository에서 사용)
// =====================================================
/// 잔고 생성 시 사용하는 내부 모델 (DB 저장용)
/// Internal model for creating balances (for database storage)
#[derive(Debug)]
pub struct UserBalanceCreate {
    /// User ID
    /// 사용자 ID
    pub user_id: u64,

    /// Asset identifier
    /// 자산 식별자
    pub mint_address: String,

    /// Initial available balance
    /// 초기 사용 가능 잔고
    pub available: Decimal,

    /// Initial locked balance (usually 0)
    /// 초기 잠긴 잔고 (보통 0)
    pub locked: Decimal,
}

/// 잔고 업데이트용 (잔고 차감/증가 시 사용)
/// Internal model for updating balances
#[derive(Debug)]
pub struct UserBalanceUpdate {
    /// Amount to add to available balance (can be negative to decrease)
    /// 사용 가능 잔고에 추가할 금액 (음수면 감소)
    pub available_delta: Option<Decimal>,

    /// Amount to add to locked balance (can be negative to decrease)
    /// 잠긴 잔고에 추가할 금액 (음수면 감소)
    pub locked_delta: Option<Decimal>,
}

