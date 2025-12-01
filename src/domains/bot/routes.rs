use axum::Router;
use crate::shared::services::AppState;
use crate::domains::bot::handlers::bot_handler;

/// Bot 라우터 생성
/// Create bot router
/// 
/// 봇 관련 API 엔드포인트를 제공합니다.
pub fn create_bot_router() -> Router<AppState> {
    Router::new()
        .route("/data", axum::routing::delete(bot_handler::delete_bot_data))
        .route("/cleanup-scheduler/status", axum::routing::get(bot_handler::get_cleanup_scheduler_status))
        .route("/cleanup-scheduler/enable", axum::routing::post(bot_handler::enable_cleanup_scheduler))
        .route("/cleanup-scheduler/disable", axum::routing::post(bot_handler::disable_cleanup_scheduler))
}

