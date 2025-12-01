use thiserror::Error;
use axum::{http::StatusCode, Json};
use serde_json::json;

/// 인증 관련 에러
/// Authentication-related errors
#[derive(Error, Debug)]
pub enum AuthError {
    /// 이메일이 이미 존재함
    /// Email already exists
    #[error("Email already exists: {email}")]
    EmailAlreadyExists { email: String },

    /// 잘못된 이메일 또는 비밀번호
    /// Invalid email or password
    #[error("Invalid email or password")]
    InvalidCredentials,

    /// 사용자를 찾을 수 없음
    /// User not found
    #[error("User not found: id={id}")]
    UserNotFound { id: u64 },

    /// 사용자를 찾을 수 없음 (이메일로)
    /// User not found by email
    #[error("User not found: email={email}")]
    UserNotFoundByEmail { email: String },

    /// 비밀번호 해싱 실패
    /// Failed to hash password
    #[error("Failed to hash password: {0}")]
    PasswordHashingFailed(String),

    /// 비밀번호 검증 실패
    /// Failed to verify password
    #[error("Failed to verify password: {0}")]
    PasswordVerificationFailed(String),

    /// 데이터베이스 에러
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// 내부 서버 에러
    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// 잘못된 또는 만료된 토큰
    /// Invalid or expired token
    #[error("Invalid or expired token")]
    InvalidToken,

    /// 토큰이 제공되지 않음
    /// Token not provided
    #[error("Token not provided")]
    MissingToken,
}

/// AuthError를 HTTP 응답으로 변환
impl From<AuthError> for (StatusCode, Json<serde_json::Value>) {
    fn from(err: AuthError) -> Self {
        let (status, message) = match &err {
            AuthError::EmailAlreadyExists { .. } => {
                (StatusCode::BAD_REQUEST, err.to_string())
            }
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, err.to_string())
            }
            AuthError::UserNotFound { .. } => {
                (StatusCode::NOT_FOUND, err.to_string())
            }
            AuthError::UserNotFoundByEmail { .. } => {
                (StatusCode::NOT_FOUND, err.to_string())
            }
            AuthError::PasswordHashingFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            AuthError::PasswordVerificationFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            AuthError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            AuthError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            AuthError::InvalidToken | AuthError::MissingToken => {
                (StatusCode::UNAUTHORIZED, err.to_string())
            }
        };

        (status, Json(json!({ "error": message })))
    }
}

