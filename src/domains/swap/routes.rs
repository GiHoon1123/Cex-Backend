// Swap domain routes
// 스왑 도메인 라우터
use axum::{routing::{get, post}, Router};
use crate::domains::swap::handlers::{swap_handler, token_handler};
use crate::shared::services::AppState;

/// Create swap router
/// 스왑 라우터 생성
pub fn create_swap_router() -> Router<AppState> {
    Router::new()
        .route("/quote", get(swap_handler::get_quote))
        .route("/transaction", post(swap_handler::create_swap_transaction))
}

/// Create tokens router
/// 토큰 라우터 생성
pub fn create_tokens_router() -> Router<AppState> {
    Router::new()
        .route("/search", get(token_handler::search_tokens))
}

