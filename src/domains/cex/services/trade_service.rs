use crate::shared::database::{Database, TradeRepository, OrderRepository};
use crate::domains::cex::models::trade::Trade;
use anyhow::{Context, Result};
use rust_decimal::Decimal;

/// 체결 내역 서비스
/// Trade Service
/// 
/// 역할:
/// - 체결 내역 조회 비즈니스 로직 담당
/// - 거래쌍별 체결 내역 조회
/// - 사용자별 체결 내역 조회
/// 
/// 특징:
/// - 읽기 전용 서비스 (조회만 담당)
/// - 체결 생성은 Engine이 담당 (이 서비스는 조회만)
/// 
/// 사용 예시:
/// - 거래소 메인 페이지의 "최근 거래" 표시
/// - 사용자 마이페이지의 "내 거래 내역" 표시
/// - 차트 데이터 생성 (가격 이력)
/// 
/// # Examples
/// ```
/// let service = TradeService::new(db);
/// 
/// // 특정 거래쌍의 최근 거래
/// let trades = service.get_trades("SOL", "USDT", Some(50)).await?;
/// 
/// // 내 거래 내역
/// let my_trades = service.get_my_trades(user_id, None, None).await?;
/// ```
#[derive(Clone)]
pub struct TradeService {
    /// 데이터베이스 연결
    /// Database connection
    db: Database,
}

impl TradeService {
    /// 생성자
    /// Constructor
    /// 
    /// # Arguments
    /// * `db` - 데이터베이스 연결
    /// 
    /// # Returns
    /// TradeService 인스턴스
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 거래쌍별 체결 내역 조회
    /// Get trades for a trading pair
    /// 
    /// 특정 거래쌍(예: SOL/USDT)의 최근 체결 내역을 조회합니다.
    /// 
    /// # Arguments
    /// * `base_mint` - 기준 자산 (예: "SOL")
    /// * `quote_mint` - 기준 통화 (예: "USDT")
    /// * `limit` - 최대 조회 개수 (기본: 50, 최대: 1000)
    /// 
    /// # Returns
    /// * `Ok(Vec<Trade>)` - 체결 내역 목록 (최신순)
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # 용도
    /// - 거래소 메인 페이지의 "최근 거래" 표시
    /// - 차트 데이터 생성 (가격 이력)
    /// - 시장 활동 모니터링
    /// 
    /// # Examples
    /// ```
    /// // SOL/USDT 최근 거래 50건
    /// let trades = service.get_trades("SOL", "USDT", Some(50)).await?;
    /// 
    /// for trade in trades {
    ///     println!("{} SOL @ {} USDT", trade.amount, trade.price);
    /// }
    /// ```
    pub async fn get_trades(
        &self,
        base_mint: &str,
        quote_mint: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Trade>> {
        let trade_repo = TradeRepository::new(self.db.pool().clone());

        // 제한 설정: 기본 50, 최대 1000
        let limit = limit.unwrap_or(50).min(1000);

        // DB에서 거래쌍별 체결 내역 조회 (최신순)
        let trades = trade_repo
            .get_by_pair(base_mint, quote_mint, Some(limit), None)
            .await
            .context(format!(
                "Failed to fetch trades for {}/{}",
                base_mint, quote_mint
            ))?;

        Ok(trades)
    }

    /// 사용자의 체결 내역 조회
    /// Get trades for a user
    /// 
    /// 특정 사용자가 참여한 모든 체결 내역을 조회합니다.
    /// (사용자가 매수자 또는 매도자로 참여한 거래)
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `limit` - 최대 조회 개수
    /// * `offset` - 페이지네이션 오프셋
    /// 
    /// # Returns
    /// * `Ok(Vec<Trade>)` - 체결 내역 목록 (최신순)
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # 처리 과정
    /// 1. 사용자의 모든 주문 조회
    /// 2. 해당 주문들이 참여한 체결 내역 조회
    /// 3. 최신순으로 반환
    /// 
    /// # 용도
    /// - 사용자 마이페이지의 "내 거래 내역" 표시
    /// - 수익/손실 계산
    /// - 거래 통계 생성
    /// 
    /// # Examples
    /// ```
    /// // 내 최근 거래 100건
    /// let my_trades = service.get_my_trades(user_id, Some(100), None).await?;
    /// 
    /// println!("총 {} 건의 거래", my_trades.len());
    /// ```
    pub async fn get_my_trades(
        &self,
        user_id: u64,
        mint: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Trade>> {
        // TradeRepository의 get_by_user 메서드 사용 (효율적)
        // Uses TradeRepository's get_by_user method (efficient)
        let trade_repo = TradeRepository::new(self.db.pool().clone());
        
        let trades = trade_repo
            .get_by_user(user_id, mint, limit, offset)
            .await
            .context("Failed to fetch user trades")?;
        
        Ok(trades)
    }

    /// 특정 주문의 체결 내역 조회
    /// Get trades for a specific order
    /// 
    /// 하나의 주문이 여러 번 체결될 수 있으므로,
    /// 해당 주문의 모든 체결 내역을 조회합니다.
    /// 
    /// # Arguments
    /// * `order_id` - 주문 ID
    /// 
    /// # Returns
    /// * `Ok(Vec<Trade>)` - 해당 주문의 체결 내역 목록
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # 용도
    /// - 주문 상세 페이지에서 "이 주문의 체결 내역" 표시
    /// - 부분 체결 확인
    /// 
    /// # Examples
    /// ```
    /// // 주문 123의 체결 내역
    /// let trades = service.get_trades_for_order(123).await?;
    /// 
    /// for trade in trades {
    ///     println!("체결: {} @ {}", trade.amount, trade.price);
    /// }
    /// ```
    pub async fn get_trades_for_order(&self, order_id: u64) -> Result<Vec<Trade>> {
        let trade_repo = TradeRepository::new(self.db.pool().clone());

        // 매수 주문으로 참여한 체결
        let buy_trades = trade_repo
            .get_by_buy_order(order_id, None, None)
            .await
            .context("Failed to fetch buy trades")?;

        // 매도 주문으로 참여한 체결
        let sell_trades = trade_repo
            .get_by_sell_order(order_id, None, None)
            .await
            .context("Failed to fetch sell trades")?;

        // 두 목록 합치기
        let mut all_trades = buy_trades;
        all_trades.extend(sell_trades);

        // 시간순 정렬 (최신순)
        all_trades.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(all_trades)
    }

    /// 최근 체결 가격 조회
    /// Get latest trade price
    /// 
    /// 특정 거래쌍의 가장 최근 체결 가격을 조회합니다.
    /// 
    /// # Arguments
    /// * `base_mint` - 기준 자산
    /// * `quote_mint` - 기준 통화
    /// 
    /// # Returns
    /// * `Ok(Some(price))` - 최근 체결 가격
    /// * `Ok(None)` - 체결 내역 없음
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # 용도
    /// - 현재가 표시 (차트)
    /// - 시장가 주문 시 참고 가격
    /// 
    /// # Examples
    /// ```
    /// if let Some(price) = service.get_latest_price("SOL", "USDT").await? {
    ///     println!("현재가: {} USDT", price);
    /// } else {
    ///     println!("거래 내역 없음");
    /// }
    /// ```
    pub async fn get_latest_price(
        &self,
        base_mint: &str,
        quote_mint: &str,
    ) -> Result<Option<rust_decimal::Decimal>> {
        let trade_repo = TradeRepository::new(self.db.pool().clone());

        // 최근 체결 1건만 조회
        let trades = trade_repo
            .get_by_pair(base_mint, quote_mint, Some(1), None)
            .await
            .context("Failed to fetch latest trade")?;

        // 가격 반환
        Ok(trades.first().map(|t| t.price))
    }

    /// 24시간 거래량 조회
    /// Get 24-hour trading volume
    /// 
    /// 특정 거래쌍의 최근 24시간 거래량을 조회합니다.
    /// 
    /// # Arguments
    /// * `base_mint` - 기준 자산
    /// * `quote_mint` - 기준 통화
    /// 
    /// # Returns
    /// * `Ok((base_volume, quote_volume))` - (자산 거래량, USDT 거래량)
    /// * `Err` - 데이터베이스 오류 시
    /// 
    /// # 용도
    /// - 거래소 메인 페이지 통계
    /// - 인기 거래쌍 표시
    /// 
    /// # Examples
    /// ```
    /// let (volume_sol, volume_usdt) = service.get_24h_volume("SOL", "USDT").await?;
    /// println!("24h 거래량: {} SOL, {} USDT", volume_sol, volume_usdt);
    /// ```
    pub async fn get_24h_volume(
        &self,
        base_mint: &str,
        quote_mint: &str,
    ) -> Result<(rust_decimal::Decimal, rust_decimal::Decimal)> {
        let trade_repo = TradeRepository::new(self.db.pool().clone());

        // TODO: TradeRepository에 get_24h_volume 메서드 추가 필요
        // 일단 임시로 0 반환
        Ok((Decimal::ZERO, Decimal::ZERO))
        
        // 나중에 구현:
        // let (base_volume, quote_volume) = trade_repo
        //     .get_24h_volume(base_mint, quote_mint)
        //     .await
        //     .context("Failed to fetch 24h volume")?;
        // Ok((base_volume, quote_volume))
    }
}

