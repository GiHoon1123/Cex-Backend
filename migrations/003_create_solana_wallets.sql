-- Solana Wallets 테이블
-- 사용자 Solana 지갑 정보 (서버에서 키 관리)
-- Note: FK 제약조건 없음, 논리적 관계만 유지
CREATE TABLE IF NOT EXISTS solana_wallets (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    public_key VARCHAR(255) UNIQUE NOT NULL,
    encrypted_private_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 테이블 코멘트
COMMENT ON TABLE solana_wallets IS '사용자 Solana 지갑 정보 테이블 (서버에서 키 관리)';

-- 컬럼 코멘트
COMMENT ON COLUMN solana_wallets.id IS '지갑 고유 ID';
COMMENT ON COLUMN solana_wallets.user_id IS '소유자 사용자 ID (users 테이블 참조, 논리적 관계)';
COMMENT ON COLUMN solana_wallets.public_key IS 'Solana 지갑 Public Key (유니크)';
COMMENT ON COLUMN solana_wallets.encrypted_private_key IS '암호화된 Private Key (Base64 인코딩)';
COMMENT ON COLUMN solana_wallets.created_at IS '지갑 생성 시간';
COMMENT ON COLUMN solana_wallets.updated_at IS '지갑 정보 수정 시간';

-- 인덱스 추가
CREATE INDEX IF NOT EXISTS idx_solana_wallets_user_id ON solana_wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_solana_wallets_public_key ON solana_wallets(public_key);

