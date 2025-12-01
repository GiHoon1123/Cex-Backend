use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Jupiter Search API 응답: 배열로 직접 반환
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = TokenSearchResponse)]
pub struct TokenSearchResponse {
    pub tokens: Vec<Token>,
}

// 실제 API 응답은 배열이므로 Vec<Token>으로 직접 deserialize
// 클라이언트에서 Vec<Token>을 받아서 TokenSearchResponse로 변환

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = Token)]
pub struct Token {
    #[serde(rename = "id")]
    pub id: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "decimals")]
    pub decimals: Option<u8>,
    #[serde(rename = "icon")]
    pub icon: Option<String>,
    #[serde(rename = "tags")]
    pub tags: Option<Vec<String>>,
    // 추가 필드들 (optional)
    #[serde(rename = "tokenProgram")]
    pub token_program: Option<String>,
    #[serde(rename = "circSupply")]
    pub circ_supply: Option<f64>,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<f64>,
    #[serde(rename = "usdPrice")]
    pub usd_price: Option<f64>,
    #[serde(rename = "mcap")]
    pub mcap: Option<f64>,
    #[serde(rename = "liquidity")]
    pub liquidity: Option<f64>,
    #[serde(rename = "isVerified")]
    pub is_verified: Option<bool>,
}

// 토큰 검색 요청 파라미터
#[derive(Debug, Serialize, Deserialize, ToSchema, utoipa::IntoParams)]
#[schema(as = TokenSearchRequest)]
pub struct TokenSearchRequest {
    /// Search query (token name, symbol, or address)
    /// 검색어 (토큰 이름, 심볼, 또는 주소)
    /// 
    /// Examples:
    /// - USDC (symbol)
    /// - SOL (symbol)
    /// - USDT (symbol)
    /// - EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v (USDC address)
    /// - So11111111111111111111111111111111111111112 (SOL address)
    #[schema(example = "USDC")]
    pub query: String,
}
