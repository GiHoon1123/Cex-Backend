// CEX Fee Service
// 거래소 수수료 서비스
// 역할: 거래 수수료 설정 조회 및 관리

use crate::shared::database::{Database, FeeConfigRepository};
use crate::domains::cex::models::fee::FeeConfig;
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use sqlx::Row;

/// 거래소 수수료 서비스
/// Exchange Fee Service
/// 
/// 역할:
/// - 거래쌍별 수수료 조회
/// - 기본 수수료 조회 (모든 거래쌍에 적용)
/// - 활성 수수료 설정 목록 조회
/// 
/// 사용처:
/// - OrderService: 주문 생성 시 수수료 계산
/// - Engine: 체결 시 수수료 계산
/// - FeeConfigHandler: 수수료 설정 조회 API
#[derive(Clone)]
pub struct FeeService {
    db: Database,
}

impl FeeService {
    /// 생성자
    /// Constructor
    /// 
    /// # Arguments
    /// * `db` - 데이터베이스 연결
    /// 
    /// # Returns
    /// FeeService 인스턴스
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 거래쌍별 수수료 조회
    /// Get fee config for a trading pair
    /// 
    /// 가장 구체적인 설정부터 찾습니다:
    /// 1. base_mint와 quote_mint가 정확히 일치하는 설정 (가장 우선)
    /// 2. base_mint만 일치하는 설정 (quote_mint = NULL)
    /// 3. quote_mint만 일치하는 설정 (base_mint = NULL)
    /// 4. 모두 NULL인 기본 설정 (최후의 수단)
    /// 
    /// # Arguments
    /// * `base_mint` - 기준 자산 (예: "SOL")
    /// * `quote_mint` - 기준 통화 (예: "USDT")
    /// 
    /// # Returns
    /// * `Ok(Some(FeeConfig))` - 수수료 설정이 있는 경우
    /// * `Ok(None)` - 수수료 설정이 없는 경우 (비정상, DB에 기본값이 있어야 함)
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # Examples
    /// ```
    /// // SOL/USDT 거래쌍의 수수료 조회
    /// let fee_config = fee_service.get_fee_config("SOL", "USDT").await?;
    /// 
    /// match fee_config {
    ///     Some(config) => {
    ///         println!("수수료율: {}% ({}%)", 
    ///                  config.fee_rate * Decimal::from(100),
    ///                  config.fee_rate);
    ///         // 예: 0.01% (0.0001)
    ///     }
    ///     None => {
    ///         println!("수수료 설정 없음 (비정상)");
    ///     }
    /// }
    /// ```
    pub async fn get_fee_config(
        &self,
        base_mint: &str,
        quote_mint: &str,
    ) -> Result<Option<FeeConfig>> {
        // Repository 생성
        // Create repository
        let fee_repo = FeeConfigRepository::new(self.db.pool().clone());

        // DB에서 거래쌍별 수수료 조회 (가장 구체적인 것부터)
        // Query fee config for trading pair from database (most specific first)
        let fee_config = fee_repo
            .get_fee_config(base_mint, quote_mint)
            .await
            .context(format!(
                "Failed to fetch fee config for {}/{}",
                base_mint, quote_mint
            ))?;

        Ok(fee_config)
    }

    /// 기본 수수료 조회 (모든 거래쌍에 적용되는 수수료)
    /// Get default fee config (applies to all trading pairs)
    /// 
    /// base_mint와 quote_mint가 모두 NULL인 수수료 설정을 조회합니다.
    /// 
    /// # Returns
    /// * `Ok(Some(FeeConfig))` - 기본 수수료 설정이 있는 경우
    /// * `Ok(None)` - 기본 수수료 설정이 없는 경우 (비정상)
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # Examples
    /// ```
    /// // 기본 수수료 조회
    /// let default_fee = fee_service.get_default_fee_config().await?;
    /// 
    /// if let Some(fee) = default_fee {
    ///     println!("기본 수수료율: {}%", fee.fee_rate * Decimal::from(100));
    /// }
    /// ```
    pub async fn get_default_fee_config(&self) -> Result<Option<FeeConfig>> {
        // base_mint와 quote_mint가 모두 NULL인 설정 조회
        // Query config where both base_mint and quote_mint are NULL
        let row = sqlx::query(
            r#"
            SELECT id, base_mint, quote_mint, fee_rate, fee_type, is_active, created_at, updated_at
            FROM fee_configs
            WHERE is_active = TRUE
              AND base_mint IS NULL
              AND quote_mint IS NULL
            LIMIT 1
            "#,
        )
        .fetch_optional(self.db.pool())
        .await
        .context("Failed to fetch default fee config")?;

        Ok(row.map(|r| FeeConfig {
            id: r.get::<i64, _>("id") as u64,
            base_mint: r.get("base_mint"),
            quote_mint: r.get("quote_mint"),
            fee_rate: r.get("fee_rate"),
            fee_type: r.get("fee_type"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// 거래쌍별 수수료 조회 (필수값 반환, 없으면 에러)
    /// Get fee config for trading pair (required, returns error if not found)
    /// 
    /// 수수료 설정이 없으면 에러를 반환합니다.
    /// OrderService나 Engine에서 사용할 때는 수수료가 반드시 있어야 하므로 이 메서드를 사용합니다.
    /// 
    /// # Arguments
    /// * `base_mint` - 기준 자산
    /// * `quote_mint` - 기준 통화
    /// 
    /// # Returns
    /// * `Ok(FeeConfig)` - 수수료 설정
    /// * `Err` - 수수료 설정이 없거나 데이터베이스 오류 시
    /// 
    /// # Examples
    /// ```
    /// // 주문 생성 시 수수료 조회 (필수)
    /// let fee_config = fee_service.get_fee_config_required("SOL", "USDT").await?;
    /// let fee_amount = trade_amount * fee_config.fee_rate;
    /// ```
    pub async fn get_fee_config_required(
        &self,
        base_mint: &str,
        quote_mint: &str,
    ) -> Result<FeeConfig> {
        self.get_fee_config(base_mint, quote_mint)
            .await?
            .context(format!(
                "Fee config not found for {}/{}. Please check fee_configs table.",
                base_mint, quote_mint
            ))
    }

    /// 기본 수수료 조회 (필수값 반환, 없으면 에러)
    /// Get default fee config (required, returns error if not found)
    /// 
    /// 기본 수수료 설정이 없으면 에러를 반환합니다.
    /// 
    /// # Returns
    /// * `Ok(FeeConfig)` - 기본 수수료 설정
    /// * `Err` - 기본 수수료 설정이 없거나 데이터베이스 오류 시
    pub async fn get_default_fee_config_required(&self) -> Result<FeeConfig> {
        self.get_default_fee_config()
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Default fee config not found. Please check fee_configs table.")
            })
    }

    /// 모든 활성 수수료 설정 조회
    /// Get all active fee configs
    /// 
    /// 관리자 페이지나 설정 확인용으로 사용됩니다.
    /// 
    /// # Returns
    /// * `Ok(Vec<FeeConfig>)` - 모든 활성 수수료 설정 목록
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # Examples
    /// ```
    /// // 모든 수수료 설정 조회
    /// let all_fees = fee_service.get_all_active_fees().await?;
    /// 
    /// for fee in all_fees {
    ///     println!("{}/{}: {}%", 
    ///              fee.base_mint.unwrap_or("ALL".to_string()),
    ///              fee.quote_mint.unwrap_or("ALL".to_string()),
    ///              fee.fee_rate * Decimal::from(100));
    /// }
    /// ```
    pub async fn get_all_active_fees(&self) -> Result<Vec<FeeConfig>> {
        let fee_repo = FeeConfigRepository::new(self.db.pool().clone());

        // DB에서 모든 활성 수수료 설정 조회
        // Query all active fee configs from database
        let fees = fee_repo
            .get_all_active()
            .await
            .context("Failed to fetch all active fee configs")?;

        Ok(fees)
    }

    /// 수수료 계산 헬퍼 메서드
    /// Calculate fee amount helper method
    /// 
    /// 거래 금액과 수수료율로 실제 수수료 금액을 계산합니다.
    /// 
    /// # Arguments
    /// * `trade_amount` - 거래 금액 (USDT 기준)
    /// * `fee_rate` - 수수료율 (예: 0.0001 = 0.01%)
    /// 
    /// # Returns
    /// 계산된 수수료 금액
    /// 
    /// # Examples
    /// ```
    /// // 100 USDT 거래, 0.01% 수수료
    /// let fee = fee_service.calculate_fee(
    ///     Decimal::from(100),
    ///     Decimal::from_str("0.0001").unwrap()
    /// );
    /// // 결과: 0.01 USDT
    /// ```
    pub fn calculate_fee(trade_amount: Decimal, fee_rate: Decimal) -> Decimal {
        trade_amount * fee_rate
    }
}

