# 체결 엔진 설계 문서 (High-Performance Matching Engine Design)

## 📋 목차

1. [개요](#개요)
2. [아키텍처](#아키텍처)
3. [핵심 컴포넌트](#핵심-컴포넌트)
4. [고성능 최적화 기법](#고성능-최적화-기법)
5. [처리 프로세스](#처리-프로세스)
6. [동시성 제어](#동시성-제어)
7. [데이터 구조](#데이터-구조)
8. [에러 처리 및 복구](#에러-처리-및-복구)

---

## 개요

### 목적

**실제 거래소 수준의 고성능 체결 엔진** - 이력서용 프로젝트를 위한 최고 수준의 성능 구현

### 주요 기능

- ✅ 지정가 주문 매칭 (Limit Order Matching)
- ✅ 시장가 주문 즉시 체결 (Market Order Execution)
- ✅ 부분 체결 처리 (Partial Fill)
- ✅ 메모리 기반 오더북 관리 (In-Memory Order Book)
- ✅ WAL (Write-Ahead Logging) 기반 복구 시스템
- ✅ 싱글 스레드 엔진 + 코어 고정
- ✅ 비동기 DB 쓰기 (백그라운드 배치 처리)
- ✅ 메모리 기반 잔고 관리

### 성능 목표 (실제 거래소 수준)

- **주문 처리 지연시간**: < 0.5ms (평균), < 1ms (99th percentile)
- **초당 주문 처리량**: 50,000+ TPS (Transactions Per Second)
- **체결 처리량**: 100,000+ orders/sec (매칭 포함)
- **메모리 사용량**: 최적화된 데이터 구조로 < 100MB per 100K orders

---

## 아키텍처

### 전체 흐름 (고성능 버전)

```
[주문 생성 요청]
    ↓
[OrderService::create_order()]
    ↓
[메모리 잔고 확인 & 잠금] (메모리 기반, < 0.1ms)
    ↓
[WAL 파일에 즉시 기록] (디스크, < 1ms, 복구용)
    ↓
[Lock-free Ring Buffer로 전송] (논블로킹, < 0.01ms)
    ↓
[Engine 싱글 스레드 처리] (코어 고정, 실시간 스케줄링)
    ├─ OrderBook::add_order() ──→ [메모리 오더북 즉시 추가]
    ├─ Matcher::match_order() ──→ [매칭 시도]
    └─ Executor::execute_in_memory() ──→ [메모리에서 체결 처리]
    ↓
[체결 결과를 DB 쓰기 큐에 추가] (비동기, 백그라운드)
    ↓
[백그라운드 워커가 배치로 DB 쓰기] (10ms마다 또는 100개 모이면)
```

### 핵심 차이점: 메모리 우선 처리

| 구분          | 일반 방식 (느림)      | 고성능 방식 (빠름)       |
| ------------- | --------------------- | ------------------------ |
| **주문 저장** | DB 먼저 (5-10ms)      | 메모리 먼저 (< 0.1ms)    |
| **잔고 확인** | DB 조회 (2-5ms)       | 메모리 조회 (< 0.1ms)    |
| **체결 처리** | DB 트랜잭션 (10-20ms) | 메모리 처리 (< 0.5ms)    |
| **DB 쓰기**   | 동기 (즉시)           | 비동기 배치 (백그라운드) |
| **주문 큐**   | 일반 Channel          | Lock-free Ring Buffer    |

### 컴포넌트 구조

```
engine/
├── types.rs          # 공통 타입 정의
├── orderbook.rs      # 메모리 기반 오더북 관리
├── matcher.rs        # 주문 매칭 알고리즘 (Price-Time Priority)
├── executor.rs       # 메모리 기반 체결 실행
├── wal.rs            # Write-Ahead Logging (복구용)
├── engine.rs         # 메인 엔진 (싱글 스레드 + 코어 고정)
├── balance_cache.rs  # 메모리 기반 잔고 캐시
└── db_writer.rs      # 비동기 DB 쓰기 워커
```

---

## 핵심 컴포넌트

### 1. Engine (engine.rs) - 싱글 스레드 엔진

**역할**: 모든 주문 처리를 단일 스레드에서 순차 처리

**특징**:
- **CPU 코어 고정**: 특정 코어에 바인딩하여 캐시 효율 극대화
- **실시간 스케줄링**: Linux SCHED_FIFO로 최우선 처리
- **Lock-free**: 싱글 스레드이므로 락 불필요
- **Ring Buffer**: Lock-free ring buffer로 주문 수신

**주요 처리 흐름**:
```rust
// 1. CPU 코어 고정 및 실시간 스케줄링 설정
// 2. Lock-free ring buffer에서 주문 수신
// 3. 메모리 잔고 확인 & 잠금
// 4. WAL 기록
// 5. 오더북에 추가
// 6. 즉시 매칭 시도
// 7. 메모리에서 체결 처리
// 8. DB 쓰기 큐에 추가 (비동기)
```

### 2. OrderBook (orderbook.rs) - 메모리 기반 오더북

**역할**: 메모리에서 초고속 오더북 관리

**데이터 구조**:
```rust
struct PriceLevel {
    price: Decimal,
    orders: VecDeque<OrderEntry>,  // FIFO 큐 (시간순)
    total_amount: Decimal,  // 빠른 집계를 위한 캐시
}

struct OrderBook {
    buy_side: BTreeMap<Decimal, PriceLevel>,   // 매수 (가격 내림차순)
    sell_side: BTreeMap<Decimal, PriceLevel>,  // 매도 (가격 오름차순)
    orders_by_id: HashMap<u64, OrderEntry>,    // O(1) 조회
}
```

**주요 메서드**:
- `add_order()`: 주문 추가 (< 0.1ms)
- `get_best_bid()`: 최고 매수가 조회 (O(1))
- `get_best_ask()`: 최저 매도가 조회 (O(1))
- `remove_order()`: 주문 제거

### 3. Matcher (matcher.rs) - 매칭 알고리즘

**역할**: Price-Time Priority 기반 초고속 매칭

**매칭 규칙**:
1. **가격 우선순위**: 매수는 높은 가격, 매도는 낮은 가격
2. **시간 우선순위**: 같은 가격이면 먼저 들어온 주문 우선

**매칭 흐름**:
```rust
// 1. 반대 측 최적 가격 레벨 조회 (O(1))
// 2. 가격 조건 확인 (buy_order.price >= sell_order.price)
// 3. 체결 수량 계산 (부분 체결 지원)
// 4. MatchResult 생성
// 5. 오더북 업데이트 (체결된 주문 제거/수량 감소)
```

**성능**: < 0.5ms per order

### 4. Executor (executor.rs) - 메모리 기반 체결 실행

**역할**: 메모리에서만 체결 처리 (DB는 나중에)

**처리 순서**:
```rust
// 1. 잔고 업데이트 (메모리 캐시)
//    - 매수자: base_mint 증가, quote_mint 감소 (수수료 포함)
//    - 매도자: base_mint 감소, quote_mint 증가 (수수료 포함)
// 2. 주문 상태 업데이트 (메모리 오더북)
// 3. Trade 객체 생성 (DB 쓰기 큐에 추가)
```

**성능**: < 0.2ms per trade

### 5. BalanceCache (balance_cache.rs) - 메모리 잔고 관리

**역할**: 잔고 조회/업데이트를 메모리에서 초고속 처리

**데이터 구조**:
```rust
// DashMap: 동시성 안전한 HashMap (읽기는 락 없음)
balances: DashMap<(user_id, mint), CachedBalance>
```

**주요 기능**:
- `lock_balance()`: 잔고 확인 & 잠금 (< 0.1ms)
- `update_on_trade()`: 체결 후 잔고 업데이트
- `get_balance()`: 잔고 조회 (HTTP 핸들러에서 사용)

### 6. WAL (wal.rs) - Write-Ahead Logging

**역할**: 복구를 위한 순차 로그 파일 기록

**특징**:
- 주문/체결 즉시 로그 파일에 기록 (< 1ms)
- 서버 장애 시 로그에서 복구 가능
- 순차 쓰기만 (랜덤 쓰기 아님, 성능 영향 최소화)

**로그 형식**:
- `OrderCreated { order }`
- `OrderCancelled { order_id }`
- `TradeExecuted { trade }`

### 7. DB Writer (db_writer.rs) - 비동기 DB 쓰기 워커

**역할**: 백그라운드에서 배치로 DB에 쓰기

**전략**:
- **시간 기반**: 10ms마다 배치 쓰기
- **크기 기반**: 100개 모이면 즉시 쓰기
- **배치 트랜잭션**: 여러 작업을 하나의 트랜잭션으로 묶기

**처리 항목**:
- 주문 생성/업데이트
- 체결 내역
- 잔고 업데이트

---

## 고성능 최적화 기법

### 1. CPU 코어 고정 바인딩 (Core Pinning)

**목적**: 특정 CPU 코어에 스레드를 고정하여 캐시 효율 극대화

**구현**:
- `core_affinity` crate 사용
- 엔진 스레드를 코어 0에 고정
- 컨텍스트 스위칭 최소화

**성능 향상**: 캐시 히트율 증가, 예측 가능한 지연시간

### 2. 실시간 스케줄링 (Real-time Scheduling)

**목적**: 엔진 스레드에 최고 우선순위 부여하여 지연시간 최소화

**구현**:
- Linux `SCHED_FIFO` 스케줄링 정책 사용
- `nix` crate를 통해 POSIX 스케줄링 설정
- 우선순위 99 (최고)로 설정

**주의사항**:
- Linux 환경에서만 가능
- 루트 권한 필요 (또는 CAP_SYS_NICE 권한)
- 잘못 설정 시 시스템 불안정 가능

**성능 향상**: 스케줄러 지연 제거, 예측 가능한 응답 시간

### 3. Lock-free Ring Buffer

**목적**: 주문 전달 시 락 오버헤드 제거

**현재**: `tokio::sync::mpsc::channel` (내부적으로 락 사용)

**개선**: `ringbuf` crate의 Lock-free ring buffer
- 단일 생산자/단일 소비자 (SPSC) 패턴
- 완전히 lock-free (스핀락도 없음)
- 캐시 친화적인 순차 접근

**성능 향상**: 락 경합 제거, 지연시간 10배 감소

### 4. Memory Pre-allocation (메모리 사전 할당)

**목적**: 런타임 메모리 할당 오버헤드 제거

**구현**:
- `Vec::with_capacity()`로 미리 할당
- 메모리 풀 패턴 사용
- 자주 사용하는 버퍼는 재사용

**적용 위치**:
- 주문 버퍼: 10,000개 주문 용량
- 매칭 결과 버퍼: 1,000개 체결 용량
- 문자열 버퍼 풀

**성능 향상**: 할당 오버헤드 제거, GC 압력 감소

### 5. Zero-copy 최적화

**목적**: 불필요한 데이터 복사 제거

**구현**:
- 가능한 곳에서 참조 (`&`) 사용
- 꼭 필요한 곳만 `Arc` (공유 소유권)
- `Bytes` 타입으로 zero-copy 네트워크 I/O

**적용 위치**:
- 매칭 함수: `&OrderEntry` 참조 사용
- 주문 전달: Arc로 공유 (복사 없음)
- 네트워크 버퍼: `Bytes` 타입

**성능 향상**: 메모리 대역폭 절약, CPU 사용량 감소

### 6. UDP 기반 Feed (오더북 스트리밍)

**목적**: 오더북 업데이트를 TCP보다 빠른 UDP로 전송

**구현**:
- `tokio::net::UdpSocket` 사용
- 멀티캐스트로 여러 클라이언트에 동시 전송
- 손실 허용 가능한 데이터에만 사용

**적용 위치**:
- 오더북 스냅샷/업데이트 스트리밍
- 시장 데이터 피드

**주의사항**:
- 손실 가능 (신뢰성 낮음)
- 주문/체결은 TCP 유지 (신뢰성 필요)

**성능 향상**: 네트워크 지연 감소, 처리량 증가

### 7. NUMA 최적화 (선택적)

**목적**: NUMA 아키텍처에서 메모리 접근 지연 최소화

**구현**:
- FFI로 `libnuma` C 라이브러리 호출
- 스레드와 메모리를 같은 NUMA 노드에 배치
- NUMA 노드별 메모리 할당

**적용 조건**:
- 멀티 소켓 서버 환경
- 대용량 메모리 사용 시
- 마이크로초 단위 최적화 필요 시

**성능 향상**: 메모리 접근 지연 30-50% 감소

---

## 처리 프로세스

### 시나리오 1: 지정가 매수 주문 생성 (전체 < 2ms)

```
1. [주문 생성 요청] POST /api/cex/orders
2. [OrderService::create_order()]
   - 메모리 잔고 확인 & 잠금 (< 0.1ms)
   - Lock-free ring buffer로 전송 (< 0.01ms)
3. [Engine 싱글 스레드 처리]
   - WAL 파일에 기록 (< 1ms)
   - OrderBook에 추가 (< 0.1ms)
   - Matcher 매칭 시도 (< 0.5ms)
   - Executor 체결 처리 (< 0.2ms)
4. [백그라운드 DB 쓰기]
   - 10ms 후 또는 100개 모이면 배치로 DB 저장
   - 클라이언트는 이미 응답 받음 ✅
```

### 시나리오 2: 시장가 매도 주문 (전체 < 1ms)

```
1. [주문 생성 요청]
2. [즉시 처리]
   - 메모리 잔고 잠금 (< 0.1ms)
   - WAL 기록 (< 1ms)
   - 오더북의 매수 측 최고가부터 즉시 체결 (< 0.5ms)
   - 부분 체결도 한 번에 처리
```

---

## 동시성 제어

### 싱글 스레드 엔진

- **모든 주문 처리는 하나의 스레드에서만**
- **락 불필요**: 동시성 문제 자체가 없음
- **Ring Buffer 사용**: 다른 스레드에서 주문 전송 (Lock-free)

### 잔고 캐시

- **DashMap 사용**: 동시성 안전한 HashMap
- **읽기는 락 없음**: HTTP 핸들러들이 동시에 읽기 가능
- **쓰기는 엔진에서만**: 싱글 스레드이므로 안전

### 주문 큐

- **Lock-free Ring Buffer**: 완전히 락 없음
- **SPSC 패턴**: 단일 생산자(HTTP 핸들러) / 단일 소비자(엔진)

---

## 데이터 구조

### 메모리 오더북 vs DB

| 구분         | 메모리 (OrderBook) | DB (orders 테이블) |
| ------------ | ------------------ | ------------------ |
| **용도**     | 실시간 매칭        | 영구 저장, 복구    |
| **데이터**   | 활성 주문만        | 모든 주문          |
| **정렬**     | BTreeMap (자동)    | 인덱스로 조회      |
| **업데이트** | 즉시               | 비동기 배치        |

### 동기화 전략

**초기 로드 (서버 시작 시)**:
```rust
// DB에서 활성 주문만 로드
let active_orders = db.get_active_orders().await?;
for order in active_orders {
    engine.add_order(order);
}
```

**주기적 동기화 (복구용, 선택적)**:
- 일반적으로 불필요 (WAL로 복구)
- 필요시 1분마다 메모리 ↔ DB 동기화

---

## 에러 처리 및 복구

### 1. 서버 재시작 시 복구

```rust
// 서버 시작 시
// 1. WAL 로그 재생
// 2. 각 엔트리를 메모리에 다시 적용
// 3. DB와 최종 동기화 (선택적)
```

### 2. 메모리 오류 처리

- **메모리 부족**: 주문 거부, 로그 기록
- **데이터 불일치**: WAL로 복구 시도

### 3. DB 쓰기 실패

- **재시도 큐**: 실패한 작업은 재시도 큐에 추가
- **로그 기록**: 모든 실패는 로그에 기록

---

## 성능 벤치마크 목표

### 목표 지표

- **주문 처리**: < 0.5ms (평균), < 1ms (99th percentile)
- **체결 처리**: < 0.2ms per trade
- **초당 처리량**: 50,000+ orders/sec
- **메모리 사용**: < 1KB per order

### 벤치마크 방법

- `criterion` crate 사용
- 각 컴포넌트별 성능 측정
- 지연시간 분포 분석

---

## 구현 우선순위

### Phase 1: 핵심 엔진 (1주)

1. ✅ Types 정의
2. ✅ OrderBook 기본 구조
3. ✅ Matcher 기본 로직
4. ✅ Engine 싱글 스레드 구조
5. ✅ Lock-free Ring Buffer

### Phase 2: 성능 최적화 (1주)

1. ✅ CPU 코어 고정
2. ✅ 실시간 스케줄링 (Linux)
3. ✅ Memory pre-allocation
4. ✅ Zero-copy 최적화
5. ✅ WAL 구현

### Phase 3: 고급 최적화 (3일)

1. ✅ UDP 기반 feed (오더북 스트리밍)
2. ✅ NUMA 최적화 (선택적)
3. ✅ DB 동기화 워커
4. ✅ 모니터링

---

## 참고 자료

- [Binance Matching Engine Architecture](https://www.binance.com/en/blog/matching-engine)
- [High-Frequency Trading Systems](https://www.amazon.com/High-Frequency-Trading-Practical-Algorithmic/dp/1118343506)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Write-Ahead Logging](https://en.wikipedia.org/wiki/Write-ahead_logging)
- [Lock-free Programming](https://preshing.com/20120612/an-introduction-to-lock-free-programming/)

---

**작성일**: 2025-01-24
**작성자**: AI Assistant
**버전**: 2.1 (High-Performance Edition with Advanced Optimizations)
