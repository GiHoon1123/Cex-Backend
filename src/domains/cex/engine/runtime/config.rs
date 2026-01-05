// =====================================================
// CoreConfig - 코어 설정 (환경별)
// =====================================================
// 역할: 환경(dev/prod)에 따라 코어 고정 설정을 자동으로 결정
//
// dev (로컬, 11코어):
//   - Engine: Core 0
//   - WAL: Core 1
//   - DB Writer: Core 2
//   - UDP Feed: Core 3
//
// prod (인스턴스, 2코어):
//   - Engine: Core 0 (코어 고정 + 우선순위 99)
//   - WAL: Core 1 (코어 고정)
//   - DB Writer: 코어 고정 안 함
// =====================================================

/// 코어 설정 구조체
/// 
/// 환경 변수 `RUST_ENV`에 따라 자동으로 코어 설정을 결정합니다.
/// 
/// # 사용 예시
/// ```
/// let config = CoreConfig::from_env();
/// CoreConfig::set_core(Some(config.engine_core));  // Core 0
/// ```
pub struct CoreConfig {
    /// 엔진 스레드 코어 (항상 Some)
    pub engine_core: usize,
    /// WAL 스레드 코어 (항상 Some)
    pub wal_core: usize,
    /// DB Writer 스레드 코어 (dev만 Some)
    pub db_writer_core: Option<usize>,
}

impl CoreConfig {
    /// 환경 변수에서 코어 설정 읽기
    /// 
    /// # 환경 변수
    /// * `RUST_ENV` - "dev" 또는 "prod" (기본값: "dev")
    /// 
    /// # Returns
    /// 환경에 맞는 코어 설정
    /// 
    /// # Examples
    /// ```
    /// // dev 환경
    /// RUST_ENV=dev
    /// // → engine_core: 0, wal_core: 1, db_writer_core: Some(2)
    /// 
    /// // prod 환경
    /// RUST_ENV=prod
    /// // → engine_core: 0, wal_core: 1, db_writer_core: None
    /// ```
    pub fn from_env() -> Self {
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string());
        eprintln!("[CoreConfig] Environment: {}", env);
        
        let config = match env.as_str() {
            "dev" => {
                // 로컬 환경 (11코어) - 여러 코어 활용
                Self {
                    engine_core: 0,
                    wal_core: 1,
                    db_writer_core: Some(2),
                }
            }
            "prod" => {
                // 프로덕션 환경 (2코어) - 하드웨어 자원 최대 활용
                // 
                // 설계 원칙:
                // 1. 엔진 스레드: Core 0 고정 + 우선순위 99 (최우선 실행)
                //    → 주문 처리 성능 최대화, 지연 시간 최소화
                // 2. WAL 스레드: 코어 고정 안 함 (OS 스케줄링)
                //    → I/O 바운드 작업, 유연한 스케줄링으로 다른 작업과 공유
                // 3. DB Writer 스레드: 코어 고정 안 함 (OS 스케줄링)
                //    → DB 작업, 유연한 스케줄링으로 다른 작업과 공유
                // 4. Docker: CPU/메모리 제한 없음 → 서버의 모든 리소스 활용
                //
                // 결과: 엔진은 Core 0 독점, 나머지는 OS가 최적 스케줄링
                Self {
                    engine_core: 0,    // Core 0에 고정 (엔진 스레드 전용, 우선순위 99)
                    wal_core: 999,     // 코어 고정 안 함 (특수 값, OS 스케줄링)
                    db_writer_core: None,  // 코어 고정 안 함 (OS 스케줄링)
                }
            }
            _ => {
                // 기본값 (dev와 동일)
                Self {
                    engine_core: 0,
                    wal_core: 1,
                    db_writer_core: None,
                }
            }
        };
        
        eprintln!("[CoreConfig] Engine core: {} ({}), WAL core: {} ({}), DB Writer core: {:?}",
            config.engine_core,
            if config.engine_core != 999 { "pinned" } else { "not pinned" },
            config.wal_core,
            if config.wal_core != 999 { "pinned" } else { "not pinned" },
            config.db_writer_core
        );
        
        config
    }
    
    /// 코어 고정 설정 (선택적)
    /// 
    /// # Arguments
    /// * `core_id` - 고정할 코어 번호 (None이면 고정 안 함)
    /// 
    /// # Note
    /// 코어 고정 실패해도 경고만 출력하고 계속 진행
    /// (권한 없거나 코어가 없을 수 있음)
    /// 
    /// # 구현
    /// 현재는 주석 처리 (core_affinity 의존성 추가 후 활성화)
    /// ```rust
    /// use core_affinity::{set_for_current, CoreId};
    /// if let Err(e) = set_for_current(CoreId { id: core }) {
    ///     log::warn!("Failed to set core affinity to {}: {}", core, e);
    /// }
    /// ```
    /// 코어 고정 설정 (Linux 전용)
    /// 
    /// # Arguments
    /// * `core_id` - 고정할 코어 번호 (None이면 고정 안 함)
    /// 
    /// # Note
    /// - Linux에서만 동작 (macOS/Windows에서는 무시)
    /// - 코어 고정 실패해도 경고만 출력하고 계속 진행
    /// - 권한 없거나 코어가 없을 수 있음
    pub fn set_core(core_id: Option<usize>) {
        #[cfg(target_os = "linux")]
        {
            if let Some(core) = core_id {
                use core_affinity::{set_for_current, CoreId};
                if set_for_current(CoreId { id: core }) {
                    eprintln!("Core affinity set to core {}", core);
                } else {
                    eprintln!("Failed to set core affinity to {}", core);
                    eprintln!("   This is normal if running without proper permissions");
                }
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // macOS/Windows에서는 코어 고정 지원 안 함 (조용히 무시)
            let _ = core_id;
        }
    }
    
    /// 실시간 스케줄링 설정 (활성화)
    /// 
    /// # Arguments
    /// * `priority` - 스케줄링 우선순위 (1-99, 99가 최고)
    /// 
    /// # Note
    /// Linux 환경에서만 동작하며, 루트 권한 또는 CAP_SYS_NICE 권한 필요
    /// 실패해도 경고만 출력하고 계속 진행
    pub fn set_realtime_scheduling(priority: u8) {
        #[cfg(target_os = "linux")]
        {
            // nix 0.27에서는 sched API가 변경되었으므로 libc를 직접 사용
            use libc::{sched_param, SCHED_FIFO, sched_setscheduler};
            
            let params = sched_param {
                sched_priority: priority as i32,
            };
            
            let result = unsafe {
                sched_setscheduler(0, SCHED_FIFO, &params)
            };
            
            if result == 0 {
                eprintln!("✅ Real-time scheduling enabled (priority: {})", priority);
            } else {
                use std::io;
                let errno = io::Error::last_os_error().raw_os_error().unwrap_or(-1);
                eprintln!("⚠️  Failed to set real-time scheduling (errno: {})", errno);
                eprintln!("   This is normal if running without root/CAP_SYS_NICE permissions");
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // macOS/Windows에서는 실시간 스케줄링 지원 안 함 (조용히 무시)
            let _ = priority;
        }
    }
}

