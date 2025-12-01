use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

// User 모델
// 역할: NestJS의 DTO/Entity 같은 것
// User: represents a user account stored in the database
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(as = User)]
pub struct User {
    /// User ID (BIGSERIAL, auto-generated)
    /// 사용자 ID (DB에서 자동 생성)
    pub id: u64,

    /// Email address (unique)
    /// 이메일 주소 (유니크)
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Password hash (bcrypt/argon2)
    /// 비밀번호 해시 (bcrypt/argon2)
    /// Note: 응답에서는 제외해야 함 (Serialize 제외)
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Username (optional)
    /// 사용자명 (선택사항)
    #[schema(example = "johndoe")]
    pub username: Option<String>,

    /// Created timestamp
    /// 생성 시간
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    /// 업데이트 시간
    pub updated_at: DateTime<Utc>,
}

// User 응답용 (password_hash 제외)
// User response DTO (without password_hash)
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = UserResponse)]
pub struct UserResponse {
    pub id: u64,
    pub email: String,
    pub username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

