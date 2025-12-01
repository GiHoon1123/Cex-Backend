use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use crate::shared::services::AppState;
use crate::shared::errors::AuthError;
use serde_json::json;

/// 인증된 사용자 정보 (JWT 토큰에서 추출)
/// Authenticated user information (extracted from JWT token)
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: u64,
    pub email: String,
}

/// AuthenticatedUser를 Axum Extractor로 구현
/// 역할: NestJS의 @UseGuards(AuthGuard) 같은 것
/// 
/// 사용법:
/// ```rust
/// pub async fn create_wallet(
///     State(app_state): State<AppState>,
///     authenticated_user: AuthenticatedUser,  // <- 이렇게 사용!
/// ) -> Result<...> {
///     let user_id = authenticated_user.user_id;
///     // ...
/// }
/// ```
#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Authorization 헤더에서 토큰 추출
        let headers = &parts.headers;
        let auth_header = headers
            .get("Authorization")
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({ "error": "Missing authorization header" })),
                )
            })?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({ "error": "Invalid authorization header" })),
                )
            })?;

        // 2. "Bearer <token>" 형식 파싱
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "Invalid authorization format. Expected: 'Bearer <token>'"
                    })),
                )
            })?;

        // 3. JWT Service로 토큰 검증 (AppState에서 가져옴)
        let claims = state
            .auth_state
            .jwt_service
            .verify_access_token(token)
            .map_err(|e| {
                let status = match e {
                    AuthError::InvalidToken | AuthError::MissingToken => {
                        StatusCode::UNAUTHORIZED
                    }
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (
                    status,
                    axum::Json(json!({ "error": e.to_string() })),
                )
            })?;

        // 4. AuthenticatedUser 반환
        Ok(AuthenticatedUser {
            user_id: claims.user_id,
            email: claims.email,
        })
    }
}
