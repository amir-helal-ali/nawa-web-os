# NAWA WASM SSR Demo Module

مثال على WASM module يُقدم SSR (Server-Side Rendering) لـ NAWA بدون Node.js.

## كيف يعمل

1. NAWA يكتب JSON props إلى ذاكرة WASM
2. WASM module يقرأ الـ props ويُولّد HTML
3. NAWA يقرأ HTML من ذاكرة WASM ويُقدمه للمستخدم

## الواجهة (WASM exports)

```wasm
export memory          ; ذاكرة WASM الخطية
export alloc(size) -> ptr    ; تخصيص buffer
export render(props_ptr, props_len) -> html_ptr  ; توليد HTML
```

## البناء

### المتطلبات
- Rust 1.75+
- WASM target: `rustup target add wasm32-unknown-unknown`

### خطوات البناء

```bash
# 1. أضف WASM target
rustup target add wasm32-unknown-unknown

# 2. ابنِ الـ module
cd examples/wasm-ssr-module
cargo build --release --target wasm32-unknown-unknown

# 3. الناتج: target/wasm32-unknown-unknown/release/nawa_ssr_demo.wasm
```

### الاستخدام مع NAWA

```bash
# 1. انسخ الـ WASM module إلى plugins dir
cp target/wasm32-unknown-unknown/release/nawa_ssr_demo.wasm ./plugins/

# 2. شغّل nawad
nawad serve --plugins-dir ./plugins

# 3. استدعِ الـ module عبر API
curl -X POST http://localhost:8080/api/ssr \
  -H "Content-Type: application/json" \
  -d '{"title":"Hello","description":"Test","items":["a","b"]}'
```

## تنسيق الـ Props (JSON)

```json
{
  "title": "عنوان الصفحة",
  "description": "وصف الصفحة",
  "items": ["عنصر 1", "عنصر 2", "عنصر 3"],
  "user": {
    "name": "أحمد",
    "role": "admin"
  }
}
```

## الناتج (HTML)

الـ module يُولّد HTML كامل مع:
- `<title>` و `<meta description>` للـ SEO
- CSS inline (dark theme عربي RTL)
- بنية semantic (`<article>`, `<section>`, `<footer>`)
- escaping للأحرف الخاصة (XSS protection)

## المزايا

- **بدون Node.js**: الـ WASM module يُجمّع من Rust مباشرة
- **سريع**: أسرع 10x من Node.js SSR
- **آمن**: WASM sandbox يحمي النظام
- **صغير**: ~50KB WASM module (مقارنة بـ 30MB Node.js)
- **SEO-friendly**: HTML كامل قبل أي JavaScript

## التخصيص

عدّل `src/lib.rs` لتغيير:
- بنية الـ props (`PageProps` struct)
- HTML template (`render_html` function)
- CSS styles

ثم أعد البناء: `cargo build --release --target wasm32-unknown-unknown`
