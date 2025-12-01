// Auth domain state
// 인증 도메인 상태
use crate::shared::database::Database;
use crate::domains::auth::services::{AuthService, JwtService};

/// Auth domain state
/// 인증 도메인에서 필요한 서비스들을 포함하는 상태
#[derive(Clone)]
pub struct AuthState {
    pub auth_service: AuthService,
    pub jwt_service: JwtService,
}

impl AuthState {
    /// Create AuthState with database and JWT service
    /// AuthState 생성 (데이터베이스와 JWT 서비스 필요)
    pub fn new(db: Database, jwt_service: JwtService) -> Self {
        Self {
            auth_service: AuthService::with_jwt_service(db, jwt_service.clone()),
            jwt_service,
        }
    }
}

