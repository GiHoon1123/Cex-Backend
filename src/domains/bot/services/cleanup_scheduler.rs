use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Context, Result};
use tokio::time::{interval, Duration};
use crate::shared::database::{Database, UserRepository};

/// 봇 데이터 정리 스케줄러
/// Bot Data Cleanup Scheduler
/// 
/// 역할:
/// - 3분마다 봇의 주문과 체결내역을 자동으로 삭제
/// - API로 활성화/비활성화 제어 가능
/// 
/// 처리 흐름:
/// 1. 스케줄러 시작 시 백그라운드 태스크 실행
/// 2. 3분마다 봇 데이터 삭제 실행
/// 3. 활성화 상태에 따라 실행 여부 결정
#[derive(Clone)]
pub struct BotCleanupScheduler {
    /// 데이터베이스 연결
    db: Database,
    
    /// 봇 1 사용자 ID
    bot1_user_id: Option<u64>,
    
    /// 봇 2 사용자 ID
    bot2_user_id: Option<u64>,
    
    /// 스케줄러 활성화 상태
    enabled: Arc<AtomicBool>,
}

impl BotCleanupScheduler {
    /// 새 스케줄러 생성
    /// Create new scheduler
    pub fn new(
        db: Database,
        bot1_user_id: Option<u64>,
        bot2_user_id: Option<u64>,
    ) -> Self {
        Self {
            db,
            bot1_user_id,
            bot2_user_id,
            enabled: Arc::new(AtomicBool::new(false)), // 기본값: 비활성화
        }
    }
    
    /// 스케줄러 시작
    /// Start scheduler
    /// 
    /// 백그라운드 태스크를 시작하여 3분마다 봇 데이터를 정리합니다.
    pub fn start(&self) {
        let db = self.db.clone();
        let bot1_user_id = self.bot1_user_id;
        let bot2_user_id = self.bot2_user_id;
        let enabled = self.enabled.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(180)); // 3분 = 180초
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                
                // 활성화 상태 확인
                if !enabled.load(Ordering::Relaxed) {
                    continue;
                }
                
                // 봇 데이터 삭제 (두 봇의 주문 ID를 모두 수집하여 매수/매도가 모두 봇인 거래만 삭제)
                if let Err(e) = Self::delete_bot_data_internal(&db, bot1_user_id, bot2_user_id).await {
                    eprintln!("[Bot Cleanup Scheduler] Failed to delete bot data: {}", e);
                }
            }
        });
    }
    
    /// 봇 데이터 삭제 (내부 메서드)
    /// Delete bot data (internal method)
    /// 
    /// 매수와 매도가 모두 봇인 거래만 삭제하여 일반 사용자의 거래 내역을 보존합니다.
    async fn delete_bot_data_internal(
        db: &Database,
        _bot1_user_id: Option<u64>,
        _bot2_user_id: Option<u64>,
    ) -> Result<()> {
        // 1. 봇 이메일로 user_id 조회 (정확성 보장)
        let user_repo = UserRepository::new(db.pool().clone());
        
        let mut bot_user_ids = Vec::new();
        
        // bot1@bot.com 조회
        if let Ok(Some(bot1)) = user_repo.get_user_by_email("bot1@bot.com").await {
            bot_user_ids.push(bot1.id as i64);
        }
        
        // bot2@bot.com 조회
        if let Ok(Some(bot2)) = user_repo.get_user_by_email("bot2@bot.com").await {
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
        .execute(db.pool())
        .await
        .context("Failed to delete bot-only trades")?;
        
        // 3. 일반 사용자가 참여한 trade에 참여한 봇 order는 보존하고, 나머지만 삭제
        // 봇끼리만 거래한 trade에 참여한 order만 삭제
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
        .execute(db.pool())
        .await
        .context("Failed to delete bot orders")?;
        
        Ok(())
    }
    
    /// 스케줄러 활성화
    /// Enable scheduler
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }
    
    /// 스케줄러 비활성화
    /// Disable scheduler
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
    
    /// 스케줄러 상태 조회
    /// Get scheduler status
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

