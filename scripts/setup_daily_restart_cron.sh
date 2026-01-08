#!/bin/bash
# =====================================================
# 컨테이너 재시작 크론 설정 스크립트
# Container Restart Cron Setup Script
# =====================================================
#
# 용도:
#   - 하루 3번 컨테이너 재시작 크론 작업을 설정합니다
#   - 컨테이너 재시작 시 서버 시작 로직에서 모든 테이블 데이터가 삭제됩니다
#
# 실행 시간:
#   - 한국 시간: 00:00, 08:00, 16:00 (하루 3번)
#   - UTC 시간: 15:00, 23:00, 07:00
#
# 크론 작업:
#   - docker restart cex-backend 실행
#   - 로그: /home/ec2-user/logs/container_restart.log
#
# 데이터 삭제:
#   - 컨테이너 재시작 시 bot_manager.rs의 prepare_bots()가 실행됨
#   - delete_all_bot_data_together()에서 모든 테이블 데이터 삭제
#   - 삭제되는 테이블: orders, trades, user_balances, transactions, 
#                     solana_wallets, refresh_tokens, users, fee_configs
#
# 실행 방법:
#   - 한 번만 실행하면 크론에 등록됩니다
#   - ./scripts/setup_daily_restart_cron.sh
#
# 주의:
#   - 기존 컨테이너 재시작 크론 작업은 모두 제거됩니다
#   - 새로운 3개의 크론 작업이 추가됩니다
# =====================================================

# 크론 작업 정의
CRON_JOB1="0 15 * * * docker restart cex-backend >> /home/ec2-user/logs/container_restart.log 2>&1"
CRON_JOB2="0 23 * * * docker restart cex-backend >> /home/ec2-user/logs/container_restart.log 2>&1"
CRON_JOB3="0 7 * * * docker restart cex-backend >> /home/ec2-user/logs/container_restart.log 2>&1"

# 현재 크론 작업 확인
echo "=== 현재 크론 작업 ==="
crontab -l 2>/dev/null || echo "크론 작업 없음"

# 기존 컨테이너 재시작 크론 제거 (있다면)
crontab -l 2>/dev/null | grep -v "docker restart cex-backend" | crontab - 2>/dev/null

# 새 크론 작업 추가 (3번)
(crontab -l 2>/dev/null; echo "$CRON_JOB1"; echo "$CRON_JOB2"; echo "$CRON_JOB3") | crontab -

echo ""
echo "=== 크론 작업 추가 완료 ==="
echo "하루 3번 cex-backend 컨테이너가 재시작됩니다:"
echo "  - UTC 15:00 (한국 시간 00:00)"
echo "  - UTC 23:00 (한국 시간 08:00)"
echo "  - UTC 07:00 (한국 시간 16:00)"
echo ""
echo "현재 크론 작업:"
crontab -l

