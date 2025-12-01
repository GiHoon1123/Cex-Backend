use sqlx::{PgPool, Row};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crate::domains::wallet::models::SolanaWallet;

// Solana 지갑 레포지토리
// 역할: NestJS의 Repository 같은 것
// SolanaWalletRepository: handles all database operations for Solana wallets

pub struct SolanaWalletRepository {
    pool: PgPool,
}

impl SolanaWalletRepository {
    // 레포지토리 생성
    // Create repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Solana 지갑 생성
    // Create Solana wallet
    // Note: 논리적 관계 검증 - user_id가 실제로 존재하는지 확인
    pub async fn create_solana_wallet(
        &self,
        user_id: u64,
        public_key: &str,
        encrypted_private_key: &str,
    ) -> Result<SolanaWallet> {
        // 논리적 관계 검증: user_id가 실제로 존재하는지 확인
        // Logical relationship validation: check if user_id actually exists
        let user_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
        )
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check if user exists")?;

        if !user_exists {
            return Err(anyhow::anyhow!("User not found: user_id={}", user_id));
        }

        // Solana 지갑 생성
        // Create Solana wallet
        let row = sqlx::query(
            r#"
            INSERT INTO solana_wallets (user_id, public_key, encrypted_private_key, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, public_key, encrypted_private_key, created_at, updated_at
            "#,
        )
        .bind(user_id as i64)
        .bind(public_key)
        .bind(encrypted_private_key)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create Solana wallet")?;

        Ok(SolanaWallet {
            id: row.get::<i64, _>("id") as u64,
            user_id: row.get::<i64, _>("user_id") as u64,
            public_key: row.get("public_key"),
            encrypted_private_key: row.get("encrypted_private_key"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    // 사용자 ID로 Solana 지갑 조회
    // Get Solana wallets by user ID
    pub async fn get_solana_wallets_by_user_id(&self, user_id: u64) -> Result<Vec<SolanaWallet>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, public_key, encrypted_private_key, created_at, updated_at
            FROM solana_wallets
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch Solana wallets by user_id")?;

        let wallets = rows
            .into_iter()
            .map(|row| SolanaWallet {
                id: row.get::<i64, _>("id") as u64,
                user_id: row.get::<i64, _>("user_id") as u64,
                public_key: row.get("public_key"),
                encrypted_private_key: row.get("encrypted_private_key"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(wallets)
    }

    // Public Key로 Solana 지갑 조회
    // Get Solana wallet by public key
    pub async fn get_solana_wallet_by_public_key(&self, public_key: &str) -> Result<Option<SolanaWallet>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, public_key, encrypted_private_key, created_at, updated_at
            FROM solana_wallets
            WHERE public_key = $1
            "#,
        )
        .bind(public_key)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch Solana wallet by public_key")?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        Ok(Some(SolanaWallet {
            id: row.get::<i64, _>("id") as u64,
            user_id: row.get::<i64, _>("user_id") as u64,
            public_key: row.get("public_key"),
            encrypted_private_key: row.get("encrypted_private_key"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    // ID로 Solana 지갑 조회
    // Get Solana wallet by ID
    pub async fn get_solana_wallet_by_id(&self, wallet_id: u64) -> Result<Option<SolanaWallet>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, public_key, encrypted_private_key, created_at, updated_at
            FROM solana_wallets
            WHERE id = $1
            "#,
        )
        .bind(wallet_id as i64)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch Solana wallet by id")?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        Ok(Some(SolanaWallet {
            id: row.get::<i64, _>("id") as u64,
            user_id: row.get::<i64, _>("user_id") as u64,
            public_key: row.get("public_key"),
            encrypted_private_key: row.get("encrypted_private_key"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }
}

