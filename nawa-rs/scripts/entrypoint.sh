#!/bin/bash
# NAWA Docker entrypoint — يُشغّل النظام بالكامل

set -e

# إذا كان الأمر "serve"، شغّل nawad serve
if [ "${1:-serve}" = "serve" ]; then
    shift 2>/dev/null || true
    exec nawad serve \
        --addr "${NAWA_ADDR:-0.0.0.0:8080}" \
        --data-dir "${NAWA_DATA_DIR:-/opt/nawa/data}" \
        --plugins-dir "${NAWA_PLUGINS_DIR:-/opt/nawa/plugins}" \
        --static-dir "${NAWA_STATIC_DIR:-/opt/nawa/static}" \
        "$@"
elif [ "${1:-}" = "new" ]; then
    # إنشاء مشروع جديد
    PROJECT_NAME="${2:-my-nawa-app}"
    mkdir -p "/opt/nawa/projects/$PROJECT_NAME"
    cd "/opt/nawa/projects/$PROJECT_NAME"
    cat > nawa.toml << 'EOF'
addr = "0.0.0.0:8080"
data_dir = "./data"
plugins_dir = "./plugins"
static_dir = "./static"
jwt_secret = "change-this"
rate_limit = 100
wal_sync = true
log_level = "info"
EOF
    mkdir -p data plugins static
    cp /opt/nawa/plugins/nawa_ssr_demo.wasm plugins/ 2>/dev/null || true
    echo "✓ Project created: /opt/nawa/projects/$PROJECT_NAME"
    echo "Run: cd /opt/nawa/projects/$PROJECT_NAME && nawad serve"
elif [ "${1:-}" = "info" ]; then
    exec nawad info
elif [ "${1:-}" = "benchmark" ]; then
    exec nawad benchmark "${@:2}"
elif [ "${1:-}" = "help" ] || [ "${1:-}" = "--help" ]; then
    cat << 'HELP'
NAWA Web Operating System (Docker)

Usage:
  docker run nawa-web-os                    # تشغيل الخادم
  docker run nawa-web-os serve              # تشغيل صريح
  docker run nawa-web-os new my-app         # إنشاء مشروع
  docker run nawa-web-os info               # معلومات النظام
  docker run nawa-web-os benchmark          # benchmark

Environment variables:
  NAWA_ADDR          عنوان الخادم (default: 0.0.0.0:8080)
  NAWA_DATA_DIR      مجلد البيانات (default: /opt/nawa/data)
  NAWA_PLUGINS_DIR   مجلد الإضافات (default: /opt/nawa/plugins)
  NAWA_STATIC_DIR    مجلد الملفات الثابتة (default: /opt/nawa/static)
  RUST_LOG           مستوى التسجيل (default: info)

Ports:
  8080  HTTP server
  8081  WebSocket server

Volumes:
  /opt/nawa/data     بيانات NAWA-DB (احفظها!)
  /opt/nawa/plugins  إضافات WASM
  /opt/nawa/static   ملفات ثابتة

Examples:
  docker run -p 8080:8080 -p 8081:8081 nawa-web-os
  docker run -v nawa-data:/opt/nawa/data -p 8080:8080 nawa-web-os
  docker run nawa-web-os new my-project
HELP
else
    exec "$@"
fi
