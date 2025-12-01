use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
// FromRow 제거: PostgreSQL BIGINT는 signed이므로 수동으로 변환 필요 

// DB 저장용 Transaction 모델
// Note: FromRow derive 제거 - PostgreSQL BIGINT는 signed이므로 수동으로 i64->u64 변환 필요
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = Transaction)]
pub struct Transaction {
    /// Transaction ID (BIGSERIAL, auto-generated)
    /// 트랜잭션 ID (DB에서 자동 생성, 음수 불가능)
    pub id: u64,

    /// Input token mint address
    pub input_mint: String,

    /// Output token mint address
    pub output_mint: String,

    /// Input amount (음수 불가능)
    pub amount: u64,

    /// Expected output amount (음수 불가능)
    pub expected_out_amount: Option<u64>,

    /// User public key
    pub user_public_key: String,

    /// Transaction bytes (base58 encoded)
    pub transaction_bytes: String,

    /// Quote response (JSON)
    pub quote_response: Option<serde_json::Value>,

    /// Transaction status (created, sent, failed, etc.)
    pub status: String,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

// ... 기존 TransactionStatus enum 그대로 ...
// Transaction 상태 enum (DB 저장용)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Created,    // 트랜잭션 생성됨
    Sent,       // 네트워크에 전송됨
    Confirmed,  // 확인됨
    Failed,     // 실패함
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Created => "created",
            TransactionStatus::Sent => "sent",
            TransactionStatus::Confirmed => "confirmed",
            TransactionStatus::Failed => "failed",
        }
    }
}