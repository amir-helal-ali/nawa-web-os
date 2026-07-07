#!/bin/bash
# NAWA Comprehensive Integration Test — all subsystems together
set -e
export PATH="$HOME/.cargo/bin:$PATH"
cd /home/z/my-project/nawa-rs

# Cleanup
rm -rf /tmp/nawa_e2e_full
mkdir -p /tmp/nawa_e2e_full
pkill -f "nawad serve" 2>/dev/null || true
sleep 1

echo "╔══════════════════════════════════════════════════════╗"
echo "║  NAWA Comprehensive Integration Test                  ║"
echo "║  Auth + DB + WebSocket + Svelte + AION + All together ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

# Start server with SvelteKit enabled
setsid ./target/release/nawad serve \
    --addr 127.0.0.1:18080 \
    --data-dir /tmp/nawa_e2e_full \
    --svelte-dir ./examples/svelte-demo/_nawa \
    > /tmp/nawa_e2e.log 2>&1 < /dev/null &
SERVER_PID=$!
echo $SERVER_PID > /tmp/nawa_e2e_pid.txt

# Wait for server
for i in {1..30}; do
    if curl -s -o /dev/null http://127.0.0.1:18080/health 2>/dev/null; then
        echo "✓ Server ready (attempt $i)"
        break
    fi
    sleep 0.5
done

PASS=0
FAIL=0
TOTAL=0

check() {
    TOTAL=$((TOTAL+1))
    if [ "$2" = "PASS" ]; then
        echo "  ✓ [$TOTAL] $1"
        PASS=$((PASS+1))
    else
        echo "  ✗ [$TOTAL] $1"
        FAIL=$((FAIL+1))
    fi
}

echo ""
echo "═══ 1. AUTH SYSTEM ═══"
# Register first user → should become admin
RESP=$(curl -s -X POST http://127.0.0.1:18080/register \
    -d "username=admin&email=admin@nawa.test&password=secret123" \
    -c /tmp/e2e_cookies.txt -w "%{http_code}" -o /dev/null)
check "Register first user (admin)" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# Get auth token
TOKEN=$(grep nawa_token /tmp/e2e_cookies.txt | awk '{print $NF}')
check "Auth token received" "$([ -n "$TOKEN" ] && echo PASS || echo FAIL)"

# Verify /auth/me
ROLE=$(curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:18080/auth/me | python3 -c "import json,sys; print(json.load(sys.stdin)['role'])")
check "First user is admin" "$([ "$ROLE" = "admin" ] && echo PASS || echo FAIL)"

# Register a second user (not verified, will be blocked by login)
curl -s -X POST http://127.0.0.1:18080/register \
    -d "username=user1&email=user1@nawa.test&password=pass123" -o /dev/null

# Login with second user (should fail because verification is required by default)
RESP=$(curl -s -X POST http://127.0.0.1:18080/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"user1@nawa.test","password":"pass123"}' \
    -w "%{http_code}" -o /dev/null)
# Expected 401 (account not verified) — this is the correct security behavior.
check "API login blocks unverified accounts (security)" "$([ "$RESP" = "401" ] && echo PASS || echo FAIL)"

echo ""
echo "═══ 2. DATABASE (NAWA-DB) ═══"
# Write JSON value
RESP=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"title":"Test Article","author":"admin","date_published":"2026-01-01"}' \
    http://127.0.0.1:18080/post:test-1 -w "%{http_code}" -o /dev/null)
check "DB write JSON" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# Read back
VAL=$(curl -s http://127.0.0.1:18080/post:test-1)
check "DB read returns JSON" "$([ -n "$VAL" ] && echo PASS || echo FAIL)"

# Scan prefix
COUNT=$(curl -s http://127.0.0.1:18080/scan/post | python3 -c "import json,sys; print(json.load(sys.stdin)['count'])")
check "DB scan prefix" "$([ "$COUNT" -ge "1" ] && echo PASS || echo FAIL)"

# Health endpoint
KEYS=$(curl -s http://127.0.0.1:18080/health | python3 -c "import json,sys; print(json.load(sys.stdin)['keys'])")
check "Health endpoint reports keys" "$([ "$KEYS" -ge "5" ] && echo PASS || echo FAIL)"

echo ""
echo "═══ 3. WEBSOCKET (Real-time push, no polling) ═══"
# WebSocket handshake (RFC 6455 test vector)
WS_KEY="dGhlIHNhbXBsZSBub25jZQ=="
WS_RESP=$(curl -s -i -N \
    -H "Connection: Upgrade" -H "Upgrade: websocket" \
    -H "Sec-WebSocket-Version: 13" \
    -H "Sec-WebSocket-Key: $WS_KEY" \
    --max-time 5 http://127.0.0.1:18081/ 2>&1 | head -20)
echo "$WS_RESP" | grep -q "101 Switching Protocols" && check "WebSocket 101 handshake" PASS || check "WebSocket 101 handshake" FAIL
echo "$WS_RESP" | grep -q "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=" && check "RFC 6455 Accept hash correct" PASS || check "RFC 6455 Accept hash correct" FAIL

# Notifications stats (events pushed by register + db_write)
NOTIFS=$(curl -s http://127.0.0.1:18080/notifications/stats | python3 -c "import json,sys; print(json.load(sys.stdin)['total'])")
check "Event Bus pushed notifications" "$([ "$NOTIFS" -ge "3" ] && echo PASS || echo FAIL)"

echo ""
echo "═══ 4. SVELTEKIT INTEGRATION ═══"
# Discovery page
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/svelte/_info)
check "SvelteKit discovery page" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# Root page (pre-rendered)
RESP=$(curl -s -o /tmp/svelte_root.html -w "%{http_code}" http://127.0.0.1:18080/svelte/)
check "SvelteKit root (pre-rendered)" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"
grep -q "window.__NAWA__" /tmp/svelte_root.html && check "NAWA bootstrap injected" PASS || check "NAWA bootstrap injected" FAIL
grep -q "polling: false" /tmp/svelte_root.html && check "No-polling guarantee embedded" PASS || check "No-polling guarantee embedded" FAIL

# About page
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/svelte/about)
check "SvelteKit about page" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# Dynamic route (catch-all)
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/svelte/blog/hello-world)
check "SvelteKit dynamic route (catch-all)" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# Auth-protected route → redirect
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/svelte/dashboard)
check "SvelteKit auth-protected route redirects" "$([ "$RESP" = "302" ] && echo PASS || echo FAIL)"

# Static asset
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/svelte/_assets/app.css)
check "SvelteKit static asset (CSS)" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

echo ""
echo "═══ 5. AION SEO ENGINE ═══"
# AION stats
RESP=$(curl -s -o /tmp/aion_stats.json -w "%{http_code}" http://127.0.0.1:18080/aion/stats)
check "AION stats endpoint" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"
ENTITIES=$(python3 -c "import json; d=json.load(open('/tmp/aion_stats.json')); print(d['knowledge_graph']['entities'])")
check "AION Knowledge Graph has entities" "$([ "$ENTITIES" -ge "1" ] && echo PASS || echo FAIL)"

# Photon Protocol
RESP=$(curl -s -o /tmp/photon.json -w "%{http_code}" http://127.0.0.1:18080/__photon__)
check "Photon Protocol endpoint" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"
PROTOCOL=$(python3 -c "import json; d=json.load(open('/tmp/photon.json')); print(d['protocol'])")
check "Photon protocol version" "$([ "$PROTOCOL" = "photon/1.0" ] && echo PASS || echo FAIL)"
FORMATS=$(python3 -c "import json; d=json.load(open('/tmp/photon.json')); print(len(d['supported_formats']))")
check "Photon supports 9 formats" "$([ "$FORMATS" = "9" ] && echo PASS || echo FAIL)"

# Sitemap
RESP=$(curl -s -o /tmp/sitemap.xml -w "%{http_code}" http://127.0.0.1:18080/sitemap.xml)
check "Dynamic sitemap.xml" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"
URL_COUNT=$(grep -c "<loc>" /tmp/sitemap.xml)
check "Sitemap contains URLs" "$([ "$URL_COUNT" -ge "1" ] && echo PASS || echo FAIL)"

# Robots.txt
RESP=$(curl -s -o /tmp/robots.txt -w "%{http_code}" http://127.0.0.1:18080/robots.txt)
check "Dynamic robots.txt" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"
grep -q "GPTBot" /tmp/robots.txt && check "Robots allows AI crawlers" PASS || check "Robots allows AI crawlers" FAIL
grep -q "Sitemap:" /tmp/robots.txt && check "Robots has sitemap URL" PASS || check "Robots has sitemap URL" FAIL

echo ""
echo "═══ 6. API ENDPOINTS ═══"
# Total endpoints
TOTAL_EP=$(curl -s http://127.0.0.1:18080/api | python3 -c "import json,sys; print(len(json.load(sys.stdin)['endpoints']))")
check "API info lists endpoints" "$([ "$TOTAL_EP" -ge "30" ] && echo PASS || echo FAIL)"

# System page
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/system)
check "System page" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# Metrics (Prometheus)
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/metrics)
check "Prometheus metrics" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# io_uring stats
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/uring)
check "io_uring stats" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

echo ""
echo "═══ 7. SECURITY ═══"
# Password reset page
RESP=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/password-reset)
check "Password reset page" "$([ "$RESP" = "200" ] && echo PASS || echo FAIL)"

# User password_hash should NOT leak
ME_RESP=$(curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:18080/auth/me)
echo "$ME_RESP" | grep -q "password_hash" && check "No password_hash leak" FAIL || check "No password_hash leak" PASS

echo ""
echo "═══ SUMMARY ═══"
echo "═══════════════════════════════════════════════"
echo "  Total checks: $TOTAL"
echo "  Passed:       $PASS ✓"
echo "  Failed:       $FAIL ✗"
echo "═══════════════════════════════════════════════"

if [ "$FAIL" -gt "0" ]; then
    echo ""
    echo "Server log (last 20 lines):"
    tail -20 /tmp/nawa_e2e.log
    exit 1
fi

echo ""
echo "🎉 ALL INTEGRATION CHECKS PASSED — System is fully integrated."

# Cleanup
kill $(cat /tmp/nawa_e2e_pid.txt) 2>/dev/null || true
echo "Server stopped."
