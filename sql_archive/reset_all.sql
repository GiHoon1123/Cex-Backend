-- =====================================================
-- RESET ALL TABLES (개발용)
-- =====================================================
-- 경고: 이 스크립트는 모든 데이터를 삭제합니다!
-- WARNING: This script will delete all data!
-- 
-- 사용법:
-- psql -U solana_user -d solana_dev < migrations/000_reset_all.sql
-- =====================================================

-- 1. SQLx 마이그레이션 테이블 삭제 (가장 중요!)
-- Drop SQLx migration tracking table (MOST IMPORTANT!)
DROP TABLE IF EXISTS _sqlx_migrations CASCADE;

-- 2. 모든 테이블 삭제 (순서 중요 - 외래키 때문에)
-- Drop all tables (order matters due to foreign keys)

DROP TABLE IF EXISTS cex_trades CASCADE;
DROP TABLE IF EXISTS cex_orders CASCADE;
DROP TABLE IF EXISTS cex_fee_configs CASCADE;
DROP TABLE IF EXISTS cex_user_balances CASCADE;

DROP TABLE IF EXISTS swap_tokens CASCADE;
DROP TABLE IF EXISTS swap_routes CASCADE;

DROP TABLE IF EXISTS user_wallets CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;

DROP TABLE IF EXISTS refresh_tokens CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- 3. 시퀀스 삭제 (ID 자동 증가)
-- Drop sequences

DROP SEQUENCE IF EXISTS users_id_seq CASCADE;
DROP SEQUENCE IF EXISTS user_wallets_id_seq CASCADE;
DROP SEQUENCE IF EXISTS transactions_id_seq CASCADE;
DROP SEQUENCE IF EXISTS refresh_tokens_id_seq CASCADE;
DROP SEQUENCE IF EXISTS cex_user_balances_id_seq CASCADE;
DROP SEQUENCE IF EXISTS cex_orders_id_seq CASCADE;
DROP SEQUENCE IF EXISTS cex_trades_id_seq CASCADE;
DROP SEQUENCE IF EXISTS cex_fee_configs_id_seq CASCADE;
DROP SEQUENCE IF EXISTS swap_tokens_id_seq CASCADE;
DROP SEQUENCE IF EXISTS swap_routes_id_seq CASCADE;

-- 4. 타입 삭제 (ENUM 타입들)
-- Drop custom types

DROP TYPE IF EXISTS order_type CASCADE;
DROP TYPE IF EXISTS order_side CASCADE;
DROP TYPE IF EXISTS order_status CASCADE;
DROP TYPE IF EXISTS transaction_type CASCADE;
DROP TYPE IF EXISTS transaction_status CASCADE;

-- 5. 인덱스는 CASCADE로 테이블 삭제 시 자동 삭제됨
-- Indexes are automatically dropped with CASCADE

-- =====================================================
-- 완료! 이제 다시 실행하세요:
-- Done! Now run again:
--   cargo run
-- 
-- 모든 마이그레이션이 처음부터 다시 실행됩니다.
-- All migrations will be re-run from scratch.
-- =====================================================

