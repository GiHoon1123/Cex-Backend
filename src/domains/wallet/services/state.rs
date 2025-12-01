// Wallet domain state
// 지갑 도메인 상태
use crate::shared::database::Database;
use crate::domains::wallet::services::WalletService;
use crate::shared::errors::WalletError;
use anyhow::Result;

/// Wallet domain state
/// 지갑 도메인에서 필요한 서비스들을 포함하는 상태
#[derive(Clone)]
pub struct WalletState {
    pub wallet_service: WalletService,
}

impl WalletState {
    /// Create WalletState with database
    /// WalletState 생성 (데이터베이스 필요)
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self {
            wallet_service: WalletService::new(db)?,
        })
    }
}

