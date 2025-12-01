-- Migration: Add filled_quote_amount column to orders table
-- 설명: 주문의 체결 금액(USDT)을 저장하기 위한 컬럼 추가
-- 시장가 주문의 경우 여러 가격으로 체결될 수 있으므로, 총 체결 금액을 저장

ALTER TABLE orders
ADD COLUMN IF NOT EXISTS filled_quote_amount DECIMAL(20, 8) NOT NULL DEFAULT 0;

-- 기존 주문의 filled_quote_amount는 0으로 유지 (과거 데이터는 계산 불가)
-- 새로운 체결부터 filled_quote_amount가 업데이트됨

COMMENT ON COLUMN orders.filled_quote_amount IS '체결된 금액 (USDT 기준, 시장가 주문의 총 결제 금액)';

