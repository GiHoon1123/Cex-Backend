use std::sync::Arc;
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use crate::shared::database::{Database, UserRepository};
use crate::domains::auth::models::user::User;
use crate::domains::auth::services::AuthService;
use crate::domains::cex::engine::runtime::HighPerformanceEngine;
use crate::domains::bot::models::BotConfig;

/// 봇 관리자
/// Bot Manager
/// 
/// 역할:
/// - 봇 계정 생성/확인 (서버 시작 시)
/// - 봇 자산 설정 (무한대에 가까운 SOL/USDT 제공)
/// - 봇 주문 생성/취소 관리
/// 
/// 처리 흐름:
/// 1. 서버 시작 시 봇 계정 확인 (없으면 생성)
/// 2. 봇 자산 설정 (1,000,000,000 SOL, 1,000,000,000 USDT)
/// 3. 봇 주문 생성/취소 API 제공
#[derive(Clone)]
pub struct BotManager {
    /// 데이터베이스 연결
    /// Database connection
    db: Database,
    
    /// 체결 엔진
    /// Matching engine
    engine: Arc<tokio::sync::Mutex<HighPerformanceEngine>>,
    
    /// 봇 설정
    /// Bot configuration
    config: BotConfig,
    
    /// 봇 1 (매수 전용) 사용자 정보
    /// Bot 1 (Buy only) user info
    bot1_user: Option<User>,
    
    /// 봇 2 (매도 전용) 사용자 정보
    /// Bot 2 (Sell only) user info
    bot2_user: Option<User>,
}

impl BotManager {
    /// 생성자
    /// Constructor
    /// 
    /// # Arguments
    /// * `db` - 데이터베이스 연결
    /// * `engine` - 체결 엔진
    /// * `config` - 봇 설정
    /// 
    /// # Returns
    /// BotManager 인스턴스
    pub fn new(
        db: Database,
        engine: Arc<tokio::sync::Mutex<HighPerformanceEngine>>,
        config: BotConfig,
    ) -> Self {
        Self {
            db,
            engine,
            config,
            bot1_user: None,
            bot2_user: None,
        }
    }

    /// 봇 계정 초기화
    /// Initialize bot accounts
    /// 
    /// 서버 시작 시 호출됩니다.
    /// - 봇 계정이 없으면 생성
    /// - 봇 자산 설정 (무한대에 가까운 SOL/USDT)
    /// 
    /// # Returns
    /// * `Ok(())` - 초기화 성공
    /// * `Err` - 초기화 실패
    /// 
    /// # 처리 과정
    /// 1. bot1 계정 확인/생성
    /// 2. bot2 계정 확인/생성
    /// 3. bot1 자산 설정
    /// 4. bot2 자산 설정
    /// 봇 계정 확인/생성 및 데이터 삭제 (엔진 시작 전)
    /// Ensure bot accounts and delete previous data (before engine start)
    /// 
    /// 엔진이 필요하지 않은 작업만 수행합니다.
    pub async fn prepare_bots(&mut self) -> Result<()> {
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        // 1. 봇 계정 확인/생성
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        let auth_service = AuthService::new(self.db.clone());
        
        // Bot 1 (매수 전용) 계정 확인/생성
        let bot1 = self.ensure_bot_account(
            &auth_service,
            &self.config.bot1_email,
            &self.config.bot1_password,
        )
        .await
        .context("Failed to ensure bot1 account")?;
        
        // Bot 2 (매도 전용) 계정 확인/생성
        let bot2 = self.ensure_bot_account(
            &auth_service,
            &self.config.bot2_email,
            &self.config.bot2_password,
        )
        .await
        .context("Failed to ensure bot2 account")?;
        
        self.bot1_user = Some(bot1.clone());
        self.bot2_user = Some(bot2.clone());
        
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        // 2. 서버 재시작 시 이전 봇 데이터 모두 삭제
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        self.delete_all_bot_data(bot1.id).await
            .context("Failed to delete bot1 data")?;
        self.delete_all_bot_data(bot2.id).await
            .context("Failed to delete bot2 data")?;
        Ok(())
    }
    
    /// 봇 잔고를 DB에 직접 쓰기 (엔진 시작 전)
    /// Set bot balances in database (before engine start)
    /// 
    /// 엔진이 시작되기 전에 DB에 직접 잔고를 쓰고,
    /// 엔진 시작 시 DB에서 자동으로 로드되도록 합니다.
    pub async fn set_bot_balances_in_db(&self) -> Result<()> {
        use crate::shared::database::repositories::cex::balance_repository::UserBalanceRepository;
        
        let bot1_id = self.bot1_user.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Bot1 not initialized"))?
            .id;
        let bot2_id = self.bot2_user.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Bot2 not initialized"))?
            .id;
        
        // 1,000,000,000 SOL, 1,000,000,000 USDT
        let huge_balance = Decimal::new(1_000_000_000, 0);
        
        use crate::domains::cex::models::balance::UserBalanceUpdate;
        
        let balance_repo = UserBalanceRepository::new(self.db.pool().clone());
        
        // Bot 1 자산 설정 (DB에 직접 쓰기)
        let update1 = UserBalanceUpdate {
            available_delta: Some(huge_balance),
            locked_delta: None,
        };
        balance_repo.update_balance(bot1_id, "SOL", &update1).await
            .context("Failed to set bot1 SOL balance in DB")?;
        balance_repo.update_balance(bot1_id, "USDT", &update1).await
            .context("Failed to set bot1 USDT balance in DB")?;
        
        // Bot 2 자산 설정 (DB에 직접 쓰기)
        balance_repo.update_balance(bot2_id, "SOL", &update1).await
            .context("Failed to set bot2 SOL balance in DB")?;
        balance_repo.update_balance(bot2_id, "USDT", &update1).await
            .context("Failed to set bot2 USDT balance in DB")?;
        
        Ok(())
    }
    
    /// 봇 잔고 설정 (엔진 시작 후 - 더 이상 사용하지 않음)
    /// Set bot balances (after engine start)
    /// 
    /// 엔진이 시작된 후에 호출해야 합니다.
    /// 
    /// 주의: 이제는 사용하지 않습니다. `set_bot_balances_in_db`를 사용하세요.
    #[allow(dead_code)]
    pub async fn initialize_bots(&mut self) -> Result<()> {
        let bot1_id = self.bot1_user.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Bot1 not initialized"))?
            .id;
        let bot2_id = self.bot2_user.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Bot2 not initialized"))?
            .id;
        
        // 1,000,000,000 SOL, 1,000,000,000 USDT
        let huge_balance = Decimal::new(1_000_000_000, 0);
        
        // Bot 1 자산 설정
        self.set_bot_balance(bot1_id, "SOL", huge_balance).await
            .context("Failed to set bot1 SOL balance")?;
        self.set_bot_balance(bot1_id, "USDT", huge_balance).await
            .context("Failed to set bot1 USDT balance")?;
        
        // Bot 2 자산 설정
        self.set_bot_balance(bot2_id, "SOL", huge_balance).await
            .context("Failed to set bot2 SOL balance")?;
        self.set_bot_balance(bot2_id, "USDT", huge_balance).await
            .context("Failed to set bot2 USDT balance")?;
        
        Ok(())
    }
    
    /// 봇 계정의 모든 데이터 삭제 (주문 및 거래)
    /// Delete all bot data (orders and trades)
    /// 
    /// 서버 재시작 시 이전에 생성된 봇 주문과 거래를 완전히 삭제합니다.
    /// 엔진 시작 전에 실행되므로 DB에서 직접 삭제합니다.
    /// 
    /// # 처리 순서
    /// 1. 봇이 참여한 거래 삭제 (foreign key 제약 때문에 먼저)
    /// 2. 봇의 모든 주문 삭제
    async fn delete_all_bot_data(&self, user_id: u64) -> Result<()> {
        // 1. 봇의 주문 ID 목록 조회 (거래 삭제를 위해 필요)
        let order_ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT id FROM orders WHERE user_id = $1
            "#,
        )
        .bind(user_id as i64)
        .fetch_all(self.db.pool())
        .await
        .context("Failed to fetch bot order IDs")?;
        
        let order_count = order_ids.len();
        let mut trade_count = 0u64;
        
        // 2. 봇이 참여한 거래 삭제 (foreign key 제약 때문에 먼저 삭제)
        if !order_ids.is_empty() {
            let deleted_trades = sqlx::query(
                r#"
                DELETE FROM trades
                WHERE buy_order_id = ANY($1) OR sell_order_id = ANY($1)
                RETURNING id
                "#,
            )
            .bind(&order_ids)
            .fetch_all(self.db.pool())
            .await
            .context("Failed to delete bot trades")?;
            
            trade_count = deleted_trades.len() as u64;
        }
        
        // 3. 봇의 모든 주문 삭제
        if order_count > 0 {
            let deleted_orders = sqlx::query(
                r#"
                DELETE FROM orders WHERE user_id = $1
                RETURNING id
                "#,
            )
            .bind(user_id as i64)
            .execute(self.db.pool())
            .await
            .context("Failed to delete bot orders")?;
            
            // 봇 데이터 삭제 완료 (로그 제거 - 정상 동작은 조용히)
        }
        
        Ok(())
    }

    /// 봇 계정 확인/생성
    /// Ensure bot account exists
    /// 
    /// 계정이 있으면 반환, 없으면 생성 후 반환
    /// 
    /// # Arguments
    /// * `auth_service` - 인증 서비스
    /// * `email` - 봇 이메일
    /// * `password` - 봇 비밀번호
    /// 
    /// # Returns
    /// * `Ok(User)` - 봇 사용자 정보
    /// * `Err` - 계정 생성/조회 실패
    async fn ensure_bot_account(
        &self,
        auth_service: &AuthService,
        email: &str,
        password: &str,
    ) -> Result<User> {
        let user_repo = UserRepository::new(self.db.pool().clone());
        
        // 계정이 이미 있는지 확인
        if let Some(user) = user_repo
            .get_user_by_email(email)
            .await
            .context("Failed to check bot account existence")?
        {
            // 계정이 이미 존재함
            return Ok(user);
        }
        
        // 계정이 없으면 생성
        use crate::domains::auth::models::SignupRequest;
        let signup_request = SignupRequest {
            email: email.to_string(),
            password: password.to_string(),
            username: Some(email.to_string()), // username도 email과 동일하게
        };
        
        auth_service
            .signup(signup_request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create bot account: {:?}", e))
    }

    /// 봇 자산 설정
    /// Set bot balance
    /// 
    /// 엔진의 `update_balance`를 사용하여 봇 자산을 설정합니다.
    /// 
    /// # Arguments
    /// * `user_id` - 봇 사용자 ID
    /// * `mint` - 자산 종류 (SOL, USDT 등)
    /// * `amount` - 설정할 잔액
    /// 
    /// # Returns
    /// * `Ok(())` - 자산 설정 성공
    /// * `Err` - 자산 설정 실패
    async fn set_bot_balance(
        &self,
        user_id: u64,
        mint: &str,
        amount: Decimal,
    ) -> Result<()> {
        use crate::domains::cex::engine::Engine;
        let engine_guard = self.engine.lock().await;
        engine_guard
            .update_balance(user_id, mint, amount)
            .await
            .context(format!("Failed to set bot balance: user_id={}, mint={}, amount={}", user_id, mint, amount))?;
        
        Ok(())
    }

    /// 봇 1 (매수 전용) 사용자 ID 가져오기
    /// Get bot 1 (buy only) user ID
    /// 
    /// # Returns
    /// * `Some(u64)` - 봇 1 사용자 ID
    /// * `None` - 봇이 아직 초기화되지 않음
    pub fn bot1_user_id(&self) -> Option<u64> {
        self.bot1_user.as_ref().map(|u| u.id)
    }

    /// 봇 2 (매도 전용) 사용자 ID 가져오기
    /// Get bot 2 (sell only) user ID
    /// 
    /// # Returns
    /// * `Some(u64)` - 봇 2 사용자 ID
    /// * `None` - 봇이 아직 초기화되지 않음
    pub fn bot2_user_id(&self) -> Option<u64> {
        self.bot2_user.as_ref().map(|u| u.id)
    }

    /// 봇 설정 가져오기
    /// Get bot configuration
    pub fn config(&self) -> &BotConfig {
        &self.config
    }

    /// 엔진 참조 가져오기
    /// Get engine reference
    pub fn engine(&self) -> &Arc<tokio::sync::Mutex<HighPerformanceEngine>> {
        &self.engine
    }
}

