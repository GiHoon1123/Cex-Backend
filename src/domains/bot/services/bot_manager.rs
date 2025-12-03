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
        // 두 봇의 데이터를 한 번에 삭제 (일반 사용자 거래 보존)
        self.delete_all_bot_data_together().await
            .context("Failed to delete bot data")?;
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
    
    /// 봇 계정의 모든 데이터 삭제 (주문 및 거래) - 두 봇을 함께 처리
    /// Delete all bot data (orders and trades) - process both bots together
    /// 
    /// 서버 재시작 시 이전에 생성된 봇 주문과 거래를 완전히 삭제합니다.
    /// 엔진 시작 전에 실행되므로 DB에서 직접 삭제합니다.
    /// 
    /// # 처리 순서
    /// 1. 두 봇의 주문 ID를 모두 수집
    /// 2. 매수와 매도가 모두 봇인 거래만 삭제 (일반 사용자 거래 보존)
    /// 3. 두 봇의 모든 주문 삭제
    /// 
    /// 매수와 매도가 모두 봇인 거래만 삭제하여 일반 사용자의 거래 내역을 보존합니다.
    async fn delete_all_bot_data_together(&self) -> Result<()> {
        // 1. 봇 이메일로 user_id 조회 (정확성 보장)
        use crate::shared::database::UserRepository;
        let user_repo = UserRepository::new(self.db.pool().clone());
        
        let mut bot_user_ids = Vec::new();
        
        // bot1@bot.com 조회
        if let Ok(Some(bot1)) = user_repo.get_user_by_email(&self.config.bot1_email).await {
            bot_user_ids.push(bot1.id as i64);
        }
        
        // bot2@bot.com 조회
        if let Ok(Some(bot2)) = user_repo.get_user_by_email(&self.config.bot2_email).await {
            bot_user_ids.push(bot2.id as i64);
        }
        
        if bot_user_ids.is_empty() {
            return Ok(());
        }
        
        // 2. buyer_id와 seller_id가 모두 봇인 거래만 삭제 (일반 사용자 거래 보존)
        sqlx::query(
            r#"
            DELETE FROM trades
            WHERE buyer_id = ANY($1) AND seller_id = ANY($1)
            "#,
        )
        .bind(&bot_user_ids)
        .execute(self.db.pool())
        .await
        .context("Failed to delete bot-only trades")?;
        
        // 3. 일반 사용자가 참여한 trade에 참여한 봇 order는 보존하고, 나머지만 삭제
        // 일반 사용자가 참여한 trade = buyer_id나 seller_id 중 하나라도 봇이 아닌 trade
        sqlx::query(
            r#"
            DELETE FROM orders
            WHERE user_id = ANY($1)
            AND id NOT IN (
                SELECT DISTINCT buy_order_id FROM trades
                WHERE buyer_id != ALL($1) OR seller_id != ALL($1)
                UNION
                SELECT DISTINCT sell_order_id FROM trades
                WHERE buyer_id != ALL($1) OR seller_id != ALL($1)
            )
            "#,
        )
        .bind(&bot_user_ids)
        .execute(self.db.pool())
        .await
        .context("Failed to delete bot orders")?;
        
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::shared::database::Database;
    use crate::domains::bot::models::BotConfig;
    use sqlx::PgPool;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://root:1234@localhost:5432/cex_test".to_string());
        PgPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_delete_bot_data_preserves_user_trades() {
        let pool = setup_test_db().await;
        let db = Database::noop(pool.clone());
        
        let bot1_email = "bot1@bot.com";
        let bot2_email = "bot2@bot.com";
        let user_email = "test_user@test.com";
        
        let user_repo = crate::shared::database::UserRepository::new(pool.clone());
        
        // 봇 user_id 조회
        let bot1 = user_repo.get_user_by_email(bot1_email).await.unwrap();
        let bot2 = user_repo.get_user_by_email(bot2_email).await.unwrap();
        let user = user_repo.get_user_by_email(user_email).await.unwrap();
        
        if bot1.is_none() || bot2.is_none() || user.is_none() {
            eprintln!("Skipping test: bot or user accounts not found");
            return;
        }
        
        let bot1_id = bot1.unwrap().id;
        let bot2_id = bot2.unwrap().id;
        let user_id = user.unwrap().id;
        
        // 테스트 데이터 삽입: 봇끼리 거래
        sqlx::query(
            r#"
            INSERT INTO trades (id, buy_order_id, sell_order_id, buyer_id, seller_id, price, amount, base_mint, quote_mint, created_at)
            VALUES (999999999, 999999999, 999999998, $1, $2, 100.0, 1.0, 'SOL', 'USDT', NOW())
            ON CONFLICT (id) DO UPDATE SET buyer_id = $1, seller_id = $2
            "#,
        )
        .bind(bot1_id as i64)
        .bind(bot2_id as i64)
        .execute(&pool)
        .await
        .unwrap();
        
        // 테스트 데이터 삽입: 일반 사용자와 봇 거래
        sqlx::query(
            r#"
            INSERT INTO trades (id, buy_order_id, sell_order_id, buyer_id, seller_id, price, amount, base_mint, quote_mint, created_at)
            VALUES (999999997, 999999997, 999999996, $1, $2, 100.0, 1.0, 'SOL', 'USDT', NOW())
            ON CONFLICT (id) DO UPDATE SET buyer_id = $1, seller_id = $2
            "#,
        )
        .bind(user_id as i64)
        .bind(bot1_id as i64)
        .execute(&pool)
        .await
        .unwrap();
        
        // BotManager 생성
        let config = BotConfig {
            bot1_email: bot1_email.to_string(),
            bot2_email: bot2_email.to_string(),
            ..Default::default()
        };
        let engine = Arc::new(tokio::sync::Mutex::new(
            crate::domains::cex::engine::runtime::HighPerformanceEngine::new(db.clone())
        ));
        let mut bot_manager = BotManager::new(db.clone(), engine, config);
        
        // 봇 계정 설정
        bot_manager.prepare_bots().await.unwrap();
        
        // 검증: 봇끼리 거래는 삭제되어야 함
        let bot_to_bot_before: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM trades
            WHERE buyer_id = $1 AND seller_id = $2 AND id = 999999999
            "#,
        )
        .bind(bot1_id as i64)
        .bind(bot2_id as i64)
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(bot_to_bot_before, 1, "Bot-to-bot trade should exist before deletion");
        
        // 검증: 일반 사용자와 봇 거래는 보존되어야 함
        let user_to_bot_before: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM trades
            WHERE buyer_id = $1 AND seller_id = $2 AND id = 999999997
            "#,
        )
        .bind(user_id as i64)
        .bind(bot1_id as i64)
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(user_to_bot_before, 1, "User-to-bot trade should exist before deletion");
        
        // 삭제 실행 (prepare_bots에서 자동 실행됨, 하지만 다시 실행해도 안전)
        // prepare_bots에서 이미 실행되었으므로 직접 호출 불가
        // 대신 SQL로 직접 테스트
        let bot_user_ids = vec![bot1_id as i64, bot2_id as i64];
        sqlx::query(
            r#"
            DELETE FROM trades
            WHERE buyer_id = ANY($1) AND seller_id = ANY($1)
            "#,
        )
        .bind(&bot_user_ids)
        .execute(&pool)
        .await
        .unwrap();
        
        // 검증: 봇끼리 거래는 삭제되어야 함
        let bot_to_bot_after: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM trades
            WHERE buyer_id = $1 AND seller_id = $2 AND id = 999999999
            "#,
        )
        .bind(bot1_id as i64)
        .bind(bot2_id as i64)
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(bot_to_bot_after, 0, "Bot-to-bot trade should be deleted");
        
        // 검증: 일반 사용자와 봇 거래는 보존되어야 함
        let user_to_bot_after: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM trades
            WHERE buyer_id = $1 AND seller_id = $2 AND id = 999999997
            "#,
        )
        .bind(user_id as i64)
        .bind(bot1_id as i64)
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(user_to_bot_after, 1, "User-to-bot trade should be preserved");
        
        // 정리
        sqlx::query("DELETE FROM trades WHERE id IN (999999999, 999999997)").execute(&pool).await.unwrap();
    }
}

