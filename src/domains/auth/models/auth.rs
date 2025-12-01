use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domains::auth::models::user::UserResponse;

// 회원가입 요청 모델
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = SignupRequest)]
pub struct SignupRequest {
    /// Email address
    /// 이메일 주소
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Password (will be hashed)
    /// 비밀번호 (해싱됨)
    #[schema(example = "password123")]
    pub password: String,

    /// Username (optional)
    /// 사용자명 (선택사항)
    #[schema(example = "johndoe")]
    pub username: Option<String>,
}

// 회원가입 응답 모델
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = SignupResponse)]
pub struct SignupResponse {
    /// User information (without password)
    /// 사용자 정보 (비밀번호 제외)
    pub user: UserResponse,

    /// Success message
    /// 성공 메시지
    pub message: String,
}

// 로그인 요청 모델
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = SigninRequest)]
pub struct SigninRequest {
    /// Email address
    /// 이메일 주소
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Password
    /// 비밀번호
    #[schema(example = "password123")]
    pub password: String,
}

// 로그인 응답 모델
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = SigninResponse)]
pub struct SigninResponse {
    /// User information (without password)
    /// 사용자 정보 (비밀번호 제외)
    pub user: UserResponse,

    /// JWT Access Token (짧은 수명)
    /// JWT Access Token (short lifetime)
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,

    /// Refresh Token (긴 수명, DB에 저장)
    /// Refresh Token (long lifetime, stored in database)
    #[schema(example = "abc123def456...")]
    pub refresh_token: String,

    /// Success message
    /// 성공 메시지
    pub message: String,
}

// 토큰 갱신 요청 모델
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = RefreshTokenRequest)]
pub struct RefreshTokenRequest {
    /// Refresh Token
    /// 리프레시 토큰
    #[schema(example = "abc123def456...")]
    pub refresh_token: String,
}

// 토큰 갱신 응답 모델
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = RefreshTokenResponse)]
pub struct RefreshTokenResponse {
    /// 새 Access Token
    /// New Access Token
    pub access_token: String,

    /// 새 Refresh Token
    /// New Refresh Token
    pub refresh_token: String,

    /// Success message
    /// 성공 메시지
    pub message: String,
}

// 로그아웃 요청 모델
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = LogoutRequest)]
pub struct LogoutRequest {
    /// Refresh Token
    /// 리프레시 토큰
    #[schema(example = "abc123def456...")]
    pub refresh_token: String,
}

