use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// 스왑 가격 조회 API 응답 모델
// 역할: NestJS의 interface 같은 것
// Note: Jupiter API는 camelCase로 응답하므로 #[serde(rename = "...")]로 매핑
// utoipa::ToSchema: Swagger 문서 자동 생성용
// schema(as = "QuoteResponse"): Swagger에서 사용할 스키마 이름
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = QuoteResponse)]
pub struct QuoteResponse {
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u32,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: Option<String>,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
    // 추가 필드들 (optional로 처리)
    #[serde(rename = "inputMint")]
    pub input_mint: Option<String>,
    #[serde(rename = "outputMint")]
    pub output_mint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = RoutePlan)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: i32,
    // 추가 필드들
    pub bps: Option<i32>,
    #[serde(rename = "usdValue")]
    pub usd_value: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = SwapInfo)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

// 스왑 가격 조회 API 요청 파라미터
// utoipa::ToSchema: Swagger 문서 자동 생성용
// utoipa::IntoParams: 쿼리 파라미터로 사용하기 위한 trait
// 예시값: 모델에서 중앙 관리 (비즈니스 로직과 분리)
// Example values: centrally managed in model (separated from business logic)
#[derive(Debug, Serialize, Deserialize, ToSchema, utoipa::IntoParams)]
#[schema(as = QuoteRequest)]
pub struct QuoteRequest {
    /// Input token mint address
    /// 입력 토큰 주소
    ///
    /// Examples:
    /// - USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    /// - USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
    /// - BONK: DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263
    #[param(example = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")]
    #[schema(example = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")]
    pub input_mint: String,
    
    /// Output token mint address
    /// 출력 토큰 주소
    ///
    /// Examples:
    /// - SOL: So11111111111111111111111111111111111111112
    /// - USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    /// - USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
    #[param(example = "So11111111111111111111111111111111111111112")]
    #[schema(example = "So11111111111111111111111111111111111111112")]
    pub output_mint: String,
    
    /// Amount to swap (in lamports/minimal units)
    /// 스왑할 수량 (lamports/최소 단위)
    ///
    /// Examples:
    /// - 1 USDC = 1000000 (6 decimals)
    /// - 0.1 SOL = 100000000 (9 decimals)
    /// - 1 USDT = 1000000 (6 decimals)
    #[param(example = 1000000)]
    #[schema(example = 1000000)]
    pub amount: u64,
    
    /// Slippage in basis points (optional, default: 50 = 0.5%)
    /// 슬리피지 (기본점 단위, 선택사항, 기본값: 50 = 0.5%)
    ///
    /// Examples:
    /// - 50 = 0.5% slippage
    /// - 100 = 1% slippage
    /// - 200 = 2% slippage
    #[param(example = 50)]
    #[schema(example = 50)]
    pub slippage_bps: Option<u32>,
}

// Jupiter Swap Transaction API 요청 모델
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = SwapTransactionRequest)]
pub struct SwapTransactionRequest {
    /// Input token mint address (e.g., USDC)
    /// 입력 토큰 주소
    #[schema(example = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")]
    pub input_mint: String,

    /// Output token mint address (e.g., SOL)
    /// 출력 토큰 주소
    #[schema(example = "So11111111111111111111111111111111111111112")]
    pub output_mint: String,

    /// Amount to swap (in lamports/minimal units)
    /// 스왑할 수량
    #[schema(example = 1000000)]
    pub amount: u64,

    /// User public key (will sign the transaction)
    /// 사용자 공개 키 (트랜잭션 서명자)
    #[schema(example = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU")]
    pub user_public_key: String,

    /// Slippage in basis points (optional, default: 50 = 0.5%)
    /// 슬리피지 (기본점 단위, 선택사항, 기본값: 50 = 0.5%)
    #[schema(example = 50)]
    pub slippage_bps: Option<u32>,

    /// Wrap and unwrap SOL (optional, default: true)
    /// SOL 래핑/언래핑 여부 (선택사항, 기본값: true)
    pub wrap_and_unwrap_sol: Option<bool>,
}

// Jupiter Swap Transaction API 응답 모델
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(as = SwapTransactionResponse)]
pub struct SwapTransactionResponse {
    /// Transaction ID (BIGSERIAL, auto-generated by DB)
    /// 트랜잭션 ID (DB에서 자동 생성, 음수 불가능)
    pub id: Option<u64>,

    /// Base58 encoded transaction bytes
    /// Base58 인코딩된 트랜잭션 바이트
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String,

    /// Last valid block height
    /// 마지막 유효 블록 높이
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: Option<u64>,

    /// Prioritization fee in lamports (optional)
    /// 우선순위 수수료 (lamports 단위, 선택사항)
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: Option<u64>,
}

