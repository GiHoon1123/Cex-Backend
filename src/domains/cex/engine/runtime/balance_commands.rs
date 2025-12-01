// =====================================================
// BalanceCommand - 잔고 업데이트 명령
// =====================================================
// 역할: 외부 입금/출금 서비스에서 엔진 스레드로
//       잔고 업데이트 명령을 전달하기 위한 메시지 타입
//
// 사용 시나리오:
// 1. 외부 지갑에서 우리 지갑으로 자산 입금 (온체인 트랜잭션)
// 2. 어드민 또는 이벤트로 인한 서비스 내 잔액 업데이트
//
// 특징:
// - 주문 큐와 별도 큐로 분리 (입금 우선 처리)
// - oneshot 채널로 결과 반환
// =====================================================

use anyhow::Result;
use tokio::sync::oneshot;
use rust_decimal::Decimal;

/// 잔고 업데이트 명령
/// 
/// 엔진 스레드에서 순차적으로 처리되며,
/// 결과는 oneshot 채널을 통해 비동기로 반환됩니다.
#[derive(Debug)]
pub enum BalanceCommand {
    /// 잔고 업데이트 (입금/출금)
    /// 
    /// # Fields
    /// * `user_id` - 사용자 ID
    /// * `mint` - 자산 종류 (예: "SOL", "USDT")
    /// * `available_delta` - available 증감량 (양수: 입금, 음수: 출금)
    /// * `response` - 결과를 반환할 oneshot 채널
    /// 
    /// # 처리 과정
    /// 1. BalanceCache에서 잔고 조회/생성
    /// 2. available 업데이트 (기존 + delta)
    /// 3. WAL 메시지 발행 (BalanceUpdated)
    /// 4. DB 명령 전송 (UpdateBalance) → DB Writer가 배치로 처리
    /// 5. 성공/실패 결과를 response로 전송
    /// 
    /// # 예시
    /// ```rust
    /// // 100 USDT 입금
    /// BalanceCommand::UpdateBalance {
    ///     user_id: 123,
    ///     mint: "USDT".to_string(),
    ///     available_delta: Decimal::new(100, 0),
    ///     response: tx,
    /// }
    /// ```
    UpdateBalance {
        user_id: u64,
        mint: String,
        available_delta: Decimal,  // 양수: 입금, 음수: 출금
        response: oneshot::Sender<Result<()>>,
    },
}

