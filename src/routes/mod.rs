// Routes module: 라우팅 설정
// 역할: 모든 도메인의 라우터를 조합
// Routes module: combines all domain routers

use axum::Router;
use crate::shared::services::AppState;

// 각 도메인의 routes import
use crate::domains::auth::routes::create_auth_router;
use crate::domains::wallet::routes::create_wallet_router;
use crate::domains::swap::routes::{create_swap_router, create_tokens_router};
use crate::domains::cex::routes::create_cex_router;
use crate::domains::bot::routes::create_bot_router;

/// Create main router (combines all domain routers)
/// 메인 라우터 생성 (모든 도메인 라우터 조합)
pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/api/auth", create_auth_router())
        .nest("/api/wallets", create_wallet_router())
        .nest("/api/swap", create_swap_router())
        .nest("/api/tokens", create_tokens_router())
        .nest("/api/cex", create_cex_router())
        .nest("/api/bot", create_bot_router())
}
