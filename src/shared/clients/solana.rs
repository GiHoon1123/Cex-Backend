use anyhow::{Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    account::Account,
    signer::keypair::Keypair,
    signer::Signer,
    native_token::lamports_to_sol,
    transaction::Transaction,
    hash::Hash,
    system_instruction,
    signature::Signature,
};
use std::str::FromStr;
use std::sync::Arc;

/// Solana RPC 클라이언트
/// 로컬 노드 (http://localhost:8899)와 통신
#[derive(Clone)]
pub struct SolanaClient {
    rpc_client: Arc<RpcClient>,
    rpc_url: String,
    commitment: CommitmentConfig,
}

impl SolanaClient {
    /// SolanaClient 생성 (로컬 네트워크 고정)
    /// Create SolanaClient (fixed to local network)
    pub fn new() -> Result<Self> {
        let rpc_url = "http://localhost:8899";

        let rpc_client = Arc::new(
            RpcClient::new_with_commitment(
                rpc_url.to_string(),
                CommitmentConfig::confirmed(),
            )
        );

        Ok(Self {
            rpc_client,
            rpc_url: rpc_url.to_string(),
            commitment: CommitmentConfig::confirmed(),
        })
    }

    /// 새 지갑(Keypair) 생성
    /// Generate new wallet (Keypair)
    pub fn generate_wallet() -> Keypair {
        Keypair::new()
    }

    /// Public Key 문자열을 Pubkey로 변환
    /// Parse public key string to Pubkey
    pub fn parse_pubkey(pubkey_str: &str) -> Result<Pubkey> {
        Pubkey::from_str(pubkey_str)
            .context(format!("Failed to parse public key: {}", pubkey_str))
    }

    /// 계정 정보 조회
    /// Get account information
    pub async fn get_account(&self, pubkey: &Pubkey) -> Result<Option<Account>> {
        // RPC 클라이언트는 Account를 직접 반환하지만, 존재하지 않으면 에러
        // 따라서 Option 처리 필요
        match self
            .rpc_client
            .get_account(pubkey)
            .await
        {
            Ok(account_info) => Ok(Some(Account {
                lamports: account_info.lamports,
                data: account_info.data,
                owner: account_info.owner,
                executable: account_info.executable,
                rent_epoch: account_info.rent_epoch,
            })),
            Err(_) => Ok(None), // 계정이 없으면 None 반환
        }
    }

    /// SOL 잔액 조회 (lamports 단위)
    /// Get SOL balance (in lamports)
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        self.rpc_client
            .get_balance(pubkey)
            .await
            .context(format!("Failed to get balance for {}", pubkey))
    }

    /// SOL 잔액 조회 (SOL 단위)
    /// Get SOL balance (in SOL)
    pub async fn get_balance_sol(&self, pubkey: &Pubkey) -> Result<f64> {
        let lamports = self.get_balance(pubkey).await?;
        Ok(lamports_to_sol(lamports))
    }

    /// 최신 블록해시 조회 (트랜잭션 서명에 필요)
    /// Get latest blockhash (required for transaction signing)
    pub async fn get_latest_blockhash(&self) -> Result<Hash> {
        self.rpc_client
            .get_latest_blockhash()
            .await
            .context("Failed to get latest blockhash")
    }

    /// SOL 전송 트랜잭션 생성
    /// Create SOL transfer transaction
    pub async fn create_transfer_transaction(
        &self,
        from_keypair: &Keypair,
        to_pubkey: &Pubkey,
        amount_lamports: u64,
    ) -> Result<Transaction> {
        let latest_blockhash = self.get_latest_blockhash().await?;

        let instruction = system_instruction::transfer(
            &from_keypair.pubkey(),
            to_pubkey,
            amount_lamports,
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&from_keypair.pubkey()),
            &[from_keypair],
            latest_blockhash,
        );

        Ok(transaction)
    }

    /// 트랜잭션 전송 및 확인 대기
    /// Send transaction and wait for confirmation
    pub async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<String> {
        let signature = self
            .rpc_client
            .send_and_confirm_transaction(transaction)
            .await
            .context("Failed to send and confirm transaction")?;

        Ok(signature.to_string())
    }

    /// 트랜잭션 전송 (확인 대기 없음)
    /// Send transaction (without waiting for confirmation)
    pub async fn send_transaction(&self, transaction: &Transaction) -> Result<String> {
        let signature = self
            .rpc_client
            .send_transaction(transaction)
            .await
            .context("Failed to send transaction")?;

        Ok(signature.to_string())
    }

    /// 트랜잭션 서명 (전송 전)
    /// Sign transaction (before sending)
    pub fn sign_transaction(transaction: &mut Transaction, keypair: &Keypair) {
        transaction.sign(&[keypair], transaction.message.recent_blockhash);
    }

    /// 트랜잭션 상태 확인 (서명으로)
    /// Get transaction status by signature
    pub async fn get_signature_status(&self, signature: &str) -> Result<Option<bool>> {
        let sig: Signature = signature
            .parse()
            .context(format!("Failed to parse signature: {}", signature))?;

        match self
            .rpc_client
            .get_signature_status(&sig)
            .await
        {
            Ok(Some(status)) => Ok(Some(status.is_ok())), // 성공 여부만 반환
            Ok(None) => Ok(None), // 아직 확인되지 않음
            Err(_) => Ok(None), // 조회 실패
        }
    }

    /// RPC URL 반환
    /// Get RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
}