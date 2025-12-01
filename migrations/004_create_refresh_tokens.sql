-- Refresh Tokens 테이블 생성
-- Refresh Token 저장 및 관리
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked BOOLEAN NOT NULL DEFAULT FALSE
);

-- 테이블 코멘트
COMMENT ON TABLE refresh_tokens IS 'JWT Refresh Token 저장 및 관리 테이블';

-- 컬럼 코멘트
COMMENT ON COLUMN refresh_tokens.id IS 'Refresh Token 고유 ID';
COMMENT ON COLUMN refresh_tokens.user_id IS '사용자 ID (users 테이블 외래키, CASCADE 삭제)';
COMMENT ON COLUMN refresh_tokens.token_hash IS '해싱된 Refresh Token (SHA256, 유니크)';
COMMENT ON COLUMN refresh_tokens.expires_at IS '토큰 만료 시간 (기본 7일)';
COMMENT ON COLUMN refresh_tokens.created_at IS '토큰 생성 시간';
COMMENT ON COLUMN refresh_tokens.updated_at IS '토큰 정보 수정 시간';
COMMENT ON COLUMN refresh_tokens.revoked IS '토큰 무효화 여부 (로그아웃 시 true)';

-- 인덱스 생성 (조회 성능 향상)
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

-- 만료된 토큰 자동 정리를 위한 인덱스 (선택사항)
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at_revoked ON refresh_tokens(expires_at, revoked);

