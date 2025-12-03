// =====================================================
// Health Check Handler
// =====================================================
// 역할: 서버 상태 확인 API
// 
// 엔드포인트: GET /api/health
// 응답: 200 OK with JSON body
// =====================================================

use axum::response::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// 헬스체크 응답 모델
/// Health check response model
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// 서버 상태
    /// Server status
    #[schema(example = "ok")]
    pub status: String,
    
    /// 서버 메시지
    /// Server message
    #[schema(example = "Server is running")]
    pub message: String,
}

/// 헬스체크 핸들러
/// Health check handler
/// 
/// 서버가 정상적으로 실행 중인지 확인하는 엔드포인트입니다.
/// 
/// # Response
/// - 200: 서버 정상 작동
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    ),
    tag = "Health"
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "Server is running".to_string(),
    })
}

