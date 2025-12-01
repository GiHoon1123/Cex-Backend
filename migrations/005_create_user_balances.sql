-- =====================================================
-- 사용자 자산 잔고 테이블 (user_balances)
-- =====================================================
-- 설명: 사용자가 보유한 각 자산(SOL, USDT 등)의 잔고를 관리합니다.
-- 거래소에서는 모든 거래가 DB에서 관리되므로 실제 블록체인 잔고와 별도로 관리됩니다.
-- 
-- 잔고 구분:
-- - available: 사용 가능한 잔고 (즉시 거래/출금 가능)
-- - locked: 주문에 사용 중인 잔고 (주문 취소 또는 체결 시 해제됨)
-- 
-- 예시:
-- - 사용자 1이 SOL 10개, USDT 1000개 보유
--   → user_id=1, mint_address='SOL', available=10.0, locked=0
--   → user_id=1, mint_address='USDT', available=1000.0, locked=0
-- 
-- - 사용자 1이 SOL 1개를 100 USDT에 매도 주문 생성
--   → SOL: available=9.0, locked=1.0 (주문에 잠김)
-- 
-- - 주문이 체결되면
--   → SOL: available=9.0, locked=0 (잠금 해제)
--   → USDT: available=1100.0, locked=0 (USDT 증가)
-- =====================================================

CREATE TABLE IF NOT EXISTS user_balances (
    -- 기본 정보
    id BIGSERIAL PRIMARY KEY,  -- 잔고 레코드 고유 ID
    
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- 사용자 ID (외래키)
    
    -- 자산 정보
    -- mint_address: 자산의 고유 식별자
    -- - 'SOL' (문자열로 저장, 네이티브 SOL)
    -- - SPL 토큰의 경우 실제 mint 주소 (예: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' - USDC)
    -- - 'USDT' (문자열로 저장, USD Tether)
    mint_address VARCHAR(255) NOT NULL,
    
    -- 잔고 정보
    -- available: 사용 가능한 잔고 (즉시 거래/출금 가능)
    -- locked: 주문에 사용 중인 잔고 (주문 취소 또는 체결 시 해제됨)
    available DECIMAL(30, 9) NOT NULL DEFAULT 0,  -- 사용 가능 잔고
    locked DECIMAL(30, 9) NOT NULL DEFAULT 0,     -- 주문 중 잠긴 잔고
    
    -- 타임스탬프
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- 잔고 레코드 생성 시간
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- 잔고 업데이트 시간
    
    -- 제약조건: 한 사용자는 같은 자산을 하나의 레코드로만 관리
    -- 예: 사용자 1의 SOL 잔고는 하나의 레코드만 존재
    UNIQUE(user_id, mint_address)
);

-- 테이블 코멘트
COMMENT ON TABLE user_balances IS '사용자 자산 잔고 관리 테이블 (CEX 거래소 잔고 관리)';

-- 컬럼 코멘트
COMMENT ON COLUMN user_balances.id IS '잔고 레코드 고유 ID';
COMMENT ON COLUMN user_balances.user_id IS '사용자 ID (users 테이블 외래키, 사용자 삭제 시 CASCADE 삭제)';
COMMENT ON COLUMN user_balances.mint_address IS '자산 식별자 (SOL, USDT, 또는 SPL 토큰 mint 주소)';
COMMENT ON COLUMN user_balances.available IS '사용 가능한 잔고 (거래/출금 즉시 사용 가능)';
COMMENT ON COLUMN user_balances.locked IS '주문에 잠긴 잔고 (주문이 체결되거나 취소되면 available로 이동)';
COMMENT ON COLUMN user_balances.created_at IS '잔고 레코드 생성 시간';
COMMENT ON COLUMN user_balances.updated_at IS '잔고 정보 마지막 업데이트 시간';

-- 인덱스 생성 (성능 최적화)
-- 사용자 ID로 빠른 잔고 조회 (사용자 잔고 조회 시 사용)
CREATE INDEX IF NOT EXISTS idx_user_balances_user_id ON user_balances(user_id);
-- 자산별 잔고 집계 시 사용 (특정 자산을 보유한 모든 사용자 조회)
CREATE INDEX IF NOT EXISTS idx_user_balances_mint_address ON user_balances(mint_address);

