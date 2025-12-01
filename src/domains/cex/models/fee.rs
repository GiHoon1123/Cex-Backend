use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// =====================================================
// FeeConfig 모델
// =====================================================
// 역할: 거래 수수료 설정을 나타내는 데이터 모델
// 설명: 거래소의 거래 수수료율을 관리
// 
// 수수료 계산:
// - 수수료 = 거래 금액 * fee_rate
// - 예: fee_rate = 0.0001 (0.01%), 거래 금액 = 100 USDT
//   → 수수료 = 100 * 0.0001 = 0.01 USDT
// 
// 수수료 유형:
// - 'taker': 시장가 주문자 수수료
// - 'maker': 지정가 주문자 수수료
// - 'both': 모두 동일한 수수료 (현재는 이 방식)
// =====================================================

/// 수수료 설정 정보 (데이터베이스에서 조회한 수수료 설정)
/// Fee configuration (fee config retrieved from database)
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(as = FeeConfig)]
pub struct FeeConfig {
    /// Fee config ID (BIGSERIAL, auto-generated)
    /// 수수료 설정 ID (DB에서 자동 생성)
    pub id: u64,

    /// Base asset (NULL means applies to all trading pairs)
    /// 기준 자산 (NULL이면 모든 거래쌍에 적용)
    #[schema(example = "SOL")]
    pub base_mint: Option<String>,

    /// Quote currency (NULL means applies to all trading pairs)
    /// 기준 통화 (NULL이면 모든 거래쌍에 적용)
    #[schema(example = "USDT")]
    pub quote_mint: Option<String>,

    /// Fee rate (decimal, e.g., 0.0001 = 0.01%)
    /// 수수료율 (소수점, 예: 0.0001 = 0.01%)
    /// Fee = trade_amount * fee_rate
    /// 수수료 = 거래 금액 * 수수료율
    #[schema(value_type = String, example = "0.0001")]
    pub fee_rate: Decimal,

    /// Fee type: 'taker', 'maker', or 'both'
    /// 수수료 유형: 'taker' (시장가), 'maker' (지정가), 'both' (모두 동일)
    #[schema(example = "both")]
    pub fee_type: String,

    /// Is this fee config active?
    /// 이 수수료 설정이 활성화되어 있는가?
    pub is_active: bool,

    /// Created timestamp
    /// 수수료 설정 생성 시간
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    /// 수수료 설정 마지막 업데이트 시간
    pub updated_at: DateTime<Utc>,
}

// =====================================================
// 수수료 설정 생성/업데이트용 (Repository에서 사용)
// =====================================================
/// 수수료 설정 생성 시 사용하는 내부 모델 (DB 저장용)
/// Internal model for creating fee configs (for database storage)
#[derive(Debug)]
pub struct FeeConfigCreate {
    /// Base asset (None for all pairs)
    /// 기준 자산 (None이면 모든 거래쌍)
    pub base_mint: Option<String>,

    /// Quote currency (None for all pairs)
    /// 기준 통화 (None이면 모든 거래쌍)
    pub quote_mint: Option<String>,

    /// Fee rate
    /// 수수료율
    pub fee_rate: Decimal,

    /// Fee type
    /// 수수료 유형
    pub fee_type: String,

    /// Is active
    /// 활성화 여부
    pub is_active: bool,
}

