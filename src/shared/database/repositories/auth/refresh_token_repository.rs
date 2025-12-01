use sqlx::{PgPool, Row};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crate::domains::auth::models::refresh_token::{RefreshToken, RefreshTokenCreate};

/// Refresh Token Repository
/// Refresh Token 데이터베이스 작업 처리
pub struct RefreshTokenRepository {
    pool: PgPool,
}

impl RefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Refresh Token 생성 (저장)
    /// Create and store refresh token
    pub async fn create(&self, data: RefreshTokenCreate) -> Result<RefreshToken> {
        let row = sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at, revoked, created_at, updated_at)
            VALUES ($1, $2, $3, FALSE, NOW(), NOW())
            RETURNING id, user_id, token_hash, expires_at, created_at, updated_at, revoked
            "#,
        )
        .bind(data.user_id as i64)  // u64 -> i64 변환 (DB는 BIGINT = i64)
        .bind(&data.token_hash)
        .bind(data.expires_at)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create refresh token")?;

        Ok(RefreshToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token_hash: row.get("token_hash"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            revoked: row.get("revoked"),
        })
    }

    /// Refresh Token 조회 (token_hash로)
    /// Find refresh token by token hash
    pub async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, updated_at, revoked
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find refresh token")?;

        if let Some(row) = row {
            Ok(Some(RefreshToken {
                id: row.get("id"),
                user_id: row.get("user_id"),
                token_hash: row.get("token_hash"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                revoked: row.get("revoked"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Refresh Token 무효화 (revoked = true)
    /// Revoke refresh token
    pub async fn revoke(&self, token_hash: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE, updated_at = NOW()
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await
        .context("Failed to revoke refresh token")?;

        Ok(())
    }

    /// 사용자의 모든 Refresh Token 무효화 (로그아웃 시)
    /// Revoke all refresh tokens for a user
    pub async fn revoke_all_for_user(&self, user_id: u64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE, updated_at = NOW()
            WHERE user_id = $1 AND revoked = FALSE
            "#,
        )
        .bind(user_id as i64)  // DB에는 i64로 저장
        .execute(&self.pool)
        .await
        .context("Failed to revoke all refresh tokens for user")?;

        Ok(())
    }

    /// 만료된 토큰 삭제 (정리 작업)
    /// Delete expired tokens (cleanup)
    pub async fn delete_expired(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete expired refresh tokens")?;

        Ok(result.rows_affected())
    }

    /// 특정 사용자의 유효한 Refresh Token 개수 조회
    /// Count valid refresh tokens for a user
    pub async fn count_valid_for_user(&self, user_id: u64) -> Result<i64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM refresh_tokens
            WHERE user_id = $1 AND revoked = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(user_id as i64)  // DB에는 i64로 저장
        .fetch_one(&self.pool)
        .await
        .context("Failed to count valid refresh tokens")?;

        Ok(row.get("count"))
    }
}

