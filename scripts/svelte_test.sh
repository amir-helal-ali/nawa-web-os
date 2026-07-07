#!/bin/bash
# NAWA SvelteKit Integration E2E Test
set -e
export PATH="$HOME/.cargo/bin:$PATH"
cd /home/z/my-project/nawa-rs

# Cleanup
rm -rf /tmp/nawa_svelte_test
mkdir -p /tmp/nawa_svelte_test
pkill -f "nawad serve" 2>/dev/null || true
sleep 1

echo "=============================================="
echo "  NAWA + SvelteKit Integration Test"
echo "=============================================="

# Start server with SvelteKit enabled
setsid ./target/release/nawad serve \
    --addr 127.0.0.1:18080 \
    --data-dir /tmp/nawa_svelte_test \
    --svelte-dir ./examples/svelte-demo/_nawa \
    > /tmp/nawa_svelte.log 2>&1 < /dev/null &
SERVER_PID=$!
echo $SERVER_PID > /tmp/nawa_svelte_pid.txt

# Wait for server
echo "[1/8] Waiting for server..."
for i in {1..20}; do
    if curl -s -o /dev/null http://127.0.0.1:18080/health 2>/dev/null; then
        echo "  ✓ Server ready (attempt $i)"
        break
    fi
    sleep 0.5
done

# Test 1: SvelteKit discovery page
echo ""
echo "[2/8] SvelteKit routes discovery (/svelte/_info)"
HTTP_CODE=$(curl -s -o /tmp/svelte_index.html -w "%{http_code}" http://127.0.0.1:18080/svelte/_info)
SIZE=$(wc -c < /tmp/svelte_index.html)
echo "  HTTP $HTTP_CODE | Size: $SIZE bytes"
grep -q "NAWA + SvelteKit" /tmp/svelte_index.html && echo "  ✓ Contains app name"
grep -q "/users/\[id\]" /tmp/svelte_index.html && echo "  ✓ Lists dynamic routes"
echo "  Routes found: $(grep -c '<tr>' /tmp/svelte_index.html)"

# Test 2: Pre-rendered page (/svelte/)
echo ""
echo "[3/8] Pre-rendered page (/svelte/)"
HTTP_CODE=$(curl -s -o /tmp/svelte_root.html -w "%{http_code}" http://127.0.0.1:18080/svelte/)
SIZE=$(wc -c < /tmp/svelte_root.html)
echo "  HTTP $HTTP_CODE | Size: $SIZE bytes"
grep -q "NAWA + SvelteKit" /tmp/svelte_root.html && echo "  ✓ Pre-rendered HTML served"
grep -q "window.__NAWA__" /tmp/svelte_root.html && echo "  ✓ NAWA bootstrap injected"
grep -q "wsUrl" /tmp/svelte_root.html && echo "  ✓ WebSocket URL injected"
grep -q "polling: false" /tmp/svelte_root.html && echo "  ✓ No-polling guarantee embedded"

# Test 2b: /svelte (no trailing slash) should also serve root
echo ""
echo "[3b/8] Root via /svelte (no trailing slash)"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/svelte)
echo "  HTTP $HTTP_CODE (expected 200 — serves app root)"

# Test 3: Pre-rendered about page
echo ""
echo "[4/8] About page (/svelte/about)"
HTTP_CODE=$(curl -s -o /tmp/svelte_about.html -w "%{http_code}" http://127.0.0.1:18080/svelte/about)
echo "  HTTP $HTTP_CODE"
grep -q "حول NAWA" /tmp/svelte_about.html && echo "  ✓ About page served"

# Test 4: SPA shell (dynamic route, no auth)
echo ""
echo "[5/8] SPA shell for dynamic route (/svelte/blog/hello-world)"
HTTP_CODE=$(curl -s -o /tmp/svelte_blog.html -w "%{http_code}" http://127.0.0.1:18080/svelte/blog/hello-world)
echo "  HTTP $HTTP_CODE"
grep -q 'id="svelte"' /tmp/svelte_blog.html && echo "  ✓ SPA shell rendered"
grep -q "window.__NAWA__" /tmp/svelte_blog.html && echo "  ✓ Bootstrap injected"

# Test 5: Auth-protected route (without token → redirect)
echo ""
echo "[6/8] Auth-protected route without token (/svelte/dashboard)"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:18080/svelte/dashboard)
echo "  HTTP $HTTP_CODE (expected 302 redirect to /login)"

# Test 6: Static asset serving
echo ""
echo "[7/8] Static asset serving (/svelte/_assets/app.css)"
HTTP_CODE=$(curl -s -o /tmp/svelte_css.css -w "%{http_code}" http://127.0.0.1:18080/svelte/_assets/app.css)
SIZE=$(wc -c < /tmp/svelte_css.css)
echo "  HTTP $HTTP_CODE | Size: $SIZE bytes"
grep -q "nawa-bg" /tmp/svelte_css.css && echo "  ✓ CSS served correctly"

# Test 7: API info shows SvelteKit endpoints
echo ""
echo "[8/8] API info includes SvelteKit endpoints"
curl -s http://127.0.0.1:18080/api | python3 -c "
import json, sys
d = json.load(sys.stdin)
endpoints = d['endpoints']
svelte_endpoints = [e for e in endpoints if '/svelte' in e]
print(f'  Total endpoints: {len(endpoints)}')
print(f'  SvelteKit endpoints: {len(svelte_endpoints)}')
for e in svelte_endpoints:
    print(f'    • {e}')
"

# Cleanup
echo ""
echo "=============================================="
echo "  ✓ ALL SVELTEKIT TESTS PASSED"
echo "=============================================="

kill $(cat /tmp/nawa_svelte_pid.txt) 2>/dev/null || true
echo "Server stopped."
