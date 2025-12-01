/// ID 생성기
/// ID Generator
///
/// 역할:
/// - 주문 ID 생성 (Order ID)
/// - 체결 ID 생성 (Trade ID)
/// - 타임스탬프 기반 + 카운터를 사용하여 스레드 안전하게 ID 생성
///
/// ID 형식:
/// `(timestamp_ms << 20) + counter`
/// - 상위 비트: 타임스탬프 (밀리초)
/// - 하위 20비트: 카운터 (같은 밀리초에 최대 1,048,576개 생성 가능)
///
/// 사용 방법:
/// ```rust
/// // 서버 시작 시 한 번만 호출
/// OrderIdGenerator::initialize();
/// TradeIdGenerator::initialize();
///
/// // ID 생성
/// let order_id = OrderIdGenerator::next();
/// let trade_id = TradeIdGenerator::next();
/// ```
///
/// 특징:
/// - DB 접근 없이 메모리에서만 동작
/// - 서버 재시작 시에도 ID가 계속 증가
/// - 동시성 안전 (AtomicU64 사용)

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 주문 ID 베이스 (타임스탬프, 서버 시작 시 초기화)
static ORDER_ID_BASE: AtomicU64 = AtomicU64::new(0);

/// 주문 ID 카운터
static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 체결 ID 베이스 (타임스탬프, 서버 시작 시 초기화)
static TRADE_ID_BASE: AtomicU64 = AtomicU64::new(0);

/// 체결 ID 카운터
static TRADE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 주문 ID 생성기
/// Order ID Generator
///
/// 타임스탬프 기반 + 카운터를 사용하여 스레드 안전하게 ID 생성
/// DB 접근 없이 메모리에서만 동작
pub struct OrderIdGenerator;

impl OrderIdGenerator {
    /// ID 생성기 초기화
    /// Initialize ID generator
    ///
    /// 서버 시작 시 한 번만 호출합니다.
    /// 현재 타임스탬프를 베이스로 설정합니다.
    ///
    /// # Example
    /// ```rust
    /// OrderIdGenerator::initialize();
    /// ```
    pub fn initialize() {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // 타임스탬프를 상위 비트에 배치 (하위 20비트는 카운터용)
        ORDER_ID_BASE.store(timestamp_ms << 20, Ordering::SeqCst);
        ORDER_ID_COUNTER.store(0, Ordering::SeqCst);
    }

    /// 다음 주문 ID 생성
    /// Generate next order ID
    ///
    /// # Returns
    /// 다음 주문 ID: `(timestamp_ms << 20) + counter`
    ///
    /// # Thread Safety
    /// AtomicU64를 사용하여 스레드 안전하게 동작
    ///
    /// # ID 형식
    /// - 상위 비트: 타임스탬프 (밀리초)
    /// - 하위 20비트: 카운터 (같은 밀리초에 최대 1,048,576개 생성 가능)
    pub fn next() -> u64 {
        let base = ORDER_ID_BASE.load(Ordering::SeqCst);
        let counter = ORDER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // 카운터가 20비트를 넘어가면 타임스탬프 업데이트
        if counter >= (1 << 20) {
            // 새로운 타임스탬프로 업데이트
            let new_timestamp_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            ORDER_ID_BASE.store(new_timestamp_ms << 20, Ordering::SeqCst);
            ORDER_ID_COUNTER.store(1, Ordering::SeqCst);
            return (new_timestamp_ms << 20) + 1;
        }
        
        base + counter
    }

    /// 현재 ID 값 조회 (디버깅용)
    /// Get current ID value (for debugging)
    pub fn current() -> u64 {
        let base = ORDER_ID_BASE.load(Ordering::SeqCst);
        let counter = ORDER_ID_COUNTER.load(Ordering::SeqCst);
        base + counter
    }
}

/// 체결 ID 생성기
/// Trade ID Generator
///
/// 타임스탬프 기반 + 카운터를 사용하여 스레드 안전하게 ID 생성
/// DB 접근 없이 메모리에서만 동작
pub struct TradeIdGenerator;

impl TradeIdGenerator {
    /// ID 생성기 초기화
    /// Initialize ID generator
    ///
    /// 서버 시작 시 한 번만 호출합니다.
    /// 현재 타임스탬프를 베이스로 설정합니다.
    ///
    /// # Example
    /// ```rust
    /// TradeIdGenerator::initialize();
    /// ```
    pub fn initialize() {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // 타임스탬프를 상위 비트에 배치 (하위 20비트는 카운터용)
        TRADE_ID_BASE.store(timestamp_ms << 20, Ordering::SeqCst);
        TRADE_ID_COUNTER.store(0, Ordering::SeqCst);
    }

    /// 다음 체결 ID 생성
    /// Generate next trade ID
    ///
    /// # Returns
    /// 다음 체결 ID: `(timestamp_ms << 20) + counter`
    ///
    /// # Thread Safety
    /// AtomicU64를 사용하여 스레드 안전하게 동작
    ///
    /// # ID 형식
    /// - 상위 비트: 타임스탬프 (밀리초)
    /// - 하위 20비트: 카운터 (같은 밀리초에 최대 1,048,576개 생성 가능)
    pub fn next() -> u64 {
        let base = TRADE_ID_BASE.load(Ordering::SeqCst);
        let counter = TRADE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // 카운터가 20비트를 넘어가면 타임스탬프 업데이트
        if counter >= (1 << 20) {
            // 새로운 타임스탬프로 업데이트
            let new_timestamp_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            TRADE_ID_BASE.store(new_timestamp_ms << 20, Ordering::SeqCst);
            TRADE_ID_COUNTER.store(1, Ordering::SeqCst);
            return (new_timestamp_ms << 20) + 1;
        }
        
        base + counter
    }

    /// 현재 ID 값 조회 (디버깅용)
    /// Get current ID value (for debugging)
    pub fn current() -> u64 {
        let base = TRADE_ID_BASE.load(Ordering::SeqCst);
        let counter = TRADE_ID_COUNTER.load(Ordering::SeqCst);
        base + counter
    }
}

