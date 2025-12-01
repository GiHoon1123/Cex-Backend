use crate::shared::clients::JupiterClient;
use crate::domains::swap::models::TokenSearchResponse;
use crate::shared::database::Database;
use anyhow::{Context, Result};

// 토큰 검색 서비스
// 역할: NestJS의 Service 같은 것
// TokenService: handles token search business logic
#[derive(Clone)]
pub struct TokenService {
    db: Database,
}

impl TokenService {
    // 생성자
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // 토큰 검색 (비즈니스 로직)
    // Search tokens (business logic)
    pub async fn search_tokens(
        &self,
        query: &str,
    ) -> Result<TokenSearchResponse> {
        let jupiter_client = JupiterClient::new()
            .context("Failed to create Jupiter client")?;

        let search_result = jupiter_client
            .search_tokens(query)
            .await
            .context("Failed to search tokens from Jupiter")?;

        Ok(search_result)
    }
}

