use crate::domains::swap::models::{QuoteResponse, TokenSearchResponse, SwapTransactionRequest, SwapTransactionResponse, Token};
use anyhow::{Context, Result};
use serde::Deserialize;
use uuid::Uuid;

// Jupiter API 클라이언트
// 역할: NestJS의 HttpClient나 axios 같은 것
// Jupiter API client for external calls
pub struct JupiterClient {
    http_client: reqwest::Client,
    base_url: String,
}

impl JupiterClient {
    // 클라이언트 생성
    // Create new Jupiter client instance
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(false)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            http_client,
            base_url: "https://lite-api.jup.ag/ultra/v1".to_string(),
        })
    }

    // Quote 조회: Jupiter API 호출
    // Get quote: call Jupiter API
    // 역할: NestJS의 private async method 같은 것
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: Option<u32>,
    ) -> Result<QuoteResponse> {
        // URL 생성
        // Build request URL
        let mut url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}",
            self.base_url, input_mint, output_mint, amount
        );

        // slippageBps 파라미터 추가 (있을 경우)
        // Add slippageBps parameter if provided
        if let Some(slippage) = slippage_bps {
            url.push_str(&format!("&slippageBps={}", slippage));
        }

        println!("Requesting Jupiter API: {}", url);  // 디버깅용 로그

        // HTTP GET 요청
        // HTTP GET request
        let response = self
            .http_client
            .get(&url)
            .header("User-Agent", "api-server/1.0")
            .send()
            .await
            .context("Failed to send request to Jupiter API")?;

        // HTTP 상태 코드 확인
        // Check HTTP status code
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Jupiter API returned error: {} - {}",
                status,
                body
            );
        }

        // JSON 파싱
        // Parse JSON response
        let quote: QuoteResponse = response
            .json()
            .await
            .context("Failed to parse Jupiter API response")?;

        Ok(quote)
    }


    pub async fn search_tokens(&self, query: &str) -> Result<TokenSearchResponse> {
        // URL 생성
        let url = format!("{}/search?query={}", self.base_url, query);

        println!("Requesting Jupiter Search API: {}", url);

        // HTTP GET 요청
        let response = self
            .http_client
            .get(&url)
            .header("User-Agent", "api-server/1.0")
            .send()
            .await
            .context("Failed to send request to Jupiter Search API")?;

        // HTTP 상태 코드 확인
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Jupiter Search API returned error: {} - {}",
                status,
                body
            );
        }

        // JSON 파싱: 실제 API 응답은 배열이므로 Vec<Token>으로 파싱
        let tokens: Vec<Token> = response
            .json()
            .await
            .context("Failed to parse Jupiter Search API response")?;

        // TokenSearchResponse로 변환
        Ok(TokenSearchResponse { tokens })
    }

    // 스왑 트랜잭션 생성: Jupiter Swap API 호출
    // Create swap transaction: call Jupiter Swap API
    // 역할: NestJS의 private async method 같은 것
    // Note: lite-api.jup.ag/ultra/v1/swap는 다른 형식을 요구하므로
    //       일반적인 Jupiter Swap API 엔드포인트 사용 시도
    pub async fn create_swap_transaction(
        &self,
        request: &SwapTransactionRequest,
        quote: &QuoteResponse,
    ) -> Result<SwapTransactionResponse> {
        // URL 생성
        // Build request URL
        // Note: api.jup.ag/v6/swap는 API 키가 필요 (401 Unauthorized)
        //       lite-api.jup.ag/ultra/v1/swap는 다른 형식 요구 (signedTransaction)
        //       현재는 api.jup.ag 사용 (API 키 필요 시 설정 필요)
        // TODO: Jupiter 공식 문서 확인하여 무료 티어 지원 엔드포인트 찾기
        let swap_url = "https://api.jup.ag/v6/swap";
        let url = swap_url.to_string();

        println!("Requesting Jupiter Swap API: {}", url);

        // 요청 본문 생성 (Jupiter API 형식에 맞춤)
        // Build request body (according to Jupiter API format)
        // Note: requestId는 요청을 추적하기 위한 고유 ID
        let request_id = Uuid::new_v4().to_string();
        let request_body = serde_json::json!({
            "quoteResponse": quote,  // quote 객체 전체를 직렬화
            "userPublicKey": request.user_public_key,
            "requestId": request_id,
            "wrapAndUnwrapSol": request.wrap_and_unwrap_sol.unwrap_or(true),
            "dynamicComputeUnitLimit": true,
            "prioritizationFeeLamports": "auto",
        });

        // HTTP POST 요청
        // HTTP POST request
        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "api-server/1.0")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Jupiter Swap API")?;

        // HTTP 상태 코드 확인
        // Check HTTP status code
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Jupiter Swap API returned error: {} - {}",
                status,
                body
            );
        }

        // JSON 파싱: Jupiter API 응답 (swapTransaction, lastValidBlockHeight 등)
        // Parse JSON response: Jupiter API response (swapTransaction, lastValidBlockHeight, etc.)
        let swap_response: SwapTransactionResponseRaw = response
            .json()
            .await
            .context("Failed to parse Jupiter Swap API response")?;

        // SwapTransactionResponse로 변환 (id는 None, DB 저장 후 설정됨)
        // Convert to SwapTransactionResponse (id is None, will be set after DB save)
        Ok(SwapTransactionResponse {
            id: None,
            swap_transaction: swap_response.swap_transaction,
            last_valid_block_height: swap_response.last_valid_block_height,
            prioritization_fee_lamports: swap_response.prioritization_fee_lamports,
        })
    }
}

// Jupiter Swap API 원시 응답 모델 (내부용)
// Raw Jupiter Swap API response model (internal use)
#[derive(Debug, Deserialize)]
struct SwapTransactionResponseRaw {
    #[serde(rename = "swapTransaction")]
    swap_transaction: String,
    #[serde(rename = "lastValidBlockHeight")]
    last_valid_block_height: Option<u64>,
    #[serde(rename = "prioritizationFeeLamports")]
    prioritization_fee_lamports: Option<u64>,
}

// 기본 구현: new() 메서드 제공
// Default implementation: provides new() method
impl Default for JupiterClient {
    fn default() -> Self {
        Self::new().expect("Failed to create JupiterClient")
    }
}

