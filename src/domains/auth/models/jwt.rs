use serde::{Deserialize, Serialize};

/// JWT Claims (토큰에 포함될 데이터)
/// JWT Claims (data to be included in token)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 사용자 ID
    /// User ID
    pub user_id: u64,
    
    /// 이메일
    /// Email
    pub email: String,
    
    /// 만료 시간 (Unix timestamp)
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    
    /// 발급 시간 (Unix timestamp)
    /// Issued at (Unix timestamp)
    pub iat: i64,
}

impl Claims {
    /// 새 Claims 생성 (만료 시간 자동 계산)
    /// Create new Claims (expiration time automatically calculated)
    pub fn new(user_id: u64, email: String, expiration_hours: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + (expiration_hours * 3600); // hours to seconds

        Self {
            user_id,
            email,
            exp,
            iat: now,
        }
    }
}