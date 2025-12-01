// Swap domain state
// 스왑 도메인 상태
use crate::shared::database::Database;
use crate::domains::swap::services::{SwapService, TokenService};

/// Swap domain state
/// 스왑 도메인에서 필요한 서비스들을 포함하는 상태
#[derive(Clone)]
pub struct SwapState {
    pub swap_service: SwapService,
    pub token_service: TokenService,
}

impl SwapState {
    /// Create SwapState with database
    /// SwapState 생성 (데이터베이스 필요)
    pub fn new(db: Database) -> Self {
        Self {
            swap_service: SwapService::new(db.clone()),
            token_service: TokenService::new(db),
        }
    }
}

