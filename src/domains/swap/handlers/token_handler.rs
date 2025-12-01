use crate::domains::swap::models::{TokenSearchRequest, TokenSearchResponse};
use crate::shared::services::AppState;
use axum::{extract::Query, extract::State, http::StatusCode, Json};
use serde_json::json;

// 토큰 검색 핸들러
// 역할: NestJS의 @Get() 핸들러 같은 것
#[utoipa::path(
    get,
    path = "/api/tokens/search",
    params(TokenSearchRequest),
    responses(
        (status = 200, description = "Tokens retrieved successfully", body = TokenSearchResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "Jupiter API error")
    ),
    tag = "Tokens"
)]
pub async fn search_tokens(
    State(app_state): State<AppState>,
    Query(params): Query<TokenSearchRequest>,
) -> Result<Json<TokenSearchResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출 (비즈니스 로직)
    // Call service (business logic)
    let search_result = app_state
        .swap_state
        .token_service
        .search_tokens(&params.query)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": format!("Failed to search tokens: {}", e)
                })),
            )
        })?;

    Ok(Json(search_result))
}