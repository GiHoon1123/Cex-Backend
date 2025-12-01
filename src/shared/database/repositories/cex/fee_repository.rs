use sqlx::{PgPool, Row};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::domains::cex::models::fee::FeeConfig;

pub struct FeeConfigRepository {
    pool: PgPool,
}

impl FeeConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 거래쌍별 수수료 설정 조회 (가장 구체적인 것부터 검색)
    /// Get fee config by trading pair (most specific first)
    /// 
    /// 검색 순서:
    /// 1. base_mint와 quote_mint가 정확히 일치하는 설정
    /// 2. base_mint만 일치하는 설정 (quote_mint = NULL)
    /// 3. quote_mint만 일치하는 설정 (base_mint = NULL)
    /// 4. 모두 NULL인 기본 설정
    pub async fn get_fee_config(
        &self,
        base_mint: &str,
        quote_mint: &str,
    ) -> Result<Option<FeeConfig>> {
        // 가장 구체적인 설정부터 찾기
        let row = sqlx::query(
            r#"
            SELECT id, base_mint, quote_mint, fee_rate, fee_type, is_active, created_at, updated_at
            FROM fee_configs
            WHERE is_active = TRUE
              AND (
                (base_mint = $1 AND quote_mint = $2)
                OR (base_mint = $1 AND quote_mint IS NULL)
                OR (base_mint IS NULL AND quote_mint = $2)
                OR (base_mint IS NULL AND quote_mint IS NULL)
              )
            ORDER BY 
              CASE WHEN base_mint IS NOT NULL AND quote_mint IS NOT NULL THEN 1
                   WHEN base_mint IS NOT NULL THEN 2
                   WHEN quote_mint IS NOT NULL THEN 3
                   ELSE 4
              END
            LIMIT 1
            "#,
        )
        .bind(base_mint)
        .bind(quote_mint)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch fee config")?;

        Ok(row.map(|r| self.row_to_fee_config(&r)))
    }

    /// 모든 활성 수수료 설정 조회
    /// Get all active fee configs
    pub async fn get_all_active(&self) -> Result<Vec<FeeConfig>> {
        let rows = sqlx::query(
            r#"
            SELECT id, base_mint, quote_mint, fee_rate, fee_type, is_active, created_at, updated_at
            FROM fee_configs
            WHERE is_active = TRUE
            ORDER BY 
              CASE WHEN base_mint IS NOT NULL AND quote_mint IS NOT NULL THEN 1
                   WHEN base_mint IS NOT NULL THEN 2
                   WHEN quote_mint IS NOT NULL THEN 3
                   ELSE 4
              END,
              base_mint ASC, quote_mint ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch all active fee configs")?;

        Ok(rows.iter().map(|r| self.row_to_fee_config(r)).collect())
    }

    /// 수수료 설정 ID로 조회
    /// Get fee config by ID
    pub async fn get_by_id(&self, fee_config_id: u64) -> Result<Option<FeeConfig>> {
        let row = sqlx::query(
            r#"
            SELECT id, base_mint, quote_mint, fee_rate, fee_type, is_active, created_at, updated_at
            FROM fee_configs
            WHERE id = $1
            "#,
        )
        .bind(fee_config_id as i64)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch fee config by id")?;

        Ok(row.map(|r| self.row_to_fee_config(&r)))
    }

    /// Row를 FeeConfig로 변환하는 헬퍼 메서드
    /// Helper method to convert Row to FeeConfig
    fn row_to_fee_config(&self, row: &sqlx::postgres::PgRow) -> FeeConfig {
        FeeConfig {
            id: row.get::<i64, _>("id") as u64,
            base_mint: row.get("base_mint"),
            quote_mint: row.get("quote_mint"),
            fee_rate: row.get("fee_rate"),
            fee_type: row.get("fee_type"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

