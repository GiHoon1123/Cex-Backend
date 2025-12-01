// Auth domain routes
// 인증 도메인 라우터
use axum::{routing::{get, post}, Router};
use crate::domains::auth::handlers::auth_handler;
use crate::shared::services::AppState;

/// Create authentication router
/// 인증 라우터 생성
pub fn create_auth_router() -> Router<AppState> {
    Router::new()
        .route("/signup", post(auth_handler::signup))
        .route("/signin", post(auth_handler::signin))
        .route("/refresh", post(auth_handler::refresh))
        .route("/logout", post(auth_handler::logout))
        .route("/me", get(auth_handler::get_me))
}

