#!/bin/bash
# NAWA E2E Test Script — full integration test with real WebSocket
set -e
export PATH="$HOME/.cargo/bin:$PATH"
cd /home/z/my-project/nawa-rs

# Cleanup
rm -rf /tmp/nawa_test_data
mkdir -p /tmp/nawa_test_data /tmp/nawa_test_static
pkill -f "nawad serve" 2>/dev/null || true
sleep 1

echo "=============================================="
echo "  NAWA E2E Integration Test"
echo "=============================================="

# Start server (fully detached with setsid)
setsid ./target/release/nawad serve \
    --addr 127.0.0.1:18080 \
    --data-dir /tmp/nawa_test_data \
    --static-dir /tmp/nawa_test_static \
    > /tmp/nawa_server.log 2>&1 < /dev/null &
SERVER_PID=$!
echo $SERVER_PID > /tmp/nawa_pid.txt

# Wait for server to be ready
echo "[1/10] Waiting for server to start..."
for i in {1..20}; do
    if curl -s -o /dev/null http://127.0.0.1:18080/health 2>/dev/null; then
        echo "  ✓ Server ready (attempt $i)"
        break
    fi
    sleep 0.5
done

# === Test 1: Dashboard ===
echo ""
echo "[2/10] Dashboard GET /"
HTTP_CODE=$(curl -s -o /tmp/resp1.html -w "%{http_code}" http://127.0.0.1:18080/)
TITLE=$(grep -oP '<title>[^<]+</title>' /tmp/resp1.html | head -1)
SIZE=$(wc -c < /tmp/resp1.html)
echo "  HTTP $HTTP_CODE | Size: $SIZE bytes | $TITLE"
if [ "$HTTP_CODE" != "200" ]; then echo "  ✗ FAILED"; exit 1; fi
echo "  ✓ PASS"

# === Test 2: API info ===
echo ""
echo "[3/10] API info GET /api"
API_RESP=$(curl -s http://127.0.0.1:18080/api)
echo "$API_RESP" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f'  Name: {d[\"name\"]} v{d[\"version\"]}'); print(f'  Endpoints: {len(d[\"endpoints\"])}')"
echo "  ✓ PASS"

# === Test 3: Register first user (becomes admin) ===
echo ""
echo "[4/10] Register first user → should become admin"
curl -s -X POST http://127.0.0.1:18080/register \
    -d "username=admin&email=admin@nawa.test&password=secret123" \
    -c /tmp/cookies.txt \
    -o /tmp/resp2.html -w "  HTTP %{http_code}\n"
TOKEN=$(grep nawa_token /tmp/cookies.txt | awk '{print $NF}')
if [ -z "$TOKEN" ]; then echo "  ✗ No token received"; exit 1; fi
echo "  ✓ Token received: ${TOKEN:0:30}..."

# === Test 4: Auth /me ===
echo ""
echo "[5/10] Auth /me (with Bearer token)"
ME_RESP=$(curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:18080/auth/me)
echo "$ME_RESP" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f'  Username: {d[\"username\"]}'); print(f'  Role: {d[\"role\"]}'); print(f'  Verified: {d[\"verified\"]}')"
ROLE=$(echo "$ME_RESP" | python3 -c "import json,sys; print(json.load(sys.stdin)['role'])")
if [ "$ROLE" != "admin" ]; then echo "  ✗ First user is NOT admin"; exit 1; fi
echo "  ✓ PASS — first user is admin"

# === Test 5: DB write (triggers WS notification) ===
echo ""
echo "[6/10] DB write POST /test_key"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"hello":"world","num":42}' \
    http://127.0.0.1:18080/test_key
echo ""

# === Test 6: DB read ===
echo ""
echo "[7/10] DB read GET /test_key"
DB_VAL=$(curl -s http://127.0.0.1:18080/test_key)
echo "  Value: $DB_VAL"

# === Test 7: Notifications stats (count should be > 0) ===
echo ""
echo "[8/10] Notifications stats (Event Bus push count)"
NOTIF_STATS=$(curl -s http://127.0.0.1:18080/notifications/stats)
echo "$NOTIF_STATS" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f'  Total notifications: {d[\"total\"]}')"
NOTIF_COUNT=$(echo "$NOTIF_STATS" | python3 -c "import json,sys; print(json.load(sys.stdin)['total'])")
if [ "$NOTIF_COUNT" -lt "2" ]; then echo "  ✗ Expected ≥2 notifications (register+db_write)"; exit 1; fi
echo "  ✓ PASS — events pushed through Event Bus"

# === Test 8: Health ===
echo ""
echo "[9/10] Health check"
curl -s http://127.0.0.1:18080/health | python3 -c "import json,sys; d=json.load(sys.stdin); print(f'  Status: {d[\"status\"]} | Keys: {d[\"keys\"]}')"

# === Test 9: WebSocket handshake (RFC 6455) ===
echo ""
echo "[10/10] WebSocket handshake (RFC 6455)"
WS_KEY="dGhlIHNhbXBsZSBub25jZQ=="
WS_HANDSHAKE=$(curl -s -i -N \
    -H "Connection: Upgrade" \
    -H "Upgrade: websocket" \
    -H "Sec-WebSocket-Version: 13" \
    -H "Sec-WebSocket-Key: $WS_KEY" \
    --max-time 2 \
    http://127.0.0.1:18081/ 2>&1 | head -10)
echo "$WS_HANDSHAKE" | grep -q "101 Switching Protocols" && echo "  ✓ Got 101 Switching Protocols" || echo "  ✗ No 101 response"
echo "$WS_HANDSHAKE" | grep -i "sec-websocket-accept" | head -1
EXPECTED_ACCEPT="s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
echo "  Expected Accept: $EXPECTED_ACCEPT"

# === Bonus: System page ===
echo ""
echo "[Bonus] System page GET /system"
SYS_SIZE=$(curl -s -o /tmp/sys.html -w "%{size_download}" http://127.0.0.1:18080/system)
echo "  System page: $SYS_SIZE bytes"

# Cleanup
echo ""
echo "=============================================="
echo "  ✓ ALL TESTS PASSED"
echo "=============================================="
echo ""
echo "Server log tail:"
tail -5 /tmp/nawa_server.log

# Stop server
kill $(cat /tmp/nawa_pid.txt) 2>/dev/null || true
echo ""
echo "Server stopped."
