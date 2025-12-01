use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Refresh Token 모델 (DB 저장용)
/// Refresh Token model (for database storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub revoked: bool,
}

/// Refresh Token 생성 요청 (새 토큰 발급 시)
/// Refresh Token creation request (when issuing new token)
#[derive(Debug)]
pub struct RefreshTokenCreate {
    pub user_id: u64,  // u64로 통일 (User 모델과 일치)
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

