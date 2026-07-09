#!/bin/bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  NAWA Web Operating System — One Command for Everything       ║
# ║  تثبيت · تحديث · حذف — بأمر واحد                              ║
# ╚══════════════════════════════════════════════════════════════╝
#
# التثبيت أو التحديث:  bash install.sh
# الحذف الكامل:        bash install.sh --uninstall

set -e
G='\033[0;32m'; B='\033[0;34m'; Y='\033[1;33m'; R='\033[0;31m'; N='\033[0m'
DIR="$HOME/.nawa"
REPO="https://github.com/amir-helal-ali/nawa-web-os"

p() { echo -e "${B}[$(date +%H:%M:%S)] $1${N}"; }
s() { echo -e "${G}[$(date +%H:%M:%S)] ✓ $1${N}"; }
w() { echo -e "${Y}[$(date +%H:%M:%S)] ⚠ $1${N}"; }

# ── الحذف الكامل ──
if [ "${1:-}" = "--uninstall" ] || [ "${1:-}" = "uninstall" ]; then
    echo -e "${R}🗑️  حذف NAWA نهائياً...${N}"
    pkill -f "nawad" 2>/dev/null || true
    sleep 1
    rm -rf "$DIR"
    sed -i '/\.nawa/d' "$HOME/.bashrc" 2>/dev/null || true
    echo -e "${G}✅ تم حذف NAWA بالكامل${N}"
    echo "نفّذ: source ~/.bashrc"
    exit 0
fi

# ── التثبيت أو التحديث ──
IS_UPDATE=false
if [ -f "$DIR/bin/nawad" ]; then
    IS_UPDATE=true
    echo -e "${B}╔══════════════════════════════════════════════╗${N}"
    echo -e "${B}║  NAWA — تحديث النظام                          ║${N}"
    echo -e "${B}╚══════════════════════════════════════════════╝${N}"
else
    echo -e "${B}╔══════════════════════════════════════════════╗${N}"
    echo -e "${B}║  NAWA — تثبيت جديد                            ║${N}"
    echo -e "${B}╚══════════════════════════════════════════════╝${N}"
fi

# 1. إيقاف الخادم إن كان يعمل
p "إيقاف الخادم..."
pkill -f "nawad serve" 2>/dev/null && sleep 1 || true
s "تم"

# 2. Rust
p "فحص Rust..."
if ! command -v cargo &>/dev/null; then
    p "تثبيت Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1 | tail -2
    source "$HOME/.cargo/env"
fi
s "Rust: $(cargo --version)"

# 3. تحميل/تحديث الكود
p "تحميل الكود من GitHub..."
mkdir -p "$DIR/src"
cd "$DIR/src"
if [ -d "nawa-web-os" ]; then
    cd nawa-web-os/nawa-rs
    git pull --quiet 2>/dev/null || true
else
    git clone --depth 1 "$REPO" nawa-web-os 2>&1 | tail -2
    cd nawa-web-os/nawa-rs
fi
s "الكود جاهز"

# 4. بناء nawad
p "بناء nawad (5-10 دقائق)..."
cargo build --release 2>&1 | tail -2
mkdir -p "$DIR/bin"
cp target/release/nawad "$DIR/bin/"
cp target/release/nawa "$DIR/bin/"
s "nawad جاهز: $(./target/release/nawad --version)"

# 5. بناء SvelteKit
p "بناء واجهة SvelteKit..."
cd examples/svelte-app
if ! command -v npm &>/dev/null; then
    w "npm غير مثبّت — تثبيت Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - 2>&1 | tail -2
    sudo apt-get install -y nodejs 2>&1 | tail -2
fi
npm install --silent 2>&1 | tail -2
npm run build 2>&1 | tail -3
cd ../..
s "SvelteKit جاهز"

# 6. بناء WASM
p "بناء WASM SSR module..."
rustup target add wasm32-unknown-unknown 2>&1 | tail -1
cd examples/wasm-ssr-module
cargo build --release --target wasm32-unknown-unknown 2>&1 | tail -1
mkdir -p "$DIR/plugins"
cp target/wasm32-unknown-unknown/release/nawa_ssr_demo.wasm "$DIR/plugins/"
cd "$DIR/src/nawa-web-os/nawa-rs"
s "WASM module جاهز"

# 7. القوالب
p "نسخ القوالب..."
mkdir -p "$DIR/templates/basic"/{data,plugins,static}
cp templates/basic/nawa.toml "$DIR/templates/basic/" 2>/dev/null || true
cp templates/basic/README.md "$DIR/templates/basic/" 2>/dev/null || true
s "القوالب جاهزة"

# 8. PATH
if ! grep -q "\.nawa/bin" "$HOME/.bashrc" 2>/dev/null; then
    echo "export PATH=\"\$HOME/.nawa/bin:\$PATH\"" >> "$HOME/.bashrc"
fi

# 9. أمر nawa الموحد
cat > "$DIR/bin/nawa" << 'CMD'
#!/bin/bash
D="$HOME/.nawa"; B="$D/bin"
case "${1:-help}" in
    serve) shift; exec "$B/nawad" serve "$@";;
    new)
        P="${2:-my-app}"; D2="$(pwd)/$P"
        echo "🚀 إنشاء مشروع: $P"
        mkdir -p "$D2"
        cp -a "$D/templates/basic/." "$D2/" 2>/dev/null || true
        mkdir -p "$D2/plugins"
        cp "$D/plugins/nawa_ssr_demo.wasm" "$D2/plugins/" 2>/dev/null || true
        echo "✓ تم في: $D2"
        echo "cd $P && nawad serve"
        echo "افتح: http://localhost:8080"
        ;;
    build-wasm)
        cd "$2" && rustup target add wasm32-unknown-unknown 2>/dev/null
        cargo build --release --target wasm32-unknown-unknown
        echo "✓ WASM جاهز"
        ;;
    info) exec "$B/nawad" info;;
    benchmark) exec "$B/nawad" benchmark "${@:2}";;
    version) "$B/nawad" --version;;
    update) bash -c "$(curl -fsSL https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/scripts/install.sh)";;
    uninstall) bash "$0" --uninstall;;
    help|*)
        echo "NAWA Web Operating System"
        echo ""
        echo "الأوامر:"
        echo "  serve [options]     تشغيل الخادم"
        echo "  new <name>          إنشاء مشروع جديد"
        echo "  build-wasm <path>   بناء WASM module"
        echo "  info                معلومات النظام"
        echo "  benchmark [ops]     قياس الأداء"
        echo "  version             الإصدار"
        echo "  update              تحديث NAWA"
        echo "  uninstall           حذف NAWA نهائياً"
        ;;
    esac
CMD
chmod +x "$DIR/bin/nawa"

# 10. التحقق
export PATH="$DIR/bin:$PATH"
V=$(nawad --version 2>&1)

echo ""
echo -e "${G}╔══════════════════════════════════════════════╗${N}"
if $IS_UPDATE; then
    echo -e "${G}║  ✅ تم تحديث NAWA بنجاح!                      ║${N}"
else
    echo -e "${G}║  ✅ تم تثبيت NAWA بنجاح!                      ║${N}"
fi
echo -e "${G}╚══════════════════════════════════════════════╝${N}"
echo ""
echo "  الإصدار: $V"
echo "  الواجهة: SvelteKit (عصرية) — http://localhost:8080"
echo "  التوثيق: http://localhost:8080/docs"
echo ""
echo "  الأوامر:"
echo "    nawa serve        — تشغيل"
echo "    nawa new my-app   — مشروع جديد"
echo "    nawa update       — تحديث"
echo "    nawa uninstall    — حذف كامل"
echo ""
echo "  نفّذ: source ~/.bashrc"
echo ""
