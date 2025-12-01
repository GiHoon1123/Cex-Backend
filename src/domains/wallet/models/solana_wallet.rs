use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

// Solana 지갑 모델
// 역할: NestJS의 DTO/Entity 같은 것
// SolanaWallet: represents a Solana wallet stored in the database
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(as = SolanaWallet)]
pub struct SolanaWallet {
    /// Wallet ID (BIGSERIAL, auto-generated)
    /// 지갑 ID (DB에서 자동 생성)
    pub id: u64,

    /// User ID (foreign key to users table, logical relationship)
    /// 사용자 ID (users 테이블 논리적 관계)
    pub user_id: u64,

    /// Solana Public Key (Base58 encoded)
    /// Solana 공개 키 (Base58 인코딩)
    #[schema(example = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU")]
    pub public_key: String,

    /// Encrypted Private Key
    /// 암호화된 개인 키
    pub encrypted_private_key: String,

    /// Created timestamp
    /// 생성 시간
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    /// 업데이트 시간
    pub updated_at: DateTime<Utc>,
}

