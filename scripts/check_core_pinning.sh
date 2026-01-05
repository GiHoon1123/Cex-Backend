#!/bin/bash
# 코어 고정 및 우선순위 확인 스크립트

echo "=== 서버 CPU 정보 ==="
nproc
lscpu | grep -E 'CPU\(s\)|Thread|Core|Socket'
echo ""

echo "=== Docker 컨테이너 상태 ==="
docker ps | grep cex-backend
echo ""

echo "=== 컨테이너 내 프로세스 및 스레드 ==="
CONTAINER_ID=$(docker ps -q -f name=cex-backend)
if [ -z "$CONTAINER_ID" ]; then
    echo "❌ cex-backend 컨테이너가 실행 중이 아닙니다."
    exit 1
fi

echo "컨테이너 ID: $CONTAINER_ID"
echo ""

echo "=== 메인 프로세스 PID ==="
MAIN_PID=$(docker exec $CONTAINER_ID ps aux | grep cex-backend | grep -v grep | awk '{print $2}' | head -1)
if [ -z "$MAIN_PID" ]; then
    echo "❌ cex-backend 프로세스를 찾을 수 없습니다."
    exit 1
fi
echo "Main PID: $MAIN_PID"
echo ""

echo "=== 모든 스레드의 CPU 할당 확인 ==="
echo "PID | TID | CPU | PRI | CMD"
echo "----|-----|-----|-----|-----"
docker exec $CONTAINER_ID sh -c "ps -eLf | grep $MAIN_PID | grep -v grep" | while read line; do
    PID=$(echo $line | awk '{print $2}')
    TID=$(echo $line | awk '{print $4}')
    CPU=$(echo $line | awk '{print $6}')
    PRI=$(echo $line | awk '{print $7}')
    CMD=$(echo $line | awk '{for(i=8;i<=NF;i++) printf "%s ", $i; print ""}')
    echo "$PID | $TID | $CPU | $PRI | $CMD"
done
echo ""

echo "=== taskset으로 CPU 할당 확인 ==="
docker exec $CONTAINER_ID taskset -cp $MAIN_PID 2>/dev/null || echo "taskset 명령어 사용 불가 (컨테이너 내부)"
echo ""

echo "=== chrt로 우선순위 확인 ==="
docker exec $CONTAINER_ID chrt -p $MAIN_PID 2>/dev/null || echo "chrt 명령어 사용 불가 (컨테이너 내부)"
echo ""

echo "=== 컨테이너 로그에서 코어 고정 메시지 확인 ==="
docker logs $CONTAINER_ID 2>&1 | grep -E "CoreConfig|Engine Thread|WAL Thread|Core|priority|scheduling" | tail -20
echo ""

echo "=== CPU 사용률 확인 (5초간) ==="
docker stats --no-stream $CONTAINER_ID | head -2
echo ""

echo "✅ 확인 완료"
echo ""
echo "예상 결과:"
echo "- 엔진 스레드: Core 0에 고정, 우선순위 99 (SCHED_FIFO)"
echo "- WAL 스레드: 코어 고정 안 함 (OS 스케줄링)"
echo "- 로그에 'Pinning to Core 0', 'priority: 99' 메시지 확인"

