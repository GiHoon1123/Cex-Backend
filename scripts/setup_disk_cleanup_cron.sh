#!/bin/bash
# 디스크 공간 정리 크론 작업을 설정합니다.
# 주기적으로 디스크 공간을 정리하여 여유 공간을 확보합니다.

# 한국 시간 새벽 2시 = UTC 17시 (전날)
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

