use std::sync::Arc;
use crate::shared::database::{Database, UserBalanceRepository};
use crate::domains::cex::models::balance::{UserBalance, UserBalanceCreate};
use crate::domains::cex::engine::{Engine, runtime::HighPerformanceEngine};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use chrono::Utc;

/// 거래소 잔고 서비스
/// Exchange Balance Service
/// 
/// 역할:
/// - 사용자의 거래소 잔고 조회 및 관리
/// - 잔고 초기화 (입금 시 사용)
/// - 잔고 조회 (API에서 사용)
/// 
/// 변경사항:
/// - 잔고 조회는 엔진 메모리에서 실시간 조회 (실시간 정확성 보장)
/// - DB는 백업/복구용 및 메타데이터(id, created_at, updated_at) 저장용
#[derive(Clone)]
pub struct BalanceService {
    db: Database,
    engine: Arc<tokio::sync::Mutex<HighPerformanceEngine>>,
}

impl BalanceService {
    /// 생성자
    /// Constructor
    /// 
    /// # Arguments
    /// * `db` - 데이터베이스 연결
    /// * `engine` - 체결 엔진 (메모리 잔고 조회용)
    /// 
    /// # Returns
    /// BalanceService 인스턴스
    pub fn new(db: Database, engine: Arc<tokio::sync::Mutex<HighPerformanceEngine>>) -> Self {
        Self { db, engine }
    }

    /// 사용자의 모든 잔고 조회
    /// Get all balances for a user
    /// 
    /// DB에서 자산 목록을 가져온 후, 각 자산의 잔고를 엔진 메모리에서 실시간 조회합니다.
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// 
    /// # Returns
    /// * `Ok(Vec<UserBalance>)` - 사용자의 모든 자산 잔고 목록
    /// * `Err` - 조회 오류 시
    pub async fn get_all_balances(&self, user_id: u64) -> Result<Vec<UserBalance>> {
        // 1. DB에서 사용자의 모든 자산 목록 조회 (메타데이터 포함)
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());
        let db_balances = balance_repo
            .get_all_by_user(user_id)
            .await
            .context("Failed to fetch user balance list from database")?;

        // 2. 각 자산마다 엔진 메모리에서 실시간 잔고 조회
        let mut balances = Vec::new();
        let engine_guard = self.engine.lock().await;

        for db_balance in db_balances {
            // 엔진 메모리에서 실시간 잔고 조회
            let (available, locked) = engine_guard
                .get_balance(user_id, &db_balance.mint_address)
                .await
                .unwrap_or((Decimal::ZERO, Decimal::ZERO)); // 실패 시 0으로 처리

            // 엔진 메모리 값으로 업데이트된 UserBalance 생성
            balances.push(UserBalance {
                id: db_balance.id,
                user_id: db_balance.user_id,
                mint_address: db_balance.mint_address,
                available, // 엔진 메모리 값 사용
                locked,    // 엔진 메모리 값 사용
                created_at: db_balance.created_at,
                updated_at: Utc::now(), // 실시간 조회이므로 현재 시간으로 업데이트
            });
        }

        Ok(balances)
    }

    /// 특정 자산의 잔고 조회
    /// Get balance for a specific asset
    /// 
    /// 엔진 메모리에서 실시간 잔고를 조회합니다.
    /// 메타데이터(id, created_at, updated_at)는 DB에서 가져옵니다.
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `mint_address` - 자산 식별자 (예: "SOL", "USDT")
    /// 
    /// # Returns
    /// * `Ok(Some(UserBalance))` - 잔고가 존재하는 경우
    /// * `Ok(None)` - 잔고가 없는 경우 (자산을 보유하지 않음)
    /// * `Err` - 조회 오류 시
    pub async fn get_balance(
        &self,
        user_id: u64,
        mint_address: &str,
    ) -> Result<Option<UserBalance>> {
        // 1. 엔진 메모리에서 실시간 잔고 조회
        let (available, locked) = {
            let engine_guard = self.engine.lock().await;
            engine_guard
                .get_balance(user_id, mint_address)
                .await
                .context("Failed to get balance from engine")?
        };

        // 2. DB에서 메타데이터 조회 (id, created_at, updated_at)
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());
        let db_balance = balance_repo
            .get_by_user_and_mint(user_id, mint_address)
            .await
            .context(format!(
                "Failed to fetch balance metadata from database for user {} and asset {}",
                user_id, mint_address
            ))?;

        // 3. 엔진 메모리 잔고가 0이고 DB에도 없으면 None 반환
        if available == Decimal::ZERO && locked == Decimal::ZERO && db_balance.is_none() {
            return Ok(None);
        }

        // 4. UserBalance 생성 (엔진 메모리 값 사용, 메타데이터는 DB 값 사용)
        let balance = if let Some(db_bal) = db_balance {
            // DB에 레코드가 있으면 엔진 메모리 값으로 업데이트
            Some(UserBalance {
                id: db_bal.id,
                user_id: db_bal.user_id,
                mint_address: db_bal.mint_address,
                available, // 엔진 메모리 값 사용
                locked,    // 엔진 메모리 값 사용
                created_at: db_bal.created_at,
                updated_at: Utc::now(), // 실시간 조회이므로 현재 시간으로 업데이트
            })
        } else {
            // DB에 레코드가 없으면 엔진 메모리 값만 사용 (임시 레코드 생성)
            // 실제로는 DB에 레코드가 있어야 하지만, 엔진 메모리에만 있는 경우를 대비
            if available > Decimal::ZERO || locked > Decimal::ZERO {
                // 엔진 메모리에 잔고가 있으면 DB에 레코드 생성
                let balance_create = UserBalanceCreate {
                    user_id,
                    mint_address: mint_address.to_string(),
                    available,
                    locked,
                };
                let new_balance = balance_repo
                    .create_or_get(&balance_create)
                    .await
                    .context("Failed to create balance record in database")?;
                Some(new_balance)
            } else {
                None
            }
        };

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