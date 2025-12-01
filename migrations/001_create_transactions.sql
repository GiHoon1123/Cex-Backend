-- Transactions 테이블
-- 스왑 트랜잭션 정보 저장
CREATE TABLE IF NOT EXISTS transactions (
    id BIGSERIAL PRIMARY KEY,
    input_mint VARCHAR(255) NOT NULL,
    output_mint VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL,
    expected_out_amount BIGINT,
    user_public_key VARCHAR(255) NOT NULL,
    transaction_bytes TEXT NOT NULL,
    quote_response JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'created',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 테이블 코멘트
COMMENT ON TABLE transactions IS '스왑 트랜잭션 정보 저장 테이블';

-- 컬럼 코멘트
COMMENT ON COLUMN transactions.id IS '트랜잭션 고유 ID';
COMMENT ON COLUMN transactions.input_mint IS '입력 토큰 주소 (Mint 주소)';
COMMENT ON COLUMN transactions.output_mint IS '출력 토큰 주소 (Mint 주소)';
COMMENT ON COLUMN transactions.amount IS '입력 토큰 수량 (최소 단위)';
COMMENT ON COLUMN transactions.expected_out_amount IS '예상 출력 토큰 수량 (최소 단위)';
COMMENT ON COLUMN transactions.user_public_key IS '사용자 지갑 Public Key';
COMMENT ON COLUMN transactions.transaction_bytes IS '트랜잭션 바이트 (Base64 인코딩)';
COMMENT ON COLUMN transactions.quote_response IS 'Jupiter Quote API 응답 (JSON)';
COMMENT ON COLUMN transactions.status IS '트랜잭션 상태 (created, sent, confirmed, failed)';
COMMENT ON COLUMN transactions.created_at IS '트랜잭션 생성 시간';

