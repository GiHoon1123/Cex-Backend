use crate::domains::swap::models::{QuoteRequest, QuoteResponse, SwapTransactionRequest, SwapTransactionResponse};
use crate::shared::services::AppState;
use axum::{extract::Query, extract::State, http::StatusCode, Json};
use serde_json::json;

// 스왑 가격 조회 핸들러
// 역할: NestJS의 @Get() 핸들러 같은 것
// Handler: queries Jupiter for swap quotes
// 비즈니스 로직: 클라이언트를 호출해서 데이터 가져오기만 함
// utoipa::path: Swagger 문서 자동 생성용 어노테이션
// Note: 예시값은 모델(QuoteRequest)에서 관리
// Note: example values are managed in model (QuoteRequest)
#[utoipa::path(
    get,
    path = "/api/swap/quote",
    params(QuoteRequest),
    responses(
        (status = 200, description = "Quote retrieved successfully", body = QuoteResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "Jupiter API error")
    ),
    tag = "Swap"
)]
pub async fn get_quote(
    State(app_state): State<AppState>,
    Query(params): Query<QuoteRequest>,
) -> Result<Json<QuoteResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출 (비즈니스 로직)
    // Call service (business logic)
    let quote = app_state
        .swap_state
        .swap_service
        .get_quote(
            &params.input_mint,
            &params.output_mint,
            params.amount,
            params.slippage_bps,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": format!("Failed to get quote: {}", e)
                })),
            )
        })?;

    Ok(Json(quote))
}


// 스왑 트랜잭션 생성 핸들러
// 역할: NestJS의 @Post() 핸들러 같은 것
// Handler: creates swap transaction via Jupiter API and saves to DB
#[utoipa::path(
    post,
    path = "/api/swap/transaction",
    request_body = SwapTransactionRequest,
    responses(
        (status = 200, description = "Swap transaction created successfully", body = SwapTransactionResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error"),
        (status = 502, description = "Jupiter API error")
    ),
    tag = "Swap"
)]
pub async fn create_swap_transaction(
    State(app_state): State<AppState>,
    Json(request): Json<SwapTransactionRequest>,
) -> Result<Json<SwapTransactionResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출 (비즈니스 로직)
    // Call service (business logic)
    let swap_response = app_state
        .swap_state
        .swap_service
        .create_swap_transaction(request)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": format!("Failed to create swap transaction: {}", e)
                })),
            )
        })?;

    Ok(Json(swap_response))
}

