use crate::shared::database::{Database, UserBalanceRepository};
use crate::domains::cex::models::balance::{UserBalance, UserBalanceCreate};
use anyhow::{Context, Result};
use rust_decimal::Decimal;

/// 거래소 잔고 서비스
/// Exchange Balance Service
/// 
/// 역할:
/// - 사용자의 거래소 잔고 조회 및 관리
/// - 잔고 초기화 (입금 시 사용)
/// - 잔고 조회 (API에서 사용)
/// 
/// 주의:
/// - 이 서비스는 엔진과 독립적으로 동작 (엔진 구현 전에도 사용 가능)
/// - 엔진은 내부적으로 BalanceCache를 사용하여 메모리 기반으로 처리
/// - 이 서비스는 DB 기반의 영구 잔고 관리 및 API 조회용
#[derive(Clone)]
pub struct BalanceService {
    db: Database,
}

impl BalanceService {
    /// 생성자
    /// Constructor
    /// 
    /// # Arguments
    /// * `db` - 데이터베이스 연결
    /// 
    /// # Returns
    /// BalanceService 인스턴스
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 사용자의 모든 잔고 조회
    /// Get all balances for a user
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// 
    /// # Returns
    /// * `Ok(Vec<UserBalance>)` - 사용자의 모든 자산 잔고 목록
    /// * `Err` - 데이터베이스 오류 시
    pub async fn get_all_balances(&self, user_id: u64) -> Result<Vec<UserBalance>> {
        // Repository 생성 (Service 내부에서 필요할 때마다 생성)
        // Create repository (created inside service as needed)
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());

        // DB에서 사용자의 모든 잔고 조회
        // Query all balances for user from database
        let balances = balance_repo
            .get_all_by_user(user_id)
            .await
            .context("Failed to fetch user balances from database")?;

        Ok(balances)
    }

    /// 특정 자산의 잔고 조회
    /// Get balance for a specific asset
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `mint_address` - 자산 식별자 (예: "SOL", "USDT")
    /// 
    /// # Returns
    /// * `Ok(Some(UserBalance))` - 잔고가 존재하는 경우
    /// * `Ok(None)` - 잔고가 없는 경우 (자산을 보유하지 않음)
    /// * `Err` - 데이터베이스 오류 시
    pub async fn get_balance(
        &self,
        user_id: u64,
        mint_address: &str,
    ) -> Result<Option<UserBalance>> {
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());

        // DB에서 특정 자산 잔고 조회
        // Query specific asset balance from database
        let balance = balance_repo
            .get_by_user_and_mint(user_id, mint_address)
            .await
            .context(format!(
                "Failed to fetch balance for user {} and asset {}",
                user_id, mint_address
            ))?;

        Ok(balance)
    }

    /// 잔고 초기화 또는 생성
    /// Initialize or create balance for user
    /// 
    /// 주의: 이미 잔고가 있으면 업데이트하지 않고 기존 잔고 반환
    /// Note: If balance already exists, returns existing balance without updating
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `mint_address` - 자산 식별자
    /// * `initial_available` - 초기 사용 가능 잔고 (기본값: 0)
    /// 
    /// # Returns
    /// * `Ok(UserBalance)` - 생성 또는 조회된 잔고
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # Use Cases
    /// - 입금 시 잔고 레코드 초기화
    /// - 새로운 자산 거래 시작 시 잔고 생성
    pub async fn init_balance(
        &self,
        user_id: u64,
        mint_address: &str,
        initial_available: Decimal,
    ) -> Result<UserBalance> {
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());

        // 잔고 생성 또는 기존 잔고 조회
        // create_or_get: 이미 있으면 기존 것 반환, 없으면 새로 생성
        // create_or_get: returns existing if exists, creates new if not
        let balance_create = UserBalanceCreate {
            user_id,
            mint_address: mint_address.to_string(),
            available: initial_available,
            locked: Decimal::ZERO, // 초기에는 잠긴 잔고 없음
        };

        let balance = balance_repo
            .create_or_get(&balance_create)
            .await
            .context(format!(
                "Failed to initialize balance for user {} and asset {}",
                user_id, mint_address
            ))?;

        Ok(balance)
    }

    /// 잔고 충분 여부 확인
    /// Check if user has sufficient balance
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `mint_address` - 자산 식별자
    /// * `required` - 필요한 수량
    /// 
    /// # Returns
    /// * `Ok(true)` - 잔고가 충분함
    /// * `Ok(false)` - 잔고가 부족함 또는 잔고가 없음
    /// * `Err` - 데이터베이스 오류 시
    pub async fn check_sufficient_balance(
        &self,
        user_id: u64,
        mint_address: &str,
        required: Decimal,
    ) -> Result<bool> {
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());

        // Repository에서 충분 여부 확인
        // Check sufficiency from repository
        let sufficient = balance_repo
            .check_sufficient_balance(user_id, mint_address, required)
            .await
            .context(format!(
                "Failed to check balance sufficiency for user {} and asset {}",
                user_id, mint_address
            ))?;

        Ok(sufficient)
    }
}