// =====================================================
// OrderCommand - 엔진 스레드로 전달할 명령
// =====================================================
// 역할: API Handler (tokio async)에서 엔진 스레드 (blocking)로
//       명령을 전달하기 위한 메시지 타입
//
// 왜 필요한가?
// - tokio async와 blocking thread 간 통신
// - Request-Response 패턴 구현
// - oneshot 채널로 결과 반환
// =====================================================

use anyhow::Result;
use tokio::sync::oneshot;
use rust_decimal::Decimal;

use crate::domains::cex::engine::types::{TradingPair, OrderEntry, MatchResult};

/// 엔진 스레드로 전달할 명령
/// 
/// 각 명령은 엔진 스레드에서 순차적으로 처리됩니다.
/// 결과는 oneshot 채널을 통해 비동기로 반환됩니다.
#[derive(Debug)]
pub enum OrderCommand {
    /// 주문 제출
    /// 
    /// # Fields
    /// * `order` - 제출할 주문
    /// * `response` - 결과를 반환할 oneshot 채널
    /// 
    /// # 처리 과정
    /// 1. WAL 메시지 발행 (OrderCreated)
    /// 2. OrderBook에 추가
    /// 3. Matcher로 매칭 시도
    /// 4. 체결된 경우 Executor로 처리
    /// 5. MatchResult 목록을 response로 전송
    SubmitOrder {
        order: OrderEntry,
        response: Option<oneshot::Sender<Result<Vec<MatchResult>>>>,
    },
    
    /// 주문 취소
    /// 
    /// # Fields
    /// * `order_id` - 취소할 주문 ID
    /// * `user_id` - 주문한 사용자 ID (권한 확인)
    /// * `trading_pair` - 거래쌍
    /// * `response` - 취소된 주문을 반환할 oneshot 채널
    /// 
    /// # 처리 과정
    /// 1. OrderBook에서 주문 찾기
    /// 2. 권한 확인 (user_id 일치)
    /// 3. WAL 메시지 발행 (OrderCancelled)
    /// 4. OrderBook에서 제거
    /// 5. 잔고 잠금 해제
    /// 6. 취소된 주문을 response로 전송
    CancelOrder {
        order_id: u64,
        user_id: u64,
        trading_pair: TradingPair,
        response: oneshot::Sender<Result<OrderEntry>>,
    },
    
    /// 오더북 조회
    /// 
    /// # Fields
    /// * `trading_pair` - 조회할 거래쌍
    /// * `depth` - 조회할 가격 레벨 개수 (None이면 전체)
    /// * `response` - 오더북을 반환할 oneshot 채널
    /// 
    /// # 반환값
    /// `(매수 주문 목록, 매도 주문 목록)`
    GetOrderbook {
        trading_pair: TradingPair,
        depth: Option<usize>,
        response: oneshot::Sender<Result<(Vec<OrderEntry>, Vec<OrderEntry>)>>,
    },
    
    /// 잔고 조회
    /// 
    /// # Fields
    /// * `user_id` - 사용자 ID
    /// * `mint` - 자산 종류
    /// * `response` - 잔고를 반환할 oneshot 채널
    /// 
    /// # 반환값
    /// `(available, locked)`
    GetBalance {
        user_id: u64,
        mint: String,
        response: oneshot::Sender<Result<(Decimal, Decimal)>>,
    },
    
    /// 잔고 잠금 (주문 생성 시)
    /// 
    /// # Fields
    /// * `user_id` - 사용자 ID
    /// * `mint` - 자산 종류
    /// * `amount` - 잠글 수량
    /// * `response` - 결과를 반환할 oneshot 채널
    /// 
    /// # 처리 과정
    /// 1. BalanceCache에서 잔고 확인
    /// 2. available >= amount 확인
    /// 3. available 감소, locked 증가
    /// 4. WAL 메시지 발행 (BalanceLocked)
    LockBalance {
        user_id: u64,
        mint: String,
        amount: Decimal,
        response: oneshot::Sender<Result<()>>,
    },
    
    /// 잔고 잠금 해제 (주문 취소 시)
    /// 
    /// # Fields
    /// * `user_id` - 사용자 ID
    /// * `mint` - 자산 종류
    /// * `amount` - 해제할 수량
    /// * `response` - 결과를 반환할 oneshot 채널
    /// 
    /// # 처리 과정
    /// 1. BalanceCache에서 locked 확인
    /// 2. locked >= amount 확인
    /// 3. locked 감소, available 증가
    /// 4. WAL 메시지 발행 (OrderCancelled)
    UnlockBalance {
        user_id: u64,
        mint: String,
        amount: Decimal,
        response: oneshot::Sender<Result<()>>,
    },
}

