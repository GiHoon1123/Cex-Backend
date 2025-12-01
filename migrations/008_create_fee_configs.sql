-- =====================================================
-- 수수료 설정 테이블 (fee_configs)
-- =====================================================
-- 설명: 거래소의 거래 수수료 설정을 관리합니다.
-- 수수료는 거래쌍별로 다르게 설정할 수 있으며, 나중에 변경 가능합니다.
-- 
-- 수수료 계산:
-- - 수수료는 거래 금액의 일정 비율로 계산됩니다.
-- - 예: 수수료율 0.0001 (0.01%), 거래 금액 100 USDT
--   → 수수료 = 100 * 0.0001 = 0.01 USDT
-- 
-- 수수료 적용 방식:
-- - 매수자: 구매한 자산에서 수수료 차감 (예: SOL 구매 시 SOL에서 차감)
-- - 매도자: 판매 금액에서 수수료 차감 (예: USDT로 전환 시 USDT에서 차감)
-- 
-- 예시:
-- - SOL/USDT 거래, 수수료율 0.0001 (0.01%)
-- - 사용자 A가 SOL 1개를 100 USDT에 구매
--   → 매수자(A) 수수료: 100 * 0.0001 = 0.01 USDT 차감
--   → 사용자 A: SOL +1.0, USDT -100.01
-- 
-- 확장성:
-- - 현재는 모든 거래쌍에 0.0001 고정이지만
-- - 나중에 거래쌍별, 사용자 등급별로 다른 수수료 적용 가능
-- =====================================================

CREATE TABLE IF NOT EXISTS fee_configs (
    -- 기본 정보
    id BIGSERIAL PRIMARY KEY,  -- 수수료 설정 고유 ID
    
    -- 거래쌍 정보 (NULL이면 모든 거래쌍에 적용)
    -- base_mint: 기준 자산 (예: 'SOL', NULL이면 전체 적용)
    -- quote_mint: 기준 통화 (예: 'USDT', NULL이면 전체 적용)
    -- 예: base_mint='SOL', quote_mint='USDT' → SOL/USDT 거래에만 적용
    -- 예: base_mint=NULL, quote_mint=NULL → 모든 거래쌍에 적용
    base_mint VARCHAR(255),   -- 기준 자산 (NULL: 전체 적용)
    quote_mint VARCHAR(255),  -- 기준 통화 (NULL: 전체 적용)
    
    -- 수수료율 설정
    -- fee_rate: 거래 금액 대비 수수료 비율 (소수점)
    -- - 0.0001 = 0.01% 수수료
    -- - 0.0002 = 0.02% 수수료
    -- - 0.00015 = 0.015% 수수료
    -- DECIMAL(10, 6): 최대 9,999,999.999999% (충분히 큼)
    fee_rate DECIMAL(10, 6) NOT NULL DEFAULT 0.0001,  -- 수수료율 (기본 0.01%)
    
    -- 수수료 적용 대상 (나중에 확장 가능)
    -- fee_type: 수수료 유형
    -- - 'taker': 시장가 주문자 수수료
    -- - 'maker': 지정가 주문자 수수료
    -- - 'both': 모두 동일 (현재는 이 방식)
    -- 나중에 maker/taker 수수료를 다르게 설정할 수 있음
    fee_type VARCHAR(20) NOT NULL DEFAULT 'both' CHECK (fee_type IN ('taker', 'maker', 'both')),
    
    -- 활성화 여부
    -- is_active: 이 수수료 설정이 활성화되어 있는지
    -- false로 설정하면 이전 설정 유지, 새로운 거래에는 적용 안 됨
    is_active BOOLEAN NOT NULL DEFAULT TRUE,  -- 활성화 여부
    
    -- 타임스탬프
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- 수수료 설정 생성 시간
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()   -- 수수료 설정 마지막 업데이트 시간
);

-- 테이블 코멘트
COMMENT ON TABLE fee_configs IS '거래 수수료 설정 테이블 (거래쌍별 수수료율 관리)';

-- 컬럼 코멘트
COMMENT ON COLUMN fee_configs.id IS '수수료 설정 고유 ID';
COMMENT ON COLUMN fee_configs.base_mint IS '기준 자산 (NULL이면 모든 거래쌍에 적용, 예: SOL, USDC)';
COMMENT ON COLUMN fee_configs.quote_mint IS '기준 통화 (NULL이면 모든 거래쌍에 적용, 예: USDT)';
COMMENT ON COLUMN fee_configs.fee_rate IS '수수료율 (소수점, 예: 0.0001 = 0.01%, 기본값 0.01%)';
COMMENT ON COLUMN fee_configs.fee_type IS '수수료 유형: taker(시장가), maker(지정가), both(모두 동일)';
COMMENT ON COLUMN fee_configs.is_active IS '활성화 여부 (false면 이전 설정 유지, 새 거래에 미적용)';
COMMENT ON COLUMN fee_configs.created_at IS '수수료 설정 생성 시간';
COMMENT ON COLUMN fee_configs.updated_at IS '수수료 설정 마지막 업데이트 시간';

-- UNIQUE 제약 추가 (중복 방지)
-- 활성화된 설정에 대해서만 거래쌍 조합이 고유해야 함
-- (base_mint, quote_mint) 조합은 활성화된 설정에서 중복 불가
-- NULL 값 처리: COALESCE로 NULL을 ''로 변환하여 UNIQUE 체크
CREATE UNIQUE INDEX IF NOT EXISTS idx_fee_configs_unique_pair 
    ON fee_configs(COALESCE(base_mint, ''), COALESCE(quote_mint, ''), is_active) 
    WHERE is_active = TRUE;

-- 인덱스 생성 (성능 최적화)
-- 거래쌍별 수수료 조회 시 사용 (거래 발생 시 수수료율을 빠르게 조회)
CREATE INDEX IF NOT EXISTS idx_fee_configs_pair ON fee_configs(base_mint, quote_mint, is_active) 
    WHERE is_active = TRUE;

-- 활성화된 수수료 설정만 조회
CREATE INDEX IF NOT EXISTS idx_fee_configs_active ON fee_configs(is_active) 
    WHERE is_active = TRUE;

-- =====================================================
-- 초기 데이터 삽입 (모든 거래쌍에 0.01% 수수료 적용)
-- =====================================================
-- 설명: 거래소 시작 시 기본 수수료 설정을 추가합니다.
-- 모든 거래쌍에 0.01% (0.0001) 수수료를 적용합니다.
-- =====================================================

-- 초기 데이터 삽입 (중복 체크 포함)
-- 이미 (NULL, NULL, TRUE) 조합이 있으면 삽입하지 않음
INSERT INTO fee_configs (base_mint, quote_mint, fee_rate, fee_type, is_active)
SELECT NULL, NULL, 0.0001, 'both', TRUE
WHERE NOT EXISTS (
    SELECT 1 FROM fee_configs 
    WHERE base_mint IS NULL 
      AND quote_mint IS NULL 
      AND is_active = TRUE
);

