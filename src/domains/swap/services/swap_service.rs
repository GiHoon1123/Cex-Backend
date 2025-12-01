use crate::shared::clients::JupiterClient;
use crate::domains::swap::models::{QuoteRequest, QuoteResponse, SwapTransactionRequest, SwapTransactionResponse};
use crate::shared::database::Database;
use crate::shared::database::repositories::wallet::TransactionRepository;
use crate::domains::wallet::models::transaction::TransactionStatus;
use anyhow::{Context, Result};

// 스왑 서비스
// 역할: NestJS의 Service 같은 것
// SwapService: handles swap-related business logic
#[derive(Clone)]
pub struct SwapService {
    db: Database,
}

impl SwapService {
    // 생성자
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // 스왑 가격 조회 (비즈니스 로직)
    // Get swap quote (business logic)
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: Option<u32>,
    ) -> Result<QuoteResponse> {
        let jupiter_client = JupiterClient::new()
            .context("Failed to create Jupiter client")?;

        let quote = jupiter_client
            .get_quote(input_mint, output_mint, amount, slippage_bps)
            .await
            .context("Failed to fetch quote from Jupiter")?;

        Ok(quote)
    }

    // 스왑 트랜잭션 생성 (비즈니스 로직)
    // Create swap transaction (business logic)
    pub async fn create_swap_transaction(
        &self,
        request: SwapTransactionRequest,
    ) -> Result<SwapTransactionResponse> {
        // 1. Jupiter 클라이언트 생성
        let jupiter_client = JupiterClient::new()
            .context("Failed to create Jupiter client")?;

        // 2. Quote 조회
        let quote = jupiter_client
            .get_quote(
                &request.input_mint,
                &request.output_mint,
                request.amount,
                request.slippage_bps,
            )
            .await
            .context("Failed to fetch quote from Jupiter")?;

        // 3. Swap 트랜잭션 생성
        let mut swap_response = jupiter_client
            .create_swap_transaction(&request, &quote)
            .await
            .context("Failed to create swap transaction from Jupiter")?;

        // 4. 예상 출력 금액 파싱
        let expected_out_amount = quote.out_amount.parse::<u64>().ok();

        // 5. Quote 응답을 JSON으로 변환
        let quote_json = serde_json::to_value(&quote).ok();

        // 6. Repository 생성 (Service 내부에서)
        let repo = TransactionRepository::new(self.db.pool().clone());

        // 7. DB에 트랜잭션 저장
        let saved_transaction = repo
            .save_transaction(
                &request.input_mint,
                &request.output_mint,
                request.amount,
                expected_out_amount,
                &request.user_public_key,
                &swap_response.swap_transaction,
                quote_json,
                TransactionStatus::Created,
            )
            .await
            .context("Failed to save transaction to database")?;

        // 8. 응답에 생성된 ID 설정
        swap_response.id = Some(saved_transaction.id);

        Ok(swap_response)
    }
}

