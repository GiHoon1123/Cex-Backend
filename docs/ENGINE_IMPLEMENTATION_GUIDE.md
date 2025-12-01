# 통합 엔진 구현 가이드 (engine.rs)

## 📋 목차

1. [구현해야 할 것들](#구현해야-할-것들)
2. [아키텍처 개요](#아키텍처-개요)
3. [핵심 컴포넌트](#핵심-컴포넌트)
4. [구현 순서](#구현-순서)
5. [난이도 높은 이유](#난이도-높은-이유)
6. [주의사항](#주의사항)

---

## 구현해야 할 것들

### 1. **HighPerformanceEngine 구조체**

```rust
pub struct HighPerformanceEngine {
    // 주문 큐 (Ring Buffer)
    order_tx: Sender<OrderCommand>,      // 주문 전송 채널
    order_rx: Receiver<OrderCommand>,    // 주문 수신 채널

    // WAL 채널
    wal_tx: Sender<WalEntry>,            // WAL 전송 채널
    wal_rx: Receiver<WalEntry>,           // WAL 수신 채널

    // 핵심 컴포넌트
    orderbooks: HashMap<TradingPair, OrderBook>,  // 거래쌍별 오더북
    matcher: Matcher,                              // 매칭 엔진
    executor: Executor,                            // 체결 엔진
    balance_cache: BalanceCache,                  // 잔고 캐시

    // 스레드 핸들
    engine_thread: Option<thread::JoinHandle<()>>,  // 매칭 스레드
    wal_thread: Option<thread::JoinHandle<()>>,      // WAL 스레드

    // 상태 관리
    running: Arc<AtomicBool>,                      // 실행 중 여부
}
```

### 2. **OrderCommand (주문 명령)**

```rust
pub enum OrderCommand {
    SubmitOrder(OrderEntry),              // 주문 제출
    CancelOrder {                          // 주문 취소
        order_id: u64,
        user_id: u64,
        trading_pair: TradingPair,
    },
    GetOrderbook {                         // 오더북 조회
        trading_pair: TradingPair,
        depth: Option<usize>,
        response: Sender<(Vec<OrderEntry>, Vec<OrderEntry>)>,
    },
    GetBalance {                           // 잔고 조회
        user_id: u64,
        mint: String,
        response: Sender<(Decimal, Decimal)>,
    },
    LockBalance {                          // 잔고 잠금
        user_id: u64,
        mint: String,
        amount: Decimal,
        response: Sender<Result<()>>,
    },
    UnlockBalance {                        // 잔고 해제
        user_id: u64,
        mint: String,
        amount: Decimal,
        response: Sender<Result<()>>,
    },
}
```

### 3. **스레드 구조**

```
┌─────────────────────────────────────────┐
│  API Handler (tokio async)              │
│  - OrderService::create_order()         │
│  - order_tx.send(SubmitOrder(...))      │
└──────────────┬──────────────────────────┘
               │
               │ crossbeam::channel
               ▼
┌─────────────────────────────────────────┐
│  Engine Thread (Core 0)                  │
│  - 코어 고정 (core_affinity)            │
│  - 실시간 스케줄링 (SCHED_FIFO)         │
│  - order_rx.recv() 루프                 │
│    ├─ OrderBook::add_order()           │
│    ├─ Matcher::match_order()           │
│    ├─ Executor::execute_trade()         │
│    └─ wal_tx.send(WalEntry)             │
└──────────────┬──────────────────────────┘
               │
               │ crossbeam::channel
               ▼
┌─────────────────────────────────────────┐
│  WAL Thread (Core 1)                    │
│  - 코어 고정 (core_affinity)            │
│  - wal_rx.recv() 루프                   │
│  - WalWriter::append()                  │
│  - fsync() (주기적)                     │
└─────────────────────────────────────────┘
```

### 4. **Engine Trait 구현**

```rust
#[async_trait]
impl Engine for HighPerformanceEngine {
    async fn submit_order(&self, order: OrderEntry) -> Result<Vec<MatchResult>> {
        // 1. 주문 명령 생성
        let (tx, rx) = oneshot::channel();
        let cmd = OrderCommand::SubmitOrder(order);

        // 2. 엔진 스레드로 전송
        self.order_tx.send(cmd)?;

        // 3. 결과 대기 (비동기)
        rx.await?
    }

    async fn cancel_order(...) -> Result<OrderEntry> { ... }
    async fn get_orderbook(...) -> Result<(...)> { ... }
    async fn lock_balance(...) -> Result<()> { ... }
    async fn unlock_balance(...) -> Result<()> { ... }
    async fn get_balance(...) -> Result<(Decimal, Decimal)> { ... }
    async fn start(&self) -> Result<()> { ... }
    async fn stop(&self) -> Result<()> { ... }
}
```

### 5. **엔진 스레드 루프**

```rust
fn engine_thread_loop(
    rx: Receiver<OrderCommand>,
    wal_tx: Sender<WalEntry>,
    orderbooks: &mut HashMap<TradingPair, OrderBook>,
    matcher: &mut Matcher,
    executor: &mut Executor,
) {
    // 1. 코어 고정 (Core 0)
    core_affinity::set_for_current(CoreId { id: 0 });

    // 2. 실시간 스케줄링 (SCHED_FIFO)
    set_realtime_scheduling();

    // 3. 메인 루프
    loop {
        match rx.recv() {
            Ok(OrderCommand::SubmitOrder(mut order)) => {
                // 3-1. TradingPair 찾기
                let pair = TradingPair::new(order.base_mint.clone(), order.quote_mint.clone());
                let orderbook = orderbooks.entry(pair).or_insert_with(|| OrderBook::new(pair));

                // 3-2. WAL 기록 (먼저!)
                wal_tx.send(WalEntry::OrderCreated { ... }).unwrap();

                // 3-3. 오더북에 추가
                orderbook.add_order(order.clone());

                // 3-4. 매칭 시도
                let matches = matcher.match_order(&mut order, orderbook);

                // 3-5. 체결 처리
                for match_result in &matches {
                    executor.execute_trade(match_result)?;
                }

                // 3-6. 결과 반환 (oneshot 채널)
                // ...
            }
            Ok(OrderCommand::CancelOrder { ... }) => { ... }
            Err(_) => break,  // 채널 닫힘
        }
    }
}
```

### 6. **WAL 스레드 루프**

```rust
fn wal_thread_loop(
    rx: Receiver<WalEntry>,
    wal_dir: PathBuf,
) {
    // 1. 코어 고정 (Core 1)
    core_affinity::set_for_current(CoreId { id: 1 });

    // 2. WAL Writer 생성
    let mut wal_writer = WalWriter::new(&wal_dir, 10)?;  // 10개마다 fsync

    // 3. 메인 루프
    loop {
        match rx.recv() {
            Ok(entry) => {
                wal_writer.append(&entry)?;
            }
            Err(_) => {
                // 마지막 동기화
                wal_writer.sync()?;
                break;
            }
        }
    }
}
```

---

## 아키텍처 개요

### 데이터 흐름

```
API Request
    ↓
OrderService::create_order()
    ↓
order_tx.send(SubmitOrder)  [crossbeam::channel]
    ↓
Engine Thread (Core 0)
    ├─ WAL 기록 → wal_tx.send() [crossbeam::channel]
    ├─ OrderBook::add_order()
    ├─ Matcher::match_order()
    └─ Executor::execute_trade()
        └─ wal_tx.send(TradeExecuted) [crossbeam::channel]
    ↓
WAL Thread (Core 1)
    └─ WalWriter::append() → fsync()
```

### 동시성 모델

- **싱글 스레드 엔진**: 모든 주문 처리는 하나의 스레드에서만
- **Lock-free 채널**: `crossbeam::channel` (SPSC 패턴)
- **비동기 응답**: `tokio::sync::oneshot` 채널로 결과 반환

---

## 핵심 컴포넌트

### 1. **Channel 구조**

```rust
// 주문 명령 채널 (SPSC)
order_tx: Sender<OrderCommand>
order_rx: Receiver<OrderCommand>

// WAL 채널 (SPSC)
wal_tx: Sender<WalEntry>
wal_rx: Receiver<WalEntry>

// 응답 채널 (oneshot, 요청마다 생성)
response_tx: Sender<Result<...>>
response_rx: Receiver<Result<...>>
```

### 2. **코어 고정**

```rust
use core_affinity::{set_for_current, CoreId};

// 엔진 스레드: Core 0
set_for_current(CoreId { id: 0 });

// WAL 스레드: Core 1
set_for_current(CoreId { id: 1 });
```

### 3. **실시간 스케줄링**

```rust
use nix::sched::{sched_setaffinity, CpuSet};
use nix::unistd::Pid;

// SCHED_FIFO 설정 (최고 우선순위)
let mut params = sched_param {
    sched_priority: 99,
};
sched_setscheduler(Pid::from_raw(0), SchedPolicy::Fifo, &params)?;
```

---

## 구현 순서

### Phase 1: 기본 구조 (난이도: ⭐⭐⭐)

1. ✅ `HighPerformanceEngine` 구조체 정의
2. ✅ `OrderCommand` enum 정의
3. ✅ Channel 생성 (`crossbeam::channel`)
4. ✅ 기본 `new()` 메서드

### Phase 2: 스레드 시작 (난이도: ⭐⭐⭐⭐)

5. ✅ `start()` 메서드 구현
   - 엔진 스레드 시작
   - WAL 스레드 시작
   - 코어 고정 (선택적)
   - 실시간 스케줄링 (선택적)

### Phase 3: 엔진 루프 (난이도: ⭐⭐⭐⭐⭐)

6. ✅ `engine_thread_loop()` 구현
   - `OrderCommand` 처리
   - `SubmitOrder` 처리
   - `CancelOrder` 처리
   - 응답 채널로 결과 반환

### Phase 4: WAL 루프 (난이도: ⭐⭐⭐)

7. ✅ `wal_thread_loop()` 구현
   - `WalEntry` 수신
   - `WalWriter::append()`
   - 주기적 `fsync()`

### Phase 5: Trait 구현 (난이도: ⭐⭐⭐⭐)

8. ✅ `Engine` trait 구현
   - `submit_order()` - oneshot 채널 사용
   - `cancel_order()` - oneshot 채널 사용
   - `get_orderbook()` - oneshot 채널 사용
   - `lock_balance()` - oneshot 채널 사용
   - `unlock_balance()` - oneshot 채널 사용
   - `get_balance()` - oneshot 채널 사용

### Phase 6: 종료 처리 (난이도: ⭐⭐⭐)

9. ✅ `stop()` 메서드 구현
   - 채널 닫기
   - 스레드 종료 대기
   - 최종 WAL 동기화

---

## 난이도 높은 이유

### 1. **멀티스레딩 복잡성** ⭐⭐⭐⭐⭐

- 스레드 간 통신 (Channel)
- 동시성 제어 (Lock-free)
- 스레드 생명주기 관리
- 에러 전파 (스레드 → 메인)

### 2. **비동기 + 동기 혼합** ⭐⭐⭐⭐

- `async fn` (API Handler)
- `thread::spawn` (엔진 스레드)
- `tokio::sync::oneshot` (응답 채널)
- `crossbeam::channel` (명령 채널)

### 3. **메시지 패싱 패턴** ⭐⭐⭐⭐

- Request-Response 패턴
- 각 요청마다 oneshot 채널 생성
- 타임아웃 처리
- 에러 처리

### 4. **시스템 레벨 최적화** ⭐⭐⭐⭐⭐

- 코어 고정 (Linux 전용)
- 실시간 스케줄링 (루트 권한 필요)
- NUMA 최적화 (선택적)

### 5. **상태 관리** ⭐⭐⭐⭐

- 엔진 실행 중 여부 (`AtomicBool`)
- 스레드 핸들 관리
- 안전한 종료 (Graceful Shutdown)

---

## 주의사항

### 1. **채널 타임아웃**

```rust
// ❌ 무한 대기 (데드락 위험)
let result = rx.await?;

// ✅ 타임아웃 설정
let result = tokio::time::timeout(
    Duration::from_millis(100),
    rx
).await??;
```

### 2. **에러 전파**

```rust
// 엔진 스레드에서 에러 발생 시
// oneshot 채널로 에러 전달
response_tx.send(Err(error)).unwrap();
```

### 3. **스레드 종료**

```rust
// 채널 닫기 → 스레드 루프 종료
drop(order_tx);  // order_rx.recv()가 Err 반환
```

### 4. **코어 고정 실패**

```rust
// 코어 고정 실패해도 계속 진행 (경고만)
if let Err(e) = set_for_current(CoreId { id: 0 }) {
    log::warn!("Failed to set core affinity: {}", e);
}
```

### 5. **실시간 스케줄링 권한**

```rust
// 루트 권한 없으면 실패 (경고만)
if let Err(e) = set_realtime_scheduling() {
    log::warn!("Failed to set realtime scheduling: {}", e);
}
```

---

## 의존성 추가

```toml
[dependencies]
crossbeam = "0.8"           # Lock-free channels
core_affinity = "0.8"       # CPU core pinning
nix = "0.27"                # Real-time scheduling
tokio = { version = "1", features = ["sync", "time"] }
async-trait = "0.1"
```

---

## 테스트 전략

### 1. **단위 테스트**

```rust
#[test]
fn test_submit_order() {
    let engine = HighPerformanceEngine::new();
    engine.start().unwrap();

    let order = create_test_order();
    let matches = engine.submit_order(order).await.unwrap();

    assert_eq!(matches.len(), 1);
}
```

### 2. **통합 테스트**

```rust
#[tokio::test]
async fn test_concurrent_orders() {
    let engine = Arc::new(HighPerformanceEngine::new());
    engine.start().await.unwrap();

    // 여러 주문 동시 제출
    let handles: Vec<_> = (0..100)
        .map(|i| {
            let engine = engine.clone();
            tokio::spawn(async move {
                engine.submit_order(create_order(i)).await
            })
        })
        .collect();

    // 모든 주문 완료 대기
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
}
```

---

## 다음 단계

1. ✅ 기본 구조 구현
2. ✅ 스레드 시작/종료
3. ✅ 주문 처리 루프
4. ✅ WAL 루프
5. ✅ 테스트 작성
6. ✅ 성능 벤치마크

**준비되셨으면 `engine.rs` 파일 생성부터 시작하세요!** 🚀
