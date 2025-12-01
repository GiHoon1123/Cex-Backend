-- Users 테이블
-- 사용자 계정 정보 (로그인/회원가입)
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    username VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 테이블 코멘트
COMMENT ON TABLE users IS '사용자 계정 정보 테이블 (로그인/회원가입)';

-- 컬럼 코멘트
COMMENT ON COLUMN users.id IS '사용자 고유 ID';
COMMENT ON COLUMN users.email IS '이메일 주소 (유니크, 로그인 시 사용)';
COMMENT ON COLUMN users.password_hash IS '비밀번호 해시 (Argon2)';
COMMENT ON COLUMN users.username IS '사용자명 (선택사항)';
COMMENT ON COLUMN users.created_at IS '계정 생성 시간';
COMMENT ON COLUMN users.updated_at IS '계정 정보 수정 시간';

-- 인덱스 추가 (이메일로 빠른 조회)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

