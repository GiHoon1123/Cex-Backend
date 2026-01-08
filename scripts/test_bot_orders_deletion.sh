#!/bin/bash
# 봇 orders 삭제 로직 테스트 스크립트

set -e

DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-root}"
DB_PASSWORD="${DB_PASSWORD:-1234}"
DB_NAME="${DB_NAME:-cex}"

echo "=== 봇 Orders 삭제 테스트 ==="
echo "DB: ${DB_HOST}:${DB_PORT}/${DB_NAME}"
echo ""

# 봇 user_id 조회
BOT1_ID=$(docker exec dustin-postgres psql -U ${DB_USER} -d ${DB_NAME} -t -c "
SELECT id FROM users WHERE email = 'bot1@bot.com';
" 2>/dev/null | tr -d ' ')

BOT2_ID=$(docker exec dustin-postgres psql -U ${DB_USER} -d ${DB_NAME} -t -c "
SELECT id FROM users WHERE email = 'bot2@bot.com';
" 2>/dev/null | tr -d ' ')

if [ -z "${BOT1_ID}" ] || [ -z "${BOT2_ID}" ]; then
    echo "❌ 봇 계정을 찾을 수 없습니다!"
    exit 1
fi

echo "봇1 ID: ${BOT1_ID}"
echo "봇2 ID: ${BOT2_ID}"
echo ""

# 1. 삭제 전 봇 orders 개수 확인
echo "1. 삭제 전 봇 orders 개수 확인..."
BOT_ORDERS_BEFORE=$(docker exec dustin-postgres psql -U ${DB_USER} -d ${DB_NAME} -t -c "
SELECT COUNT(*) FROM orders 
WHERE user_id IN (${BOT1_ID}, ${BOT2_ID});
" 2>/dev/null | tr -d ' ')

echo "   봇 orders 개수: ${BOT_ORDERS_BEFORE}"
echo ""

# 2. 일반 사용자와 거래한 봇 orders 개수 확인 (보존되어야 함)
echo "2. 일반 사용자와 거래한 봇 orders 개수 확인 (보존 대상)..."
PRESERVED_ORDERS=$(docker exec dustin-postgres psql -U ${DB_USER} -d ${DB_NAME} -t -c "
SELECT COUNT(DISTINCT o.id) FROM orders o
JOIN trades t ON (o.id = t.buy_order_id OR o.id = t.sell_order_id)
WHERE o.user_id IN (${BOT1_ID}, ${BOT2_ID})
AND (t.buyer_id != ALL(ARRAY[${BOT1_ID}, ${BOT2_ID}]) 
     OR t.seller_id != ALL(ARRAY[${BOT1_ID}, ${BOT2_ID}]));
" 2>/dev/null | tr -d ' ')

echo "   보존되어야 할 orders 개수: ${PRESERVED_ORDERS}"
echo ""

# 3. 삭제 예상 개수 계산
EXPECTED_DELETED=$((BOT_ORDERS_BEFORE - PRESERVED_ORDERS))
echo "3. 삭제 예상 개수: ${EXPECTED_DELETED}"
echo ""

# 4. 실제 삭제 쿼리 실행 (트랜잭션으로 롤백하여 테스트)
echo "4. 삭제 쿼리 실행 중 (트랜잭션 롤백으로 테스트)..."
DELETED_COUNT=$(docker exec dustin-postgres psql -U ${DB_USER} -d ${DB_NAME} -t -A -c "
BEGIN;
WITH deleted AS (
    DELETE FROM orders
    WHERE user_id = ANY(ARRAY[${BOT1_ID}, ${BOT2_ID}])
    AND NOT EXISTS (
        SELECT 1 FROM trades
        WHERE (buy_order_id = orders.id OR sell_order_id = orders.id)
        AND (buyer_id != ALL(ARRAY[${BOT1_ID}, ${BOT2_ID}]) 
             OR seller_id != ALL(ARRAY[${BOT1_ID}, ${BOT2_ID}]))
    )
    RETURNING id
)
SELECT COUNT(*) FROM deleted;
ROLLBACK;
" 2>/dev/null | grep -E "^[0-9]+$" | head -1)

echo "   실제 삭제될 개수 (테스트): ${DELETED_COUNT}"
echo ""

# 5. 실제 삭제 실행 여부 확인
echo "5. 실제 삭제를 실행하시겠습니까? (y/n)"
read -r CONFIRM

if [ "${CONFIRM}" != "y" ] && [ "${CONFIRM}" != "Y" ]; then
    echo "   테스트만 수행하고 실제 삭제는 하지 않습니다."
    echo ""
    echo "=== 검증 결과 (테스트) ==="
    EXPECTED_AFTER=$((BOT_ORDERS_BEFORE - DELETED_COUNT))
    if [ "${EXPECTED_AFTER}" -eq "${PRESERVED_ORDERS}" ]; then
        echo "✅ 성공: 삭제 후 예상 개수가 보존 대상과 일치합니다!"
        echo "   삭제 전: ${BOT_ORDERS_BEFORE}"
        echo "   삭제 후 예상: ${EXPECTED_AFTER}"
        echo "   보존된 orders: ${PRESERVED_ORDERS}"
        echo "   삭제될 orders: ${DELETED_COUNT}"
    else
        echo "❌ 실패: 삭제 후 예상 개수가 보존 대상과 다릅니다!"
        echo "   삭제 전: ${BOT_ORDERS_BEFORE}"
        echo "   삭제 후 예상: ${EXPECTED_AFTER}"
        echo "   예상 (보존 대상): ${PRESERVED_ORDERS}"
        echo "   삭제될 orders: ${DELETED_COUNT}"
        echo ""
        echo "   차이: $((EXPECTED_AFTER - PRESERVED_ORDERS))"
        exit 1
    fi
    exit 0
fi

# 6. 실제 삭제 실행
echo "6. 실제 삭제 실행 중..."
docker exec dustin-postgres psql -U ${DB_USER} -d ${DB_NAME} -c "
DELETE FROM orders
WHERE user_id = ANY(ARRAY[${BOT1_ID}, ${BOT2_ID}])
AND NOT EXISTS (
    SELECT 1 FROM trades
    WHERE (buy_order_id = orders.id OR sell_order_id = orders.id)
    AND (buyer_id != ALL(ARRAY[${BOT1_ID}, ${BOT2_ID}]) 
         OR seller_id != ALL(ARRAY[${BOT1_ID}, ${BOT2_ID}]))
);
" > /dev/null 2>&1

# 7. 삭제 후 봇 orders 개수 확인
echo "7. 삭제 후 봇 orders 개수 확인..."
BOT_ORDERS_AFTER=$(docker exec dustin-postgres psql -U ${DB_USER} -d ${DB_NAME} -t -c "
SELECT COUNT(*) FROM orders 
WHERE user_id IN (${BOT1_ID}, ${BOT2_ID});
" 2>/dev/null | tr -d ' ')

echo "   봇 orders 개수: ${BOT_ORDERS_AFTER}"
echo ""

# 8. 검증
echo "=== 검증 결과 ==="
if [ "${BOT_ORDERS_AFTER}" -eq "${PRESERVED_ORDERS}" ]; then
    echo "✅ 성공: 삭제 후 orders 개수가 보존 대상과 일치합니다!"
    echo "   삭제 전: ${BOT_ORDERS_BEFORE}"
    echo "   삭제 후: ${BOT_ORDERS_AFTER}"
    echo "   보존된 orders: ${PRESERVED_ORDERS}"
    echo "   삭제된 orders: ${DELETED_COUNT}"
else
    echo "❌ 실패: 삭제 후 orders 개수가 예상과 다릅니다!"
    echo "   삭제 전: ${BOT_ORDERS_BEFORE}"
    echo "   삭제 후: ${BOT_ORDERS_AFTER}"
    echo "   예상 (보존 대상): ${PRESERVED_ORDERS}"
    echo "   삭제된 orders: ${DELETED_COUNT}"
    echo ""
    echo "   차이: $((BOT_ORDERS_AFTER - PRESERVED_ORDERS))"
    exit 1
fi

echo ""
echo "=== 테스트 완료 ==="
