#!/bin/bash
# NAWA Web Operating System — Installer
# يبني النظام من المصدر — دائماً أحدث إصدار
set -e

G='\033[0;32m'; B='\033[0;34m'; Y='\033[1;33m'; N='\033[0m'
DIR="$HOME/.nawa"
REPO="https://github.com/amir-helal-ali/nawa-web-os"

p() { echo -e "${B}[$(date +%H:%M:%S)] $1${N}"; }
s() { echo -e "${G}[$(date +%H:%M:%S)] ✓ $1${N}"; }

echo -e "${B}╔══════════════════════════════════════════════╗${N}"
echo -e "${B}║  NAWA Web Operating System — Installer        ║${N}"
echo -e "${B}╚══════════════════════════════════════════════╝${N}"

# 1. حذف النسخة القديمة
p "حذف النسخة القديمة..."
pkill -f "nawad" 2>/dev/null || true
rm -rf "$DIR"
sed -i '/\.nawa/d' "$HOME/.bashrc" 2>/dev/null || true
s "تم التنظيف"

# 2. Rust
p "فحص Rust..."
if ! command -v cargo &>/dev/null; then
    p "تثبيت Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1 | tail -2
    source "$HOME/.cargo/env"
fi
s "Rust: $(cargo --version)"

# 3. تحميل الكود
p "تحميل الكود من GitHub..."
mkdir -p "$DIR/src"
cd "$DIR/src"
if [ -d "nawa-web-os" ]; then
    cd nawa-web-os/nawa-rs && git pull --quiet 2>/dev/null || true
else
    git clone --depth 1 "$REPO" nawa-web-os 2>&1 | tail -2
    cd nawa-web-os/nawa-rs
fi
s "الكود جاهز"

# 4. بناء
p "بناء NAWA (5-10 دقائق)..."
cargo build --release 2>&1 | tail -2
mkdir -p "$DIR/bin" "$DIR/plugins"
cp target/release/nawad "$DIR/bin/"
cp target/release/nawa "$DIR/bin/"
s "nawad + nawa-cli جاهزان"

# 5. WASM
p "بناء WASM SSR module..."
rustup target add wasm32-unknown-unknown 2>&1 | tail -1
cd examples/wasm-ssr-module
cargo build --release --target wasm32-unknown-unknown 2>&1 | tail -1
cp target/wasm32-unknown-unknown/release/nawa_ssr_demo.wasm "$DIR/plugins/"
cd "$DIR/src/nawa-web-os/nawa-rs"
s "WASM module جاهز"

# 6. القوالب
p "إنشاء القوالب..."
mkdir -p "$DIR/templates/basic"/{data,plugins,static}
cp templates/basic/nawa.toml "$DIR/templates/basic/" 2>/dev/null || true
cp templates/basic/README.md "$DIR/templates/basic/" 2>/dev/null || true
s "القوالب جاهزة"

# 7. PATH
echo "export PATH=\"\$HOME/.nawa/bin:\$PATH\"" >> "$HOME/.bashrc"

# 8. أمر nawa
cat > "$DIR/bin/nawa" << 'CMD'
#!/bin/bash
D="$HOME/.nawa"; B="$D/bin"
case "${1:-help}" in
    serve) shift; exec "$B/nawad" serve "$@";;
    new) P="${2:-my-app}"; D2="$(pwd)/$P"; echo "🚀 إنشاء: $P"; mkdir -p "$D2"; cp -a "$D/templates/basic/." "$D2/" 2>/dev/null; mkdir -p "$D2/plugins"; cp "$D/plugins/nawa_ssr_demo.wasm" "$D2/plugins/" 2>/dev/null; echo "✓ $D2"; echo "cd $P && nawad serve";;
    build-wasm) cd "$2" && rustup target add wasm32-unknown-unknown 2>/dev/null; cargo build --release --target wasm32-unknown-unknown; echo "✓ WASM جاهز";;
    info) exec "$B/nawad" info;;
    benchmark) exec "$B/nawad" benchmark "${@:2}";;
    version) "$B/nawad" --version;;
    update) bash -c "$(curl -fsSL https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/scripts/install.sh)";;
    uninstall) echo "🗑️ حذف..."; pkill -f nawad 2>/dev/null; rm -rf "$D"; sed -i '/\.nawa/d' ~/.bashrc; echo "✅ تم";;
    help|*) echo "NAWA — أوامر: serve, new, build-wasm, info, benchmark, version, update, uninstall";;
esac
CMD
chmod +x "$DIR/bin/nawa"

# 9. التحقق
export PATH="$DIR/bin:$PATH"
V=$(nawad --version 2>&1)
s "nawad: $V"

echo ""
echo -e "${G}╔══════════════════════════════════════════════╗${N}"
echo -e "${G}║  ✅ تم تثبيت NAWA بنجاح!                     ║${N}"
echo -e "${G}╚══════════════════════════════════════════════╝${N}"
echo ""
echo "  الخطوات:"
echo "  1. source ~/.bashrc"
echo "  2. nawa new my-app"
echo "  3. cd my-app && nawad serve"
echo "  4. افتح: http://localhost:8080"
echo ""
