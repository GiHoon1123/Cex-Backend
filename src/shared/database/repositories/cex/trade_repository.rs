use sqlx::{PgPool, Row};
use anyhow::{Context, Result};
use chrono::Utc;
use rust_decimal::Decimal;
use crate::domains::cex::models::trade::{Trade, TradeCreate};

pub struct TradeRepository {
    pool: PgPool,
}

impl TradeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 체결 내역 생성
    /// Create trade
    pub async fn create(&self, trade_create: &TradeCreate) -> Result<Trade> {
        let row = sqlx::query(
            r#"
            INSERT INTO trades (
                buy_order_id, sell_order_id, base_mint, quote_mint,
                price, amount, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, buy_order_id, sell_order_id, base_mint, quote_mint,
                      price, amount, created_at
            "#,
        )
        .bind(trade_create.buy_order_id as i64)
        .bind(trade_create.sell_order_id as i64)
        .bind(&trade_create.base_mint)
        .bind(&trade_create.quote_mint)
        .bind(&trade_create.price)
        .bind(&trade_create.amount)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create trade")?;

        Ok(self.row_to_trade(&row))
    }

    /// 체결 내역 ID로 조회
    /// Get trade by ID
    pub async fn get_by_id(&self, trade_id: u64) -> Result<Option<Trade>> {
        let row = sqlx::query(
            r#"
            SELECT id, buy_order_id, sell_order_id, base_mint, quote_mint,
                   price, amount, created_at
            FROM trades
            WHERE id = $1
            "#,
        )
        .bind(trade_id as i64)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch trade by id")?;

        Ok(row.map(|r| self.row_to_trade(&r)))
    }

    /// 거래쌍별 체결 내역 조회 (최신순)
    /// Get trades by trading pair (sorted by time, newest first)
    pub async fn get_by_pair(
        &self,
        base_mint: &str,
        quote_mint: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Trade>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, buy_order_id, sell_order_id, base_mint, quote_mint,
                   price, amount, created_at
            FROM trades
            WHERE base_mint = $1 AND quote_mint = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(base_mint)
        .bind(quote_mint)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch trades by pair")?;

        Ok(rows.iter().map(|r| self.row_to_trade(r)).collect())
    }

    /// 매수 주문 ID로 체결 내역 조회
    /// Get trades by buy order ID
    pub async fn get_by_buy_order(
        &self,
        buy_order_id: u64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Trade>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, buy_order_id, sell_order_id, base_mint, quote_mint,
                   price, amount, created_at
            FROM trades
            WHERE buy_order_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(buy_order_id as i64)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch trades by buy order")?;

        Ok(rows.iter().map(|r| self.row_to_trade(r)).collect())
    }

    /// 매도 주문 ID로 체결 내역 조회
    /// Get trades by sell order ID
    pub async fn get_by_sell_order(
        &self,
        sell_order_id: u64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Trade>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, buy_order_id, sell_order_id, base_mint, quote_mint,
                   price, amount, created_at
            FROM trades
            WHERE sell_order_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(sell_order_id as i64)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch trades by sell order")?;

        Ok(rows.iter().map(|r| self.row_to_trade(r)).collect())
    }

    /// 최신 체결 내역 조회 (모든 거래쌍)
    /// Get recent trades (all trading pairs)
    pub async fn get_recent_trades(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<Trade>> {
        let limit = limit.unwrap_or(100);

        let rows = sqlx::query(
            r#"
            SELECT id, buy_order_id, sell_order_id, base_mint, quote_mint,
                   price, amount, created_at
            FROM trades
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch recent trades")?;

        Ok(rows.iter().map(|r| self.row_to_trade(r)).collect())
    }

    /// 사용자별 거래 내역 조회
    /// Get trades by user ID
    /// 
    /// 사용자가 매수한 거래(buy_order의 user_id) 또는 매도한 거래(sell_order의 user_id)를 모두 조회
    /// Returns all trades where user was either buyer or seller
    pub async fn get_by_user(
        &self,
        user_id: u64,
        mint: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Trade>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let query = if let Some(mint) = mint {
            sqlx::query(
                r#"
                SELECT t.id, t.buy_order_id, t.sell_order_id, t.base_mint, t.quote_mint,
                       t.price, t.amount, t.created_at
                FROM trades t
                INNER JOIN orders buy_order ON t.buy_order_id = buy_order.id
                INNER JOIN orders sell_order ON t.sell_order_id = sell_order.id
                WHERE (buy_order.user_id = $1 OR sell_order.user_id = $1)
                  AND t.base_mint = $2
                ORDER BY t.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(user_id as i64)
            .bind(mint)
            .bind(limit)
            .bind(offset)
        } else {
            sqlx::query(
                r#"
                SELECT t.id, t.buy_order_id, t.sell_order_id, t.base_mint, t.quote_mint,
                       t.price, t.amount, t.created_at
                FROM trades t
                INNER JOIN orders buy_order ON t.buy_order_id = buy_order.id
                INNER JOIN orders sell_order ON t.sell_order_id = sell_order.id
                WHERE buy_order.user_id = $1 OR sell_order.user_id = $1
                ORDER BY t.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id as i64)
            .bind(limit)
            .bind(offset)
        };

        let rows = query
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch trades by user")?;

        Ok(rows.iter().map(|r| self.row_to_trade(r)).collect())
    }

    /// 사용자의 특정 자산 매수 거래 통계 (평균 매수가 계산용)
    /// Get buy trade statistics for user's specific asset (for calculating average entry price)
    /// 
    /// Returns:
    /// - total_bought_amount: 총 매수 수량
    /// - total_bought_cost: 총 매수 금액 (price × amount의 합)
    /// - average_entry_price: 평균 매수가 (total_bought_cost / total_bought_amount)
    pub async fn get_buy_statistics(
        &self,
        user_id: u64,
        base_mint: &str,
    ) -> Result<Option<(Decimal, Decimal, Decimal)>> {
        let row = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(t.amount), 0) as total_bought_amount,
                COALESCE(SUM(t.price * t.amount), 0) as total_bought_cost
            FROM trades t
            INNER JOIN orders o ON t.buy_order_id = o.id
            WHERE o.user_id = $1 AND t.base_mint = $2
            "#,
        )
        .bind(user_id as i64)
        .bind(base_mint)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch buy statistics")?;

        if let Some(row) = row {
            let total_bought_amount: Decimal = row.get("total_bought_amount");
            let total_bought_cost: Decimal = row.get("total_bought_cost");

            if total_bought_amount.is_zero() {
                return Ok(None);
            }

            let average_entry_price = total_bought_cost / total_bought_amount;
            Ok(Some((total_bought_amount, total_bought_cost, average_entry_price)))
        } else {
            Ok(None)
        }
    }

    /// 사용자의 특정 자산 매도 거래 통계 (실현 손익 계산용)
    /// Get sell trade statistics for user's specific asset (for calculating realized P&L)
    /// 
    /// Returns:
    /// - total_sell_trades: 총 매도 횟수
    /// - total_sold_amount: 총 매도 수량
    /// - total_sold_value: 총 매도 금액 (price × amount의 합)
    pub async fn get_sell_statistics(
        &self,
        user_id: u64,
        base_mint: &str,
    ) -> Result<(u64, Decimal, Decimal)> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*)::BIGINT as total_sell_trades,
                COALESCE(SUM(t.amount), 0) as total_sold_amount,
                COALESCE(SUM(t.price * t.amount), 0) as total_sold_value
            FROM trades t
            INNER JOIN orders o ON t.sell_order_id = o.id
            WHERE o.user_id = $1 AND t.base_mint = $2
            "#,
        )
        .bind(user_id as i64)
        .bind(base_mint)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch sell statistics")?;

        let total_sell_trades = row.get::<i64, _>("total_sell_trades") as u64;
        let total_sold_amount: Decimal = row.get("total_sold_amount");
        let total_sold_value: Decimal = row.get("total_sold_value");

        Ok((total_sell_trades, total_sold_amount, total_sold_value))
    }

    /// 사용자의 특정 자산 매수 거래 횟수
    /// Get total number of buy trades for user's specific asset
    pub async fn get_buy_trade_count(
        &self,
        user_id: u64,
        base_mint: &str,
    ) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT t.id)::BIGINT as count
            FROM trades t
            INNER JOIN orders o ON t.buy_order_id = o.id
            WHERE o.user_id = $1 AND t.base_mint = $2
            "#,
        )
        .bind(user_id as i64)
        .bind(base_mint)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch buy trade count")?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// 특정 자산의 최근 체결가 조회 (현재 시장 가격 추정용)
    /// Get latest trade price for specific asset (for estimating current market price)
    pub async fn get_latest_price(
        &self,
        base_mint: &str,
    ) -> Result<Option<Decimal>> {
        let row = sqlx::query(
            r#"
            SELECT price
            FROM trades
            WHERE base_mint = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(base_mint)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch latest price")?;

        Ok(row.map(|r| r.get("price")))
    }

    /// Row를 Trade로 변환하는 헬퍼 메서드
    /// Helper method to convert Row to Trade
    fn row_to_trade(&self, row: &sqlx::postgres::PgRow) -> Trade {
        Trade {
            id: row.get::<i64, _>("id") as u64,
            buy_order_id: row.get::<i64, _>("buy_order_id") as u64,
            sell_order_id: row.get::<i64, _>("sell_order_id") as u64,
            base_mint: row.get("base_mint"),
            quote_mint: row.get("quote_mint"),
            price: row.get("price"),
            amount: row.get("amount"),
            created_at: row.get("created_at"),
        }
    }
}

