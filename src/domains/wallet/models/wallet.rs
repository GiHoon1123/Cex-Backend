use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domains::wallet::models::SolanaWallet;

/// 지갑 생성 요청
/// Create wallet request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = CreateWalletRequest)]
pub struct CreateWalletRequest {
    /// User ID (사용자 ID)
    #[schema(example = 1)]
    pub user_id: u64,
}

/// 지갑 생성 응답
/// Create wallet response
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = CreateWalletResponse)]
pub struct CreateWalletResponse {
    pub wallet: SolanaWallet,
    pub message: String,
}

/// 지갑 조회 응답
/// Get wallet response
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = WalletResponse)]
pub struct WalletResponse {
    pub wallet: SolanaWallet,
}

/// 지갑 목록 응답
/// Get wallets response
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = WalletsResponse)]
pub struct WalletsResponse {
    pub wallets: Vec<SolanaWallet>,
}

/// 지갑 잔액 조회 응답
/// Wallet balance response (blockchain balance)
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = WalletBalanceResponse)]
pub struct WalletBalanceResponse {
    /// 잔액 (lamports)
    /// Balance in lamports
    #[schema(example = 1000000000)]
    pub balance_lamports: u64,
    
    /// 잔액 (SOL)
    /// Balance in SOL
    #[schema(example = 1.0)]
    pub balance_sol: f64,
    
    /// Public Key
    /// 공개 키
    #[schema(example = "7xKXtg2CW87d97TXJSDpbD5jBheTqA83TZRuJosgAsU")]
    pub public_key: String,
}

/// SOL 전송 요청
/// Transfer SOL request
#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = TransferSolRequest)]
pub struct TransferSolRequest {
    /// 수신자 Public Key
    /// Recipient public key
    #[schema(example = "7xKXtg2CW87d97TXJSDpbD5jBheTqA83TZRuJosgAsU")]
    pub to_public_key: String,
    
    /// 전송할 금액 (lamports)
    /// Amount to transfer (in lamports)
    #[schema(example = 1000000000)]
    pub amount_lamports: u64,
}

/// SOL 전송 응답
/// Transfer SOL response
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = TransferSolResponse)]
pub struct TransferSolResponse {
    /// 트랜잭션 서명
    /// Transaction signature
    #[schema(example = "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW")]
    pub signature: String,
    
    /// 메시지
    /// Message
    pub message: String,
}

/// 트랜잭션 상태 조회 응답
/// Get transaction status response
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = TransactionStatusResponse)]
pub struct TransactionStatusResponse {
    /// 트랜잭션 서명
    /// Transaction signature
    pub signature: String,
    
    /// 트랜잭션 상태 (true: 성공, false: 실패, None: 확인 중)
    /// Transaction status (true: success, false: failed, None: pending)
    pub status: Option<bool>,
}

