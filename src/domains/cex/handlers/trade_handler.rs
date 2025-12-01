use crate::domains::cex::models::trade::Trade;
use crate::shared::services::AppState;
use crate::shared::middleware::auth::AuthenticatedUser;
use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

// =====================================================
// Trade Handler
// =====================================================
// 역할: 체결 내역 관련 HTTP API 엔드포인트
// 
// 특징:
// - 읽기 전용 (조회만 담당)
// - 체결 생성은 Engine이 자동으로 처리
// =====================================================

/// 거래쌍별 체결 내역 쿼리 파라미터
/// Query parameters for trades by trading pair
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TradesQuery {
    /// 기준 자산 (예: "SOL")
    /// Base asset (e.g., "SOL")
    pub base_mint: String,
    
    /// 기준 통화 (예: "USDT", 기본값)
    /// Quote currency (e.g., "USDT", default)
    #[serde(default = "default_quote_mint")]
    pub quote_mint: String,
    
    /// 최대 조회 개수 (기본: 50, 최대: 1000)
    /// Limit (default: 50, max: 1000)
    #[serde(default)]
    pub limit: Option<i64>,
}

fn default_quote_mint() -> String {
    "USDT".to_string()
}

/// 거래쌍별 체결 내역 조회 핸들러
/// Get trades for trading pair handler
/// 
/// 특정 거래쌍(예: SOL/USDT)의 최근 체결 내역을 조회합니다.
/// 
/// # Query Parameters
/// - base_mint: 기준 자산 (required, 예: "SOL")
/// - quote_mint: 기준 통화 (optional, 기본: "USDT")
/// - limit: 최대 조회 개수 (optional, 기본: 50, 최대: 1000)
/// 
/// # Response
/// - 200: 체결 내역 조회 성공
/// - 400: 잘못된 요청
/// - 500: 서버 오류
/// 
/// # 용도
/// - 거래소 메인 페이지의 "최근 거래" 표시
/// - 차트 데이터 생성
#[utoipa::path(
    get,
    path = "/api/cex/trades",
    params(
        TradesQuery
    ),
    responses(
        (status = 200, description = "Trades retrieved successfully", body = Vec<Trade>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "CEX Trades"
)]
pub async fn get_trades(
    State(app_state): State<AppState>,
    Query(query): Query<TradesQuery>,
) -> Result<Json<Vec<Trade>>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출
    let trades = app_state
        .cex_state
        .trade_service
        .get_trades(&query.base_mint, &query.quote_mint, query.limit)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch trades: {}", e)
                })),
            )
        })?;

    Ok(Json(trades))
}

/// 내 체결 내역 쿼리 파라미터
/// Query parameters for my trades
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct MyTradesQuery {
    /// 자산 식별자 (선택, 특정 자산만 필터링)
    /// Asset identifier (optional, filter by specific asset)
    #[serde(default)]
    pub mint: Option<String>,
    
    /// 최대 조회 개수 (기본: 100)
    /// Limit (default: 100)
    #[serde(default)]
    pub limit: Option<i64>,
    
    /// 페이지네이션 오프셋 (기본: 0)
    /// Offset for pagination (default: 0)
    #[serde(default)]
    pub offset: Option<i64>,
}

/// 내 체결 내역 조회 핸들러
/// Get my trades handler
/// 
/// 현재 로그인한 사용자가 참여한 모든 체결 내역을 조회합니다.
/// 특정 자산(mint)을 지정하면 해당 자산의 거래 내역만 필터링합니다.
/// 
/// # Authentication
/// JWT 토큰 필요
/// 
/// # Query Parameters
/// - mint: 자산 식별자 (optional, 예: "SOL", 특정 자산만 필터링)
/// - limit: 최대 조회 개수 (optional, default: 100)
/// - offset: 페이지네이션 오프셋 (optional, default: 0)
/// 
/// # Response
/// - 200: 체결 내역 조회 성공
/// - 401: 인증 실패
/// - 500: 서버 오류
/// 
/// # 용도
/// - 사용자 마이페이지의 "내 거래 내역" 표시
/// - 특정 자산의 거래 내역 조회 (포지션 페이지에서 사용)
#[utoipa::path(
    get,
    path = "/api/cex/trades/my",
    params(
        MyTradesQuery
    ),
    responses(
        (status = 200, description = "My trades retrieved successfully", body = Vec<Trade>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "CEX Trades",
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn get_my_trades(
    State(app_state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Query(query): Query<MyTradesQuery>,
) -> Result<Json<Vec<Trade>>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출 (mint 파라미터 전달)
    let trades = app_state
        .cex_state
        .trade_service
        .get_my_trades(user_id, query.mint.as_deref(), query.limit, query.offset)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch my trades: {}", e)
                })),
            )
        })?;

    Ok(Json(trades))
}

/// 최근 가격 응답 모델
/// Latest price response model
#[derive(Debug, Serialize, ToSchema)]
pub struct LatestPriceResponse {
    /// 기준 자산
    /// Base asset
    pub base_mint: String,
    
    /// 기준 통화
    /// Quote currency
    pub quote_mint: String,
    
    /// 최근 체결 가격 (없으면 null)
    /// Latest trade price (null if no trades)
    pub price: Option<rust_decimal::Decimal>,
}

/// 최근 가격 쿼리 파라미터
/// Query parameters for latest price
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct LatestPriceQuery {
    /// 기준 자산 (예: "SOL")
    /// Base asset (e.g., "SOL")
    pub base_mint: String,
    
    /// 기준 통화 (예: "USDT", 기본값)
    /// Quote currency (e.g., "USDT", default)
    #[serde(default = "default_quote_mint")]
    pub quote_mint: String,
}

/// 최근 체결 가격 조회 핸들러
/// Get latest price handler
/// 
/// 특정 거래쌍의 가장 최근 체결 가격을 조회합니다.
/// 
/// # Query Parameters
/// - base_mint: 기준 자산 (required, 예: "SOL")
/// - quote_mint: 기준 통화 (optional, 기본: "USDT")
/// 
/// # Response
/// - 200: 최근 가격 조회 성공
/// - 400: 잘못된 요청
/// - 500: 서버 오류
/// 
/// # 용도
/// - 현재가 표시 (차트)
/// - 시장가 주문 시 참고 가격
#[utoipa::path(
    get,
    path = "/api/cex/price",
    params(
        LatestPriceQuery
    ),
    responses(
        (status = 200, description = "Latest price retrieved successfully", body = LatestPriceResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "CEX Trades"
)]
pub async fn get_latest_price(
    State(app_state): State<AppState>,
    Query(query): Query<LatestPriceQuery>,
) -> Result<Json<LatestPriceResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출
    let price = app_state
        .cex_state
        .trade_service
        .get_latest_price(&query.base_mint, &query.quote_mint)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch latest price: {}", e)
                })),
            )
        })?;

    Ok(Json(LatestPriceResponse {
        base_mint: query.base_mint,
        quote_mint: query.quote_mint,
        price,
    }))
}

/// 24시간 거래량 응답 모델
/// 24-hour volume response model
#[derive(Debug, Serialize, ToSchema)]
pub struct VolumeResponse {
    /// 기준 자산
    /// Base asset
    pub base_mint: String,
    
    /// 기준 통화
    /// Quote currency
    pub quote_mint: String,
    
    /// 기준 자산 거래량 (예: SOL)
    /// Base asset volume (e.g., SOL)
    pub base_volume: rust_decimal::Decimal,
    
    /// 기준 통화 거래량 (예: USDT)
    /// Quote currency volume (e.g., USDT)
    pub quote_volume: rust_decimal::Decimal,
}

/// 24시간 거래량 쿼리 파라미터
/// Query parameters for 24h volume
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct VolumeQuery {
    /// 기준 자산 (예: "SOL")
    /// Base asset (e.g., "SOL")
    pub base_mint: String,
    
    /// 기준 통화 (예: "USDT", 기본값)
    /// Quote currency (e.g., "USDT", default)
    #[serde(default = "default_quote_mint")]
    pub quote_mint: String,
}

/// 24시간 거래량 조회 핸들러
/// Get 24-hour volume handler
/// 
/// 특정 거래쌍의 최근 24시간 거래량을 조회합니다.
/// 
/// # Query Parameters
/// - base_mint: 기준 자산 (required, 예: "SOL")
/// - quote_mint: 기준 통화 (optional, 기본: "USDT")
/// 
/// # Response
/// - 200: 거래량 조회 성공
/// - 400: 잘못된 요청
/// - 500: 서버 오류
/// 
/// # 용도
/// - 거래소 메인 페이지 통계
/// - 인기 거래쌍 표시
#[utoipa::path(
    get,
    path = "/api/cex/volume",
    params(
        VolumeQuery
    ),
    responses(
        (status = 200, description = "24h volume retrieved successfully", body = VolumeResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "CEX Trades"
)]
pub async fn get_24h_volume(
    State(app_state): State<AppState>,
    Query(query): Query<VolumeQuery>,
) -> Result<Json<VolumeResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출
    let (base_volume, quote_volume) = app_state
        .cex_state
        .trade_service
        .get_24h_volume(&query.base_mint, &query.quote_mint)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch 24h volume: {}", e)
                })),
            )
        })?;

    Ok(Json(VolumeResponse {
        base_mint: query.base_mint,
        quote_mint: query.quote_mint,
        base_volume,
        quote_volume,
    }))
}

