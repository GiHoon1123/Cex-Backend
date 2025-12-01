-- =====================================================
-- trades 테이블에 CASCADE 제약조건 추가
-- =====================================================
-- 설명: users 테이블의 데이터를 삭제할 때 관련된 trades 데이터도 자동으로 삭제되도록 합니다.
-- 
-- 변경 사항:
-- - buyer_id 외래키에 ON DELETE CASCADE 추가
-- - seller_id 외래키에 ON DELETE CASCADE 추가
-- 
-- 효과:
-- - users 삭제 시 → trades도 자동 삭제
-- - orders 삭제 시 → trades도 자동 삭제 (이미 CASCADE 있음)
-- =====================================================

-- 기존 제약조건 삭제
ALTER TABLE trades 
    DROP CONSTRAINT IF EXISTS trades_buyer_id_fkey,
    DROP CONSTRAINT IF EXISTS trades_seller_id_fkey;

-- CASCADE가 포함된 새로운 제약조건 추가
ALTER TABLE trades
    ADD CONSTRAINT trades_buyer_id_fkey 
        FOREIGN KEY (buyer_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE,
    ADD CONSTRAINT trades_seller_id_fkey 
        FOREIGN KEY (seller_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE;

-- 완료 메시지
COMMENT ON CONSTRAINT trades_buyer_id_fkey ON trades IS '매수자 사용자 ID 외래키 (users 삭제 시 CASCADE 삭제)';
COMMENT ON CONSTRAINT trades_seller_id_fkey ON trades IS '매도자 사용자 ID 외래키 (users 삭제 시 CASCADE 삭제)';

