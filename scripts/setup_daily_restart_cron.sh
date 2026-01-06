#!/bin/bash

# 매일 밤 11시 (한국 시간) 컨테이너 재시작 크론 설정
# UTC 14시 = 한국 시간 23시

CRON_JOB="0 14 * * * /usr/bin/docker restart cex-backend >> /home/ec2-user/logs/container_restart.log 2>&1"

# 현재 크론 작업 확인
echo "=== 현재 크론 작업 ==="
crontab -l 2>/dev/null || echo "크론 작업 없음"

# 기존 컨테이너 재시작 크론 제거 (있다면)
crontab -l 2>/dev/null | grep -v "docker restart cex-backend" | crontab - 2>/dev/null

# 새 크론 작업 추가
(crontab -l 2>/dev/null; echo "$CRON_JOB") | crontab -

echo ""
echo "=== 크론 작업 추가 완료 ==="
echo "매일 UTC 14시 (한국 시간 23시)에 cex-backend 컨테이너가 재시작됩니다."
echo ""
echo "현재 크론 작업:"
crontab -l

