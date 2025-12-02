use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Context, Result};
use tokio::time::{interval, Duration};
use crate::shared::database::Database;

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
                
                // 봇 1 데이터 삭제
                if let Some(user_id) = bot1_user_id {
                    if let Err(e) = Self::delete_bot_data_internal(&db, user_id).await {
                        eprintln!("[Bot Cleanup Scheduler] Failed to delete bot1 data: {}", e);
                    }
                }
                
                // 봇 2 데이터 삭제
                if let Some(user_id) = bot2_user_id {
                    if let Err(e) = Self::delete_bot_data_internal(&db, user_id).await {
                        eprintln!("[Bot Cleanup Scheduler] Failed to delete bot2 data: {}", e);
                    }
                }
            }
        });
    }
    
    /// 봇 데이터 삭제 (내부 메서드)
    /// Delete bot data (internal method)
    async fn delete_bot_data_internal(db: &Database, user_id: u64) -> Result<()> {
        // 1. 봇의 주문 ID 목록 조회 (거래 삭제를 위해 필요)
        let order_ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT id FROM orders WHERE user_id = $1
            "#,
        )
        .bind(user_id as i64)
        .fetch_all(db.pool())
        .await
        .context("Failed to fetch bot order IDs")?;
        
        // 2. 봇이 참여한 거래 삭제 (foreign key 제약 때문에 먼저 삭제)
        if !order_ids.is_empty() {
            sqlx::query(
                r#"
                DELETE FROM trades
                WHERE buy_order_id = ANY($1) OR sell_order_id = ANY($1)
                "#,
            )
            .bind(&order_ids)
            .execute(db.pool())
            .await
            .context("Failed to delete bot trades")?;
        }
        
        // 3. 봇의 모든 주문 삭제
        if !order_ids.is_empty() {
            sqlx::query(
                r#"
                DELETE FROM orders WHERE user_id = $1
                "#,
            )
            .bind(user_id as i64)
            .execute(db.pool())
            .await
            .context("Failed to delete bot orders")?;
        }
        
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

