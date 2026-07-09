#!/bin/bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  NAWA Web Operating System — Universal Installer               ║
# ║  أمر واحد يُثبّت النظام بالكامل ويُجهّزه لبناء المشاريع          ║
# ╚══════════════════════════════════════════════════════════════╝
set -e

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

NAWA_VERSION="1.7.0"
NAWA_INSTALL_DIR="${NAWA_INSTALL_DIR:-$HOME/.nawa}"
NAWA_BIN_DIR="$NAWA_INSTALL_DIR/bin"
NAWA_DATA_DIR="$NAWA_INSTALL_DIR/data"
NAWA_PLUGINS_DIR="$NAWA_INSTALL_DIR/plugins"
NAWA_TEMPLATES_DIR="$NAWA_INSTALL_DIR/templates"
NAWA_REPO="https://github.com/amir-helal-ali/nawa-web-os"
NAWA_RELEASE_URL="https://github.com/amir-helal-ali/nawa-web-os/releases/download/v${NAWA_VERSION}"

print_banner() {
    echo -e "${BLUE}"
    cat << 'BANNER'
╔══════════════════════════════════════════════════════════════╗
║  NAWA — Revolutionary Web Operating System                    ║
║  Binary واحد خالص · بدون Node.js · Rust خالص                  ║
╚══════════════════════════════════════════════════════════════╝
BANNER
    echo -e "${NC}"
}

print_step() { echo -e "${BLUE}[$(date +%H:%M:%S)] $1${NC}"; }
print_success() { echo -e "${GREEN}[$(date +%H:%M:%S)] ✓ $1${NC}"; }
print_warning() { echo -e "${YELLOW}[$(date +%H:%M:%S)] ⚠ $1${NC}"; }
print_error() { echo -e "${RED}[$(date +%H:%M:%S)] ✗ $1${NC}"; }

print_banner
echo -e "  ${YELLOW}الإصدار:${NC} $NAWA_VERSION"
echo -e "  ${YELLOW}مسار التثبيت:${NC} $NAWA_INSTALL_DIR"
echo ""

# فحص النظام
print_step "فحص النظام..."
OS="$(uname -s)"; ARCH="$(uname -m)"
case "$OS" in Linux*) NAWA_OS="linux";; Darwin*) NAWA_OS="macos";; *) print_error "نظام غير مدعوم: $OS"; exit 1;; esac
case "$ARCH" in x86_64|amd64) NAWA_ARCH="x86_64";; arm64|aarch64) NAWA_ARCH="arm64";; *) print_error "معمارية غير مدعومة: $ARCH"; exit 1;; esac
print_success "النظام: $NAWA_OS-$NAWA_ARCH"

# فحص التبعيات
print_step "فحص التبعيات..."
command -v curl &>/dev/null && print_success "curl مثبّت" || print_warning "curl غير مثبّت"
command -v cargo &>/dev/null && print_success "Rust مثبّت" || {
    print_warning "Rust غير مثبّت — جاري التثبيت..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1 | tail -3
    source "$HOME/.cargo/env"
    print_success "Rust مثبّت: $(cargo --version)"
}

# إنشاء مجلدات التثبيت
print_step "إنشاء مجلدات التثبيت..."
mkdir -p "$NAWA_BIN_DIR" "$NAWA_DATA_DIR" "$NAWA_PLUGINS_DIR" "$NAWA_TEMPLATES_DIR"
print_success "المجلدات جاهزة"

# تحميل الـ binary
print_step "تحميل NAWA binary..."
NAWAD_BINARY="$NAWA_BIN_DIR/nawad"
NAWA_CLI_BINARY="$NAWA_BIN_DIR/nawa"
DOWNLOAD_SUCCESS=false

if [ "$NAWA_OS" = "linux" ] && [ "$NAWA_ARCH" = "x86_64" ]; then
    NAWAD_URL="$NAWA_RELEASE_URL/nawad-linux-x86_64"
    if curl -fsSL "$NAWAD_URL" -o "$NAWAD_BINARY" 2>/dev/null; then
        curl -fsSL "$NAWA_RELEASE_URL/nawa-cli-linux-x86_64" -o "$NAWA_CLI_BINARY" 2>/dev/null
        chmod +x "$NAWAD_BINARY" "$NAWA_CLI_BINARY"
        DOWNLOAD_SUCCESS=true
    fi
fi

# البناء من المصدر إذا فشل التحميل
if [ "$DOWNLOAD_SUCCESS" = false ]; then
    print_warning "التحميل المباشر فشل — البناء من المصدر..."
    NAWA_SRC_DIR="$NAWA_INSTALL_DIR/src"
    mkdir -p "$NAWA_SRC_DIR"
    if [ -d "$NAWA_SRC_DIR/nawa-web-os" ]; then
        cd "$NAWA_SRC_DIR/nawa-web-os/nawa-rs" && git pull --quiet 2>/dev/null || true
    else
        git clone --depth 1 "$NAWA_REPO" "$NAWA_SRC_DIR/nawa-web-os" 2>&1 | tail -3
        cd "$NAWA_SRC_DIR/nawa-web-os/nawa-rs"
    fi
    print_step "بناء NAWA (5-10 دقائق)..."
    cargo build --release 2>&1 | tail -5
    cp target/release/nawad "$NAWAD_BINARY"
    cp target/release/nawa "$NAWA_CLI_BINARY"
    chmod +x "$NAWAD_BINARY" "$NAWA_CLI_BINARY"
    print_success "اكتمل البناء"
fi

# بناء WASM SSR module
print_step "إعداد WASM SSR module..."
WASM_MODULE="$NAWA_PLUGINS_DIR/nawa_ssr_demo.wasm"
if [ ! -f "$WASM_MODULE" ]; then
    if [ -d "$NAWA_SRC_DIR/nawa-web-os" ]; then
        cd "$NAWA_SRC_DIR/nawa-web-os/nawa-rs/examples/wasm-ssr-module"
        rustup target add wasm32-unknown-unknown 2>&1 | tail -1
        cargo build --release --target wasm32-unknown-unknown 2>&1 | tail -2
        cp target/wasm32-unknown-unknown/release/nawa_ssr_demo.wasm "$WASM_MODULE"
        print_success "WASM SSR module جاهز"
    fi
else
    print_success "WASM SSR module موجود"
fi

# إنشاء قوالب المشاريع
print_step "إنشاء قوالب المشاريع..."
mkdir -p "$NAWA_TEMPLATES_DIR/basic/data" "$NAWA_TEMPLATES_DIR/basic/plugins" "$NAWA_TEMPLATES_DIR/basic/static"
cat > "$NAWA_TEMPLATES_DIR/basic/nawa.toml" << 'EOF'
addr = "127.0.0.1:8080"
data_dir = "./data"
plugins_dir = "./plugins"
static_dir = "./static"
jwt_secret = "change-this-secret"
rate_limit = 100
wal_sync = true
log_level = "info"
EOF
cat > "$NAWA_TEMPLATES_DIR/basic/README.md" << 'EOF'
# مشروع NAWA
## التشغيل
nawad serve
## الميزات
- Dashboard: http://localhost:8080
- AION SEO: http://localhost:8080/__photon__
- WASM SSR: POST http://localhost:8080/api/wasm-ssr
EOF
print_success "القوالب جاهزة"

# إضافة لـ PATH
print_step "إضافة NAWA لـ PATH..."
SHELL_NAME="$(basename "$SHELL")"
case "$SHELL_NAME" in bash) PROFILE="$HOME/.bashrc";; zsh) PROFILE="$HOME/.zshrc";; *) PROFILE="$HOME/.profile";; esac
if ! grep -q "$NAWA_BIN_DIR" "$PROFILE" 2>/dev/null; then
    echo "export PATH=\"\$PATH:$NAWA_BIN_DIR\"" >> "$PROFILE"
    print_success "أُضيف لـ $PROFILE"
fi

# إنشاء أمر nawa الموحد
print_step "إنشاء أمر nawa الموحد..."
cat > "$NAWA_BIN_DIR/nawa" << SCRIPT
#!/bin/bash
NAWA_DIR="\${NAWA_INSTALL_DIR:-$NAWA_INSTALL_DIR}"
NAWA_BIN="\$NAWA_DIR/bin"
case "\${1:-help}" in
    serve) shift; exec "\$NAWA_BIN/nawad" serve "\$@";;
    new)
        PROJECT_NAME="\${2:-my-nawa-app}"
        PROJECT_DIR="\$(pwd)/\$PROJECT_NAME"
        echo "🚀 إنشاء مشروع NAWA: \$PROJECT_NAME"
        mkdir -p "\$PROJECT_DIR"
        cp -a "\$NAWA_DIR/templates/basic/." "\$PROJECT_DIR/" 2>/dev/null || true
        mkdir -p "\$PROJECT_DIR/plugins"
        cp "\$NAWA_DIR/plugins/nawa_ssr_demo.wasm" "\$PROJECT_DIR/plugins/" 2>/dev/null || true
        echo "✓ تم في: \$PROJECT_DIR"
        echo "الخطوات: cd \$PROJECT_NAME && nawad serve"
        echo "ثم افتح: http://localhost:8080"
        ;;
    build-wasm)
        cd "\$2" && rustup target add wasm32-unknown-unknown 2>/dev/null
        cargo build --release --target wasm32-unknown-unknown
        echo "✓ WASM جاهز: target/wasm32-unknown-unknown/release/"
        ;;
    info) exec "\$NAWA_BIN/nawad" info;;
    benchmark) exec "\$NAWA_BIN/nawad" benchmark "\${@:2}";;
    version) "\$NAWA_BIN/nawad" --version;;
    update) bash -c "\$(curl -fsSL https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/scripts/install.sh)";;
    help|--help|-h)
        echo "NAWA Web Operating System"
        echo ""
        echo "الأوامر:"
        echo "  serve [options]       تشغيل الخادم"
        echo "  new <name>            إنشاء مشروع جديد"
        echo "  build-wasm <path>     بناء WASM module"
        echo "  info                  معلومات النظام"
        echo "  benchmark [ops]       benchmark"
        echo "  version               الإصدار"
        echo "  update                تحديث NAWA"
        echo "  help                  هذه المساعدة"
        echo ""
        echo "أمثلة:"
        echo "  nawa serve                    # تشغيل على 8080"
        echo "  nawa new my-app               # مشروع جديد"
        echo "  nawa build-wasm ./module      # بناء WASM"
        ;;
    *) echo "أمر غير معروف: \$1"; echo "استخدم: nawa help"; exit 1;;
esac
SCRIPT
chmod +x "$NAWA_BIN_DIR/nawa"
print_success "أمر nawa جاهز"

# التحقق
print_step "التحقق..."
"$NAWAD_BINARY" --version &>/dev/null && print_success "nawad يعمل" || { print_error "nawad لا يعمل"; exit 1; }

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ تم تثبيت NAWA بنجاح!                         ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${YELLOW}الخطوات التالية:${NC}"
echo -e "  1. ${GREEN}source $PROFILE${NC}"
echo -e "  2. ${GREEN}nawa new my-first-app${NC}"
echo -e "  3. ${GREEN}cd my-first-app && nawad serve${NC}"
echo -e "  4. ${GREEN}افتح: http://localhost:8080${NC}"
echo ""
echo -e "  ${BLUE}NAWA جاهز! 🦀${NC}"
echo ""
