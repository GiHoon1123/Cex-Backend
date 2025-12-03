-- =====================================================
-- 체결 내역 테이블 (trades)
-- =====================================================
-- 설명: 주문이 매칭되어 실제로 거래가 발생한 내역을 저장합니다.
-- 
-- 체결 과정:
-- 1. 사용자 A가 매수 주문 생성 (100 USDT에 SOL 1개 구매)
-- 2. 사용자 B가 매도 주문 생성 (100 USDT에 SOL 1개 판매)
-- 3. 가격이 일치하므로 매칭 → 체결 발생
-- 4. trades 테이블에 체결 내역 기록
-- 5. 각 사용자의 잔고 업데이트
-- 
-- 예시:
-- - buy_order_id: 사용자 A의 매수 주문 ID
-- - sell_order_id: 사용자 B의 매도 주문 ID
-- - price: 100.0 (체결 가격)
-- - amount: 1.0 (체결 수량)
-- 
-- 봇 주문과의 체결:
-- - 봇 주문과 사용자 주문이 매칭되어 체결되는 경우도 일반 체결과 동일하게 기록됨
-- - buy_order_id 또는 sell_order_id 중 하나가 봇 주문일 수 있음
-- =====================================================

CREATE TABLE IF NOT EXISTS trades (
    -- 기본 정보
    id BIGSERIAL PRIMARY KEY,  -- 체결 내역 고유 ID
    
    -- 체결된 주문 정보
    -- buy_order_id: 매수 주문 ID (누가 구매했는지)
    -- sell_order_id: 매도 주문 ID (누가 판매했는지)
    -- 주의: 봇 주문과 사용자 주문 모두 동일하게 참조됨
    -- CASCADE: orders 삭제 시 관련 trades도 자동 삭제
    buy_order_id BIGINT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,   -- 매수 주문 ID
    sell_order_id BIGINT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,  -- 매도 주문 ID
    
    -- 체결 참여자 정보 (주문 테이블을 다시 조회하지 않고 바로 사용하기 위함)
    -- CASCADE: users 삭제 시 관련 trades도 자동 삭제
    buyer_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,   -- 매수자 사용자 ID
    seller_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- 매도자 사용자 ID
    
    -- 거래쌍 정보 (주문에서 복사)
    base_mint VARCHAR(255) NOT NULL,   -- 기준 자산 (예: SOL)
    quote_mint VARCHAR(255) NOT NULL,  -- 기준 통화 (예: USDT)
    
    -- 체결 정보
    -- price: 실제 체결된 가격 (USDT 기준)
    -- amount: 체결된 수량 (base_mint 기준)
    -- 예: SOL 1개를 100 USDT에 체결 → price=100.0, amount=1.0
    price DECIMAL(30, 9) NOT NULL,   -- 체결 가격 (USDT)
    amount DECIMAL(30, 9) NOT NULL,  -- 체결 수량 (base_mint)
    
    -- 타임스탬프
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- 체결 발생 시간
);

-- 테이블 코멘트
COMMENT ON TABLE trades IS '체결 내역 테이블 (주문 매칭 후 실제 거래 발생 내역, 봇 주문 포함)';

-- 컬럼 코멘트
COMMENT ON COLUMN trades.id IS '체결 내역 고유 ID';
COMMENT ON COLUMN trades.buy_order_id IS '매수 주문 ID (누가 구매했는지, 봇 주문 포함, orders 삭제 시 CASCADE 삭제)';
COMMENT ON COLUMN trades.sell_order_id IS '매도 주문 ID (누가 판매했는지, 봇 주문 포함, orders 삭제 시 CASCADE 삭제)';
COMMENT ON COLUMN trades.buyer_id IS '매수자 사용자 ID (orders.user_id와 동일, 빠른 조회용, users 삭제 시 CASCADE 삭제)';
COMMENT ON COLUMN trades.seller_id IS '매도자 사용자 ID (orders.user_id와 동일, 빠른 조회용, users 삭제 시 CASCADE 삭제)';
COMMENT ON COLUMN trades.base_mint IS '거래된 자산 (SOL, USDC 등)';
COMMENT ON COLUMN trades.quote_mint IS '기준 통화 (항상 USDT)';
COMMENT ON COLUMN trades.price IS '체결 가격 (USDT 기준, 예: SOL 1개 = 100 USDT)';
COMMENT ON COLUMN trades.amount IS '체결 수량 (base_mint 기준, 예: SOL 1.0개)';
COMMENT ON COLUMN trades.created_at IS '체결 발생 시간';

-- 인덱스 생성 (성능 최적화)
-- 거래쌍별 체결 내역 조회 (거래 내역 페이지에서 사용)
-- 시간순으로 정렬하여 최신 거래부터 보여줄 수 있음
CREATE INDEX IF NOT EXISTS idx_trades_pair_time ON trades(base_mint, quote_mint, created_at DESC);

-- 특정 주문의 체결 내역 조회 (주문 상세 페이지에서 사용)
-- 사용자가 자신의 주문이 언제, 얼마나 체결되었는지 확인할 때 사용
CREATE INDEX IF NOT EXISTS idx_trades_buy_order ON trades(buy_order_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_trades_sell_order ON trades(sell_order_id, created_at DESC);

-- 사용자별 체결 내역 조회
CREATE INDEX IF NOT EXISTS idx_trades_buyer ON trades(buyer_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_trades_seller ON trades(seller_id, created_at DESC);

-- 제약조건 코멘트 (CASCADE 설정 명시)
COMMENT ON CONSTRAINT trades_buy_order_id_fkey ON trades IS '매수 주문 ID 외래키 (orders 삭제 시 CASCADE 삭제)';
COMMENT ON CONSTRAINT trades_sell_order_id_fkey ON trades IS '매도 주문 ID 외래키 (orders 삭제 시 CASCADE 삭제)';
COMMENT ON CONSTRAINT trades_buyer_id_fkey ON trades IS '매수자 사용자 ID 외래키 (users 삭제 시 CASCADE 삭제)';
COMMENT ON CONSTRAINT trades_seller_id_fkey ON trades IS '매도자 사용자 ID 외래키 (users 삭제 시 CASCADE 삭제)';

