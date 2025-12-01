use thiserror::Error;
use axum::{http::StatusCode, Json};
use serde_json::json;

/// 지갑 관련 에러
/// Wallet-related errors
#[derive(Error, Debug)]
pub enum WalletError {
    /// 지갑을 찾을 수 없음
    /// Wallet not found
    #[error("Wallet not found: id={id}")]
    NotFound { id: u64 },

    /// 사용자당 지갑이 이미 존재함
    /// Wallet already exists for user
    #[error("Wallet already exists for user: user_id={user_id}")]
    WalletAlreadyExists { user_id: u64 },

    /// 지갑을 찾을 수 없음 (Public Key로)
    /// Wallet not found by public key
    #[error("Wallet not found: public_key={public_key}")]
    NotFoundByPublicKey { public_key: String },

    /// 잔액 부족
    /// Insufficient balance
    #[error("Insufficient balance: required={required}, available={available}")]
    InsufficientBalance { required: u64, available: u64 },

    /// Public Key 파싱 실패
    /// Failed to parse public key
    #[error("Failed to parse public key: {public_key}")]
    InvalidPublicKey { public_key: String },

    /// Private Key 복호화 실패
    /// Failed to decrypt private key
    #[error("Failed to decrypt private key: {0}")]
    DecryptionFailed(String),

    /// Solana 네트워크 에러
    /// Solana network error
    #[error("Solana network error: {0}")]
    SolanaNetworkError(String),

    /// 트랜잭션 생성 실패
    /// Failed to create transaction
    #[error("Failed to create transaction: {0}")]
    TransactionCreationFailed(String),

    /// 트랜잭션 전송 실패
    /// Failed to send transaction
    #[error("Failed to send transaction: {0}")]
    TransactionSendFailed(String),

    /// 데이터베이스 에러
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// 내부 서버 에러
    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// WalletError를 HTTP 응답으로 변환
impl From<WalletError> for (StatusCode, Json<serde_json::Value>) {
    fn from(err: WalletError) -> Self {
        let (status, message) = match &err {
            WalletError::NotFound { .. } => {
                (StatusCode::NOT_FOUND, err.to_string())
            }
            WalletError::NotFoundByPublicKey { .. } => {
                (StatusCode::NOT_FOUND, err.to_string())
            }
            WalletError::WalletAlreadyExists { .. } => {
                (StatusCode::BAD_REQUEST, err.to_string())
            }
            WalletError::InsufficientBalance { .. } => {
                (StatusCode::BAD_REQUEST, err.to_string())
            }
            WalletError::InvalidPublicKey { .. } => {
                (StatusCode::BAD_REQUEST, err.to_string())
            }
            WalletError::DecryptionFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            WalletError::SolanaNetworkError(_) => {
                (StatusCode::BAD_GATEWAY, err.to_string())
            }
            WalletError::TransactionCreationFailed(_) => {
                (StatusCode::BAD_REQUEST, err.to_string())
            }
            WalletError::TransactionSendFailed(_) => {
                (StatusCode::BAD_GATEWAY, err.to_string())
            }
            WalletError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            WalletError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        };

        (status, Json(json!({ "error": message })))
    }
}

