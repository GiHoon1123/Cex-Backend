use crate::domains::auth::models::{
    SignupRequest, SignupResponse, SigninRequest, SigninResponse,
    RefreshTokenRequest, RefreshTokenResponse, LogoutRequest,
    UserResponse,
};
use crate::shared::services::AppState;
use crate::shared::errors::AuthError;
use axum::{extract::State, http::StatusCode, Json};
use crate::shared::middleware::auth::AuthenticatedUser;

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "User created successfully", body = SignupResponse),
        (status = 400, description = "Bad request (email already exists)"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn signup(
    State(app_state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<Json<SignupResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출 (비즈니스 로직)
    let user = app_state
        .auth_state
        .auth_service
        .signup(request)
        .await
        .map_err(|e: AuthError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(SignupResponse {
        user: user.into(),
        message: "User created successfully".to_string(),
    }))
}

// 로그인 핸들러
#[utoipa::path(
    post,
    path = "/api/auth/signin",
    request_body = SigninRequest,
    responses(
        (status = 200, description = "Login successful", body = SigninResponse),
        (status = 401, description = "Invalid email or password"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn signin(
    State(app_state): State<AppState>,
    Json(request): Json<SigninRequest>,
) -> Result<Json<SigninResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출 (비즈니스 로직 + Refresh Token 생성)
    let (user, refresh_token) = app_state
        .auth_state
        .auth_service
        .signin(request)
        .await
        .map_err(|e: AuthError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    // Access Token 발급
    let access_token = app_state
        .auth_state
        .jwt_service
        .generate_access_token(user.id, user.email.clone())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to generate token: {}", e) })),
            )
        })?;

    Ok(Json(SigninResponse {
        user: user.into(),
        access_token,
        refresh_token,
        message: "Login successful".to_string(),
    }))
}

/// 토큰 갱신 핸들러
/// Refresh token handler
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshTokenResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn refresh(
    State(app_state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    let (access_token, refresh_token) = app_state
        .auth_state
        .auth_service
        .refresh_access_token(&request.refresh_token)
        .await
        .map_err(|e: AuthError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(RefreshTokenResponse {
        access_token,
        refresh_token,
        message: "Token refreshed successfully".to_string(),
    }))
}

/// 로그아웃 핸들러
/// Logout handler
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Invalid token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(app_state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    app_state
        .auth_state
        .auth_service
        .logout(&request.refresh_token)
        .await
        .map_err(|e: AuthError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(serde_json::json!({
        "message": "Logout successful"
    })))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "User info retrieved successfully", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Auth"
)]
pub async fn get_me(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Service 호출 (비즈니스 로직)
    let user = app_state
        .auth_state
        .auth_service
        .get_user_info(authenticated_user.user_id)
        .await
        .map_err(|e: AuthError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(user.into()))
}