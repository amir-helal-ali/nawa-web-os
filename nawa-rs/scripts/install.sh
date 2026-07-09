#!/bin/bash
# NAWA Web Operating System — Universal Installer v2.0.0
# أمر واحد يثبّت النظام بالكامل بكل مشتملاته
set -e

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
NAWA_VERSION="2.0.0"
NAWA_INSTALL_DIR="${NAWA_INSTALL_DIR:-$HOME/.nawa}"
NAWA_BIN_DIR="$NAWA_INSTALL_DIR/bin"
NAWA_PLUGINS_DIR="$NAWA_INSTALL_DIR/plugins"
NAWA_TEMPLATES_DIR="$NAWA_INSTALL_DIR/templates/basic"
NAWA_REPO="https://github.com/amir-helal-ali/nawa-web-os"

print_step() { echo -e "${BLUE}[$(date +%H:%M:%S)] $1${NC}"; }
print_success() { echo -e "${GREEN}[$(date +%H:%M:%S)] ✓ $1${NC}"; }

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  NAWA Web Operating System v2.0.0 — Full Installer          ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 1. إيقاف وحذف النسخة القديمة
print_step "حذف النسخة القديمة..."
pkill -f "nawad serve" 2>/dev/null || true
sleep 1
rm -rf "$NAWA_INSTALL_DIR"
sed -i '/\.nawa/d' "$HOME/.bashrc" 2>/dev/null || true
print_success "تم التنظيف"

# 2. فحص Rust
print_step "فحص Rust..."
if ! command -v cargo &>/dev/null; then
    print_step "تثبيت Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1 | tail -3
    source "$HOME/.cargo/env"
fi
print_success "Rust: $(cargo --version)"

# 3. تحميل الكود
print_step "تحميل NAWA v$NAWA_VERSION من GitHub..."
NAWA_SRC="$NAWA_INSTALL_DIR/src"
mkdir -p "$NAWA_SRC"
cd "$NAWA_SRC"
if [ -d "nawa-web-os" ]; then
    cd nawa-web-os/nawa-rs && git pull --quiet 2>/dev/null || true
else
    git clone --depth 1 "$NAWA_REPO" nawa-web-os 2>&1 | tail -3
    cd nawa-web-os/nawa-rs
fi
print_success "الكود جاهز"

# 4. بناء nawad + nawa-cli
print_step "بناء NAWA (5-10 دقائق)..."
cargo build --release 2>&1 | tail -3
mkdir -p "$NAWA_BIN_DIR"
cp target/release/nawad "$NAWA_BIN_DIR/"
cp target/release/nawa "$NAWA_BIN_DIR/"
print_success "nawad + nawa-cli جاهزان"

# 5. بناء WASM SSR module
print_step "بناء WASM SSR module..."
rustup target add wasm32-unknown-unknown 2>&1 | tail -1
cd examples/wasm-ssr-module
cargo build --release --target wasm32-unknown-unknown 2>&1 | tail -2
mkdir -p "$NAWA_PLUGINS_DIR"
cp target/wasm32-unknown-unknown/release/nawa_ssr_demo.wasm "$NAWA_PLUGINS_DIR/"
cd "$NAWA_SRC/nawa-web-os/nawa-rs"
print_success "WASM module جاهز"

# 6. القوالب
print_step "إنشاء القوالب..."
mkdir -p "$NAWA_TEMPLATES_DIR" "$NAWA_TEMPLATES_DIR/data" "$NAWA_TEMPLATES_DIR/plugins" "$NAWA_TEMPLATES_DIR/static"
cp templates/basic/nawa.toml "$NAWA_TEMPLATES_DIR/" 2>/dev/null || true
cp templates/basic/README.md "$NAWA_TEMPLATES_DIR/" 2>/dev/null || true
print_success "القوالب جاهزة"

# 7. PATH
print_step "إضافة لـ PATH..."
echo "export PATH=\"\$HOME/.nawa/bin:\$PATH\"" >> "$HOME/.bashrc"
print_success "أُضيف لـ ~/.bashrc"

# 8. أمر nawa الموحد
print_step "إنشاء أمر nawa..."
cat > "$NAWA_BIN_DIR/nawa" << 'CMD'
#!/bin/bash
NAWA_DIR="$HOME/.nawa"
NAWA_BIN="$NAWA_DIR/bin"
case "${1:-help}" in
    serve) shift; exec "$NAWA_BIN/nawad" serve "$@";;
    new)
        PROJECT_NAME="${2:-my-nawa-app}"
        PROJECT_DIR="$(pwd)/$PROJECT_NAME"
        echo "🚀 إنشاء مشروع NAWA: $PROJECT_NAME"
        mkdir -p "$PROJECT_DIR"
        cp -a "$NAWA_DIR/templates/basic/." "$PROJECT_DIR/" 2>/dev/null || true
        mkdir -p "$PROJECT_DIR/plugins"
        cp "$NAWA_DIR/plugins/nawa_ssr_demo.wasm" "$PROJECT_DIR/plugins/" 2>/dev/null || true
        echo "✓ تم في: $PROJECT_DIR"
        echo "الخطوات: cd $PROJECT_NAME && nawad serve"
        echo "ثم افتح: http://localhost:8080"
        ;;
    build-wasm)
        cd "$2" && rustup target add wasm32-unknown-unknown 2>/dev/null
        cargo build --release --target wasm32-unknown-unknown
        echo "✓ WASM جاهز"
        ;;
    info) exec "$NAWA_BIN/nawad" info;;
    benchmark) exec "$NAWA_BIN/nawad" benchmark "${@:2}";;
    version) "$NAWA_BIN/nawad" --version;;
    update) bash -c "$(curl -fsSL https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/scripts/install.sh)";;
    uninstall)
        echo "🗑️ حذف NAWA..."
        pkill -f "nawad" 2>/dev/null || true
        rm -rf "$NAWA_DIR"
        sed -i '/\.nawa/d' "$HOME/.bashrc" 2>/dev/null || true
        echo "✅ تم حذف NAWA. نفّذ: source ~/.bashrc"
        ;;
    help|--help|-h)
        echo "NAWA Web Operating System"
        echo ""
        echo "الأوامر:"
        echo "  serve [options]       تشغيل الخادم"
        echo "  new <name>            إنشاء مشروع جديد"
        echo "  build-wasm <path>     بناء WASM module"
        echo "  info                  معلومات النظام"
        echo "  benchmark [ops]       قياس الأداء"
        echo "  version               الإصدار"
        echo "  update                تحديث NAWA"
        echo "  uninstall             حذف NAWA نهائياً"
        echo "  help                  هذه المساعدة"
        ;;
    *) echo "أمر غير معروف: $1"; echo "استخدم: nawa help"; exit 1;;
esac
CMD
chmod +x "$NAWA_BIN_DIR/nawa"
print_success "أمر nawa جاهز"

# 9. التحقق
print_step "التحقق..."
export PATH="$HOME/.nawa/bin:$PATH"
VERSION=$(nawad --version 2>&1)
print_success "nawad: $VERSION"

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ تم تثبيت NAWA v2.0.0 بالكامل!                ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${YELLOW}الخطوات التالية:${NC}"
echo -e "  1. ${GREEN}source ~/.bashrc${NC}"
echo -e "  2. ${GREEN}nawa new my-app${NC}"
echo -e "  3. ${GREEN}cd my-app && nawad serve${NC}"
echo -e "  4. ${GREEN}افتح: http://localhost:8080${NC}"
echo ""
