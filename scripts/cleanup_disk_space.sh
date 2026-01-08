#!/bin/bash
# 디스크 공간 정리 스크립트
# 주기적으로 실행하여 디스크 공간을 확보합니다.
# 디스크 사용량이 90% 이상이면 강제로 데이터를 삭제합니다.

LOG_FILE="/home/ec2-user/logs/disk_cleanup.log"
LOG_DIR="/home/ec2-user/logs"

# 로그 디렉토리 생성
mkdir -p "$LOG_DIR"

# 로그 함수
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# 디스크 사용량 확인 함수
get_disk_usage() {
    df -h / | tail -1 | awk '{print $5}' | sed 's/%//'
}

# 강제 데이터 삭제 함수 (디스크가 90% 이상일 때)
force_delete_data() {
    log "⚠️ 디스크 사용량이 90% 이상입니다. 강제 데이터 삭제를 시작합니다..."
    
    # PostgreSQL에 직접 연결하여 모든 테이블 데이터 삭제
    log "1. PostgreSQL 데이터베이스에 직접 연결하여 데이터 삭제 중..."
    
    # 컨테이너가 실행 중인지 확인
    if ! docker ps | grep -q cex-postgres; then
        log "❌ PostgreSQL 컨테이너가 실행 중이 아닙니다. 컨테이너를 시작합니다..."
        docker start cex-postgres
        sleep 5
    fi
    
    # PostgreSQL 연결 대기
    for i in {1..30}; do
        if docker exec cex-postgres pg_isready -U root > /dev/null 2>&1; then
            break
        fi
        log "PostgreSQL 연결 대기 중... ($i/30)"
        sleep 1
    done
    
    # 모든 테이블 데이터 삭제 (마이그레이션 테이블 제외)
    log "2. 모든 테이블 데이터 삭제 중..."
    docker exec cex-postgres psql -U root -d cex <<EOF 2>&1 | tee -a "$LOG_FILE"
-- 외래키 제약조건을 일시적으로 비활성화 (삭제 순서 문제 해결)
SET session_replication_role = 'replica';

-- 모든 테이블 데이터 삭제 (마이그레이션 테이블 제외)
DELETE FROM trades;
DELETE FROM orders;
DELETE FROM user_balances;
DELETE FROM transactions;
DELETE FROM solana_wallets;
DELETE FROM refresh_tokens;
DELETE FROM users;
DELETE FROM fee_configs;

-- 외래키 제약조건 다시 활성화
SET session_replication_role = 'origin';

-- VACUUM FULL 실행 (공간 회수)
VACUUM FULL;
EOF
    
    if [ $? -eq 0 ]; then
        log "✅ 강제 데이터 삭제 완료"
    else
        log "❌ 강제 데이터 삭제 실패"
    fi
}

log "=== 디스크 공간 정리 시작 ==="

# 디스크 사용량 확인
DISK_USAGE=$(get_disk_usage)
log "현재 디스크 사용량: ${DISK_USAGE}%"

# 디스크 사용량이 90% 이상이면 강제 데이터 삭제
if [ "$DISK_USAGE" -ge 90 ]; then
    log "⚠️ 경고: 디스크 사용량이 ${DISK_USAGE}%입니다 (90% 이상)"
    force_delete_data
    
    # 삭제 후 다시 디스크 사용량 확인
    DISK_USAGE=$(get_disk_usage)
    log "데이터 삭제 후 디스크 사용량: ${DISK_USAGE}%"
    
    # 여전히 90% 이상이면 컨테이너 재시작 시도
    if [ "$DISK_USAGE" -ge 90 ]; then
        log "⚠️ 여전히 디스크 사용량이 높습니다. 컨테이너 재시작을 시도합니다..."
        docker restart cex-backend 2>&1 | tee -a "$LOG_FILE"
        sleep 10
        DISK_USAGE=$(get_disk_usage)
        log "컨테이너 재시작 후 디스크 사용량: ${DISK_USAGE}%"
    fi
fi

# 1. Docker 사용하지 않는 이미지 삭제
log "1. Docker 사용하지 않는 이미지 삭제 중..."
DOCKER_IMAGES_REMOVED=$(docker image prune -f 2>&1)
log "Docker 이미지 정리 결과: $DOCKER_IMAGES_REMOVED"

# 2. Docker 빌드 캐시 정리 (7일 이상 된 캐시)
log "2. Docker 빌드 캐시 정리 중..."
DOCKER_BUILD_CACHE_REMOVED=$(docker builder prune -af --filter "until=168h" 2>&1)
log "Docker 빌드 캐시 정리 결과: $DOCKER_BUILD_CACHE_REMOVED"

# 3. PostgreSQL VACUUM (데이터베이스 최적화 및 공간 회수)
log "3. PostgreSQL VACUUM 실행 중..."
docker exec cex-postgres psql -U root -d cex -c "VACUUM ANALYZE;" 2>&1 | tee -a "$LOG_FILE"
if [ $? -eq 0 ]; then
    log "PostgreSQL VACUUM 완료"
else
    log "PostgreSQL VACUUM 실패"
fi

# 4. PostgreSQL 오래된 WAL 파일 정리 (자동으로 관리되지만 확인)
log "4. PostgreSQL WAL 상태 확인..."
WAL_SIZE=$(docker exec cex-postgres psql -U root -d cex -c "SELECT pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), '0/0'));" 2>/dev/null | tail -n +3 | head -n 1 | tr -d ' ')
if [ -n "$WAL_SIZE" ] && [ "$WAL_SIZE" != "" ]; then
    log "PostgreSQL WAL 크기: $WAL_SIZE"
else
    log "PostgreSQL WAL 크기 확인 실패 (자동 관리됨)"
fi

# 5. 시스템 패키지 캐시 정리 (dnf/yum 캐시)
log "5. 시스템 패키지 캐시 정리 중..."
if command -v dnf &> /dev/null; then
    DNF_CACHE_CLEANED=$(sudo dnf clean all 2>&1)
    log "DNF 캐시 정리 결과: $DNF_CACHE_CLEANED"
elif command -v yum &> /dev/null; then
    YUM_CACHE_CLEANED=$(sudo yum clean all 2>&1)
    log "YUM 캐시 정리 결과: $YUM_CACHE_CLEANED"
fi

# 6. 디스크 사용량 확인 (정리 후)
log "6. 정리 후 디스크 사용량:"
df -h / | tail -1 | tee -a "$LOG_FILE"

log "=== 디스크 공간 정리 완료 ==="

