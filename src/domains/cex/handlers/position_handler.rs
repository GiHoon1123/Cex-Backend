// CEX Position Handler
// 거래소 포지션 핸들러
// 역할: 포지션 조회 API 엔드포인트 처리 (평균 매수가, 손익, 수익률 등)

use crate::shared::services::AppState;
use crate::shared::middleware::auth::AuthenticatedUser;
use crate::domains::cex::models::position::{AssetPositionResponse, AllPositionsResponse};
use axum::{extract::{Path, State}, http::StatusCode, Json};

/// 사용자의 특정 자산 포지션 조회 핸들러
/// Get position for specific asset
/// 
/// 경로: GET /api/cex/positions/{mint}
/// 인증: 필요 (JWT 토큰)
/// 
/// # Path Parameters
/// * `mint` - 자산 식별자 (예: "SOL", "USDT")
/// 
/// # Returns
/// * `200 OK` - 포지션 정보 반환 (자산을 보유한 경우)
/// * `404 Not Found` - 포지션 정보 없음 (자산을 보유하지 않거나 매수 거래가 없는 경우)
/// * `401 Unauthorized` - 인증 실패
/// * `500 Internal Server Error` - 서버 오류
/// 
/// # Response Example
/// ```json
/// {
///   "position": {
///     "mint": "SOL",
///     "current_balance": "11.0",
///     "available": "10.0",
///     "locked": "1.0",
///     "average_entry_price": "100.5",
///     "total_bought_amount": "15.0",
///     "total_bought_cost": "1507.5",
///     "current_market_price": "110.0",
///     "current_value": "1210.0",
///     "unrealized_pnl": "702.5",
///     "unrealized_pnl_percent": "46.6",
///     "trade_summary": {
///       "total_buy_trades": 5,
///       "total_sell_trades": 2,
///       "realized_pnl": "50.0"
///     }
///   }
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/cex/positions/{mint}",
    params(
        ("mint" = String, Path, description = "Asset identifier (e.g., 'SOL', 'USDT')")
    ),
    responses(
        (status = 200, description = "Position retrieved successfully", body = AssetPositionResponse),
        (status = 404, description = "Position not found (user does not hold this asset or has no buy trades)"),
        (status = 401, description = "Unauthorized (missing or invalid token)"),
        (status = 500, description = "Internal server error")
    ),
    tag = "CEX Positions",
    security(("BearerAuth" = []))
)]
pub async fn get_position(
    State(app_state): State<AppState>,
    Path(mint): Path<String>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<AssetPositionResponse>, (StatusCode, Json<serde_json::Value>)> {
    // JWT 토큰에서 추출한 user_id 사용
    // Use user_id extracted from JWT token
    let user_id = authenticated_user.user_id;

    // PositionService를 통해 포지션 정보 조회
    // Get position information through PositionService
    let position = app_state
        .cex_state
        .position_service
        .get_position(user_id, &mint)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch position: {}", e)
                })),
            )
        })?;

    match position {
        Some(pos) => Ok(Json(AssetPositionResponse { position: pos })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Position not found for asset: {}", mint)
            })),
        )),
    }
}

/// 사용자의 모든 자산 포지션 조회 핸들러
/// Get all positions for authenticated user
/// 
/// 경로: GET /api/cex/positions
/// 인증: 필요 (JWT 토큰)
/// 
/// # Returns
/// * `200 OK` - 포지션 목록 반환
/// * `401 Unauthorized` - 인증 실패
/// * `500 Internal Server Error` - 서버 오류
/// 
/// # Response Example
/// ```json
/// {
///   "positions": [
///     {
///       "mint": "SOL",
///       "current_balance": "11.0",
///       "available": "10.0",
///       "locked": "1.0",
///       "average_entry_price": "100.5",
///       "total_bought_amount": "15.0",
///       "total_bought_cost": "1507.5",
///       "current_market_price": "110.0",
///       "current_value": "1210.0",
///       "unrealized_pnl": "702.5",
///       "unrealized_pnl_percent": "46.6",
///       "trade_summary": {
///         "total_buy_trades": 5,
///         "total_sell_trades": 2,
///         "realized_pnl": "50.0"
///       }
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/cex/positions",
    responses(
        (status = 200, description = "Positions retrieved successfully", body = AllPositionsResponse),
        (status = 401, description = "Unauthorized (missing or invalid token)"),
        (status = 500, description = "Internal server error")
    ),
    tag = "CEX Positions",
    security(("BearerAuth" = []))
)]
pub async fn get_all_positions(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<AllPositionsResponse>, (StatusCode, Json<serde_json::Value>)> {
    // JWT 토큰에서 추출한 user_id 사용
    // Use user_id extracted from JWT token
    let user_id = authenticated_user.user_id;

    // PositionService를 통해 모든 포지션 정보 조회
    // Get all positions through PositionService
    let positions = app_state
        .cex_state
        .position_service
        .get_all_positions(user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch positions: {}", e)
                })),
            )
        })?;

    Ok(Json(AllPositionsResponse { positions }))
}

