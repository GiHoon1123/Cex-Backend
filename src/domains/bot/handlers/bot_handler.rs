use axum::{extract::State, Json, response::IntoResponse};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::shared::services::AppState;
use crate::shared::database::UserRepository;

/// 봇 데이터 삭제 요청
/// Delete bot data request
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteBotDataRequest {
    /// 삭제할 봇 이메일 (bot1@bot.com 또는 bot2@bot.com)
    /// Bot email to delete data for
    pub bot_email: String,
}

/// 봇 데이터 삭제 응답
/// Delete bot data response
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteBotDataResponse {
    /// 삭제된 주문 수
    /// Number of deleted orders
    pub deleted_orders: u64,
    
    /// 삭제된 거래 수
    /// Number of deleted trades
    pub deleted_trades: u64,
    
    /// 메시지
    /// Message
    pub message: String,
}

/// 봇 데이터 삭제
/// Delete bot data
/// 
/// 특정 봇의 모든 주문과 거래 내역을 삭제합니다.
/// 
/// # Arguments
/// * `State(app_state)` - 애플리케이션 상태
/// * `Json(request)` - 삭제 요청 (봇 이메일)
/// 
/// # Returns
/// * `Json(DeleteBotDataResponse)` - 삭제 결과
/// 
/// # Errors
/// - 봇 이메일이 유효하지 않음
/// - 데이터베이스 오류
#[utoipa::path(
    delete,
    path = "/api/bot/data",
    request_body = DeleteBotDataRequest,
    responses(
        (status = 200, description = "봇 데이터 삭제 성공", body = DeleteBotDataResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 500, description = "서버 오류")
    ),
    tag = "Bot"
)]
pub async fn delete_bot_data(
    State(app_state): State<AppState>,
    Json(request): Json<DeleteBotDataRequest>,
) -> axum::response::Response {
    // 데이터베이스 연결 가져오기
    let db = app_state.db.clone();
    
    // 봇 이메일로 사용자 ID 조회
    let user_repo = UserRepository::new(db.pool().clone());
    let user = match user_repo.get_user_by_email(&request.bot_email).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("[Bot Handler] Failed to find user: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };
    
    let user_id = match user {
        Some(u) => u.id,
        None => {
            return (axum::http::StatusCode::NOT_FOUND, 
                axum::Json(serde_json::json!({
                    "error": format!("Bot not found: {}", request.bot_email)
                }))).into_response();
        }
    };
    
    // 1. 모든 봇 user_id 조회 (bot1@bot.com, bot2@bot.com)
    let mut bot_user_ids = Vec::new();
    
    // bot1@bot.com 조회
    if let Ok(Some(bot1)) = user_repo.get_user_by_email("bot1@bot.com").await {
        bot_user_ids.push(bot1.id as i64);
    }
    
    // bot2@bot.com 조회
    if let Ok(Some(bot2)) = user_repo.get_user_by_email("bot2@bot.com").await {
        bot_user_ids.push(bot2.id as i64);
    }
    
    if bot_user_ids.is_empty() {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, 
            axum::Json(serde_json::json!({
                "error": "No bot users found"
            }))).into_response();
    }
    
    // 2. 모든 봇의 주문 ID 목록 조회
    let all_bot_order_ids: Vec<i64> = match sqlx::query_scalar(
        "SELECT id FROM orders WHERE user_id = ANY($1)"
    )
    .bind(&bot_user_ids)
    .fetch_all(db.pool())
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            eprintln!("[Bot Handler] Failed to fetch bot orders: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };
    
    // 3. 매수와 매도가 모두 봇인 거래만 삭제 (일반 사용자 거래 보존)
    let deleted_trades = if !all_bot_order_ids.is_empty() {
        match sqlx::query(
            "DELETE FROM trades WHERE buy_order_id = ANY($1) AND sell_order_id = ANY($1) RETURNING id"
        )
        .bind(&all_bot_order_ids)
        .fetch_all(db.pool())
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                eprintln!("[Bot Handler] Failed to delete trades: {}", e);
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
            }
        }
    } else {
        vec![]
    };
    
    // 3. 봇의 주문 삭제
    let deleted_orders = match sqlx::query(
        "DELETE FROM orders WHERE user_id = $1 RETURNING id"
    )
    .bind(user_id as i64)
    .fetch_all(db.pool())
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("[Bot Handler] Failed to delete orders: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };
    
    (axum::http::StatusCode::OK, axum::Json(DeleteBotDataResponse {
        deleted_orders: deleted_orders.len() as u64,
        deleted_trades: deleted_trades.len() as u64,
        message: format!(
            "Deleted {} orders and {} trades for bot {}",
            deleted_orders.len(),
            deleted_trades.len(),
            request.bot_email
        ),
    })).into_response()
}

/// 스케줄러 상태 조회
/// Get scheduler status
/// 
/// 봇 데이터 정리 스케줄러의 활성화 상태를 조회합니다.
#[utoipa::path(
    get,
    path = "/api/bot/cleanup-scheduler/status",
    responses(
        (status = 200, description = "스케줄러 상태 조회 성공", body = serde_json::Value),
        (status = 500, description = "서버 오류")
    ),
    tag = "Bot"
)]
pub async fn get_cleanup_scheduler_status(
    State(app_state): State<AppState>,
) -> axum::response::Response {
    let is_enabled = app_state.bot_cleanup_scheduler.is_enabled();
    
    (axum::http::StatusCode::OK, axum::Json(serde_json::json!({
        "enabled": is_enabled,
        "interval_seconds": 180,
        "message": if is_enabled {
            "스케줄러가 활성화되어 있습니다. 3분마다 봇 데이터를 정리합니다."
        } else {
            "스케줄러가 비활성화되어 있습니다."
        }
    }))).into_response()
}

/// 스케줄러 활성화
/// Enable scheduler
/// 
/// 봇 데이터 정리 스케줄러를 활성화합니다.
#[utoipa::path(
    post,
    path = "/api/bot/cleanup-scheduler/enable",
    responses(
        (status = 200, description = "스케줄러 활성화 성공", body = serde_json::Value),
        (status = 500, description = "서버 오류")
    ),
    tag = "Bot"
)]
pub async fn enable_cleanup_scheduler(
    State(app_state): State<AppState>,
) -> axum::response::Response {
    app_state.bot_cleanup_scheduler.enable();
    
    (axum::http::StatusCode::OK, axum::Json(serde_json::json!({
        "enabled": true,
        "message": "스케줄러가 활성화되었습니다. 3분마다 봇 데이터를 정리합니다."
    }))).into_response()
}

/// 스케줄러 비활성화
/// Disable scheduler
/// 
/// 봇 데이터 정리 스케줄러를 비활성화합니다.
#[utoipa::path(
    post,
    path = "/api/bot/cleanup-scheduler/disable",
    responses(
        (status = 200, description = "스케줄러 비활성화 성공", body = serde_json::Value),
        (status = 500, description = "서버 오류")
    ),
    tag = "Bot"
)]
pub async fn disable_cleanup_scheduler(
    State(app_state): State<AppState>,
) -> axum::response::Response {
    app_state.bot_cleanup_scheduler.disable();
    
    (axum::http::StatusCode::OK, axum::Json(serde_json::json!({
        "enabled": false,
        "message": "스케줄러가 비활성화되었습니다."
    }))).into_response()
}

