-- =====================================================
-- trades 테이블의 buy_order_id, sell_order_id에 CASCADE 추가
-- =====================================================
-- 설명: orders 테이블의 데이터를 삭제할 때 관련된 trades 데이터도 자동으로 삭제되도록 합니다.
-- 
-- 변경 사항:
-- - buy_order_id 외래키에 ON DELETE CASCADE 추가
-- - sell_order_id 외래키에 ON DELETE CASCADE 추가
-- 
-- 효과:
-- - orders 삭제 시 → 관련된 trades도 자동 삭제
-- - 봇 데이터 정리 시 orders 삭제가 성공적으로 수행됨
-- =====================================================

-- 기존 제약조건 삭제
ALTER TABLE trades 
    DROP CONSTRAINT IF EXISTS trades_buy_order_id_fkey,
    DROP CONSTRAINT IF EXISTS trades_sell_order_id_fkey;

-- CASCADE가 포함된 새로운 제약조건 추가
ALTER TABLE trades
    ADD CONSTRAINT trades_buy_order_id_fkey 
        FOREIGN KEY (buy_order_id) 
        REFERENCES orders(id) 
        ON DELETE CASCADE,
    ADD CONSTRAINT trades_sell_order_id_fkey 
        FOREIGN KEY (sell_order_id) 
        REFERENCES orders(id) 
        ON DELETE CASCADE;

-- 완료 메시지
COMMENT ON CONSTRAINT trades_buy_order_id_fkey ON trades IS '매수 주문 ID 외래키 (orders 삭제 시 CASCADE 삭제)';
COMMENT ON CONSTRAINT trades_sell_order_id_fkey ON trades IS '매도 주문 ID 외래키 (orders 삭제 시 CASCADE 삭제)';

