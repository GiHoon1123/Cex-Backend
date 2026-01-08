#!/bin/bash
# =====================================================
# 디스크 공간 정리 크론 설정 스크립트
# Disk Cleanup Cron Setup Script
# =====================================================
#
# 용도:
#   - 디스크 공간 정리 크론 작업을 설정합니다
#   - cleanup_disk_space.sh를 크론에 등록합니다
#   - 주기적으로 디스크 공간을 정리하여 여유 공간을 확보합니다
#
# 실행 시간:
#   - 한국 시간: 매일 새벽 2시
#   - UTC 시간: 매일 17시 (전날)
#
# 크론 작업:
#   - /usr/local/bin/cleanup_disk_space.sh 실행
#   - 로그: /home/ec2-user/logs/disk_cleanup.log
#
# 주요 기능 (cleanup_disk_space.sh):
#   1. 디스크 사용량 확인 (90% 이상 시 강제 데이터 삭제)
#   2. Docker 이미지 및 빌드 캐시 정리
#   3. PostgreSQL VACUUM ANALYZE 실행
#   4. PostgreSQL WAL 상태 확인
#   5. 시스템 패키지 캐시 정리
#
# 실행 방법:
#   - 한 번만 실행하면 크론에 등록됩니다
#   - ./scripts/setup_disk_cleanup_cron.sh
#
# 주의:
#   - cleanup_disk_space.sh가 /usr/local/bin/에 복사됩니다
#   - 기존 디스크 정리 크론 작업은 제거되고 새로 추가됩니다
# =====================================================
CRON_JOB="0 17 * * * /usr/local/bin/cleanup_disk_space.sh >> /home/ec2-user/logs/disk_cleanup.log 2>&1"
LOG_DIR="/home/ec2-user/logs"
SCRIPT_PATH="/usr/local/bin/cleanup_disk_space.sh"

# 로그 디렉토리 생성
mkdir -p "$LOG_DIR"

# 스크립트를 /usr/local/bin에 복사
if [ -f "scripts/cleanup_disk_space.sh" ]; then
    sudo cp scripts/cleanup_disk_space.sh "$SCRIPT_PATH"
    sudo chmod +x "$SCRIPT_PATH"
    echo "스크립트가 $SCRIPT_PATH 에 복사되었습니다."
else
    echo "오류: scripts/cleanup_disk_space.sh 파일을 찾을 수 없습니다."
    exit 1
fi

# 기존 크론 작업에서 해당 라인 제거 (중복 방지)
(crontab -l 2>/dev/null | grep -v "cleanup_disk_space.sh") | crontab -

# 새로운 크론 작업 추가 (매주 일요일 새벽 2시)
(crontab -l 2>/dev/null; echo "$CRON_JOB") | crontab -

echo "크론 작업이 성공적으로 설정되었습니다."
echo "매일 한국시간 새벽 2시 (UTC 17시)에 디스크 공간 정리가 실행됩니다."
echo "로그는 $LOG_DIR/disk_cleanup.log 에서 확인할 수 있습니다."

