// src/domains/auth/services/jwt_service.rs
use crate::shared::errors::AuthError;
use crate::domains::auth::models::jwt::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Sha256, Digest};
use rand::Rng;
use rand::distributions::Alphanumeric;

/// JWT 서비스
/// JWT Service for token generation and verification
#[derive(Clone)]
pub struct JwtService {
    secret: String,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    /// JWT Service 생성
    /// Create JWT Service
    pub fn new(secret: String) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        Self {
            secret,
            encoding_key,
            decoding_key,
        }
    }

    /// Access Token 발급 (짧은 수명)
    /// Generate Access Token (short lifetime)
    pub fn generate_access_token(&self, user_id: u64, email: String) -> Result<String, AuthError> {
        let claims = Claims::new(user_id, email, 1); // 1시간 만료

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to generate access token: {}", e)))
    }

    /// Refresh Token 생성 (랜덤 문자열, DB에 저장할 것)
    /// Generate Refresh Token (random string, to be stored in DB)
    pub fn generate_refresh_token(&self) -> String {
        // 64자 랜덤 문자열 생성
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        
        token
    }

    /// Refresh Token 해싱 (DB 저장용)
    /// Hash Refresh Token (for database storage)
    pub fn hash_refresh_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Access Token 검증
    /// Verify Access Token
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        AuthError::InvalidToken // 만료된 토큰
                    }
                    _ => AuthError::InvalidToken,
                }
            })?;

        Ok(token_data.claims)
    }
}