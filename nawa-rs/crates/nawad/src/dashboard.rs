//! Dashboard renderer — beautiful, professional, built-in.
//!
//! Renders the main dashboard using nawa-engine's design system.
//! Zero-copy HTML generation, no external CSS frameworks.

use nawa_db::DbEngine;
use nawa_auth::AuthStore;
use nawa_uring::NawaUring;

/// Render the main dashboard — the home page of NAWA.
pub fn render_dashboard(db: &DbEngine, auth: &AuthStore, uring: &NawaUring) -> String {
    let db_stats = db.stats();
    let _uring_stats = uring.stats();
    let user_count = auth.user_count();

    // Build stats HTML.
    let stats_html = format!(r#"
    <div class="nawa-stats">
        <div class="nawa-stat"><div class="nawa-stat-val">{keys}</div><div class="nawa-stat-label">DB Keys</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{users}</div><div class="nawa-stat-label">Users</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{puts}</div><div class="nawa-stat-label">DB Puts</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{gets}</div><div class="nawa-stat-label">DB Gets</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{real}</div><div class="nawa-stat-label">io_uring</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{entries}</div><div class="nawa-stat-label">Ring Entries</div></div>
    </div>"#,
        keys = db.len(),
        users = user_count,
        puts = db_stats.puts,
        gets = db_stats.gets,
        real = if uring.is_real_uring() { "✓" } else { "—" },
        entries = uring.config().entries,
    );

    // Build user table (if any users exist).
    let users_table = if user_count > 0 {
        let users = auth.list_users().unwrap_or_default();
        let rows: String = users.iter().map(|u| {
            let role_badge = if u.role == "admin" {
                r#"<span class="nawa-badge nawa-badge-danger">admin</span>"#
            } else {
                r#"<span class="nawa-badge nawa-badge-info">user</span>"#
            };
            let verified_badge = if u.verified {
                r#"<span class="nawa-badge nawa-badge-ok">موثّق</span>"#
            } else {
                r#"<span class="nawa-badge nawa-badge-warn">غير موثّق</span>"#
            };
            format!(r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                u.username, u.email, role_badge, verified_badge,
                u.created_at.split('T').next().unwrap_or("?"))
        }).collect::<Vec<_>>().join("\n");

        format!(r#"<div class="nawa-card"><h2>إدارة المستخدمين</h2>
        <table class="nawa-table"><thead><tr><th>الاسم</th><th>البريد</th><th>الدور</th><th>الحالة</th><th>تاريخ الإنشاء</th></tr></thead><tbody>{rows}</tbody></table></div>"#, rows = rows)
    } else {
        r#"<div class="nawa-card"><h2>مرحباً في NAWA!</h2><p>أول مستخدم يُسجّل يصبح أدمن تلقائياً. <a href="/register" class="nawa-btn nawa-btn-primary">إنشاء حساب أدمن</a></p></div>"#.to_string()
    };

    // Build DB entries table.
    let entries = db.scan_prefix("", 50);
    let db_table = if entries.is_empty() {
        r#"<div class="nawa-card"><h2>قاعدة البيانات</h2><p>لا توجد بيانات بعد. استخدم <code>POST /:key</code> لإضافة.</p></div>"#.to_string()
    } else {
        let rows: String = entries.iter().map(|(k, v)| {
            let key = String::from_utf8_lossy(k);
            let val: String = v.display().chars().take(60).collect();
            format!(r#"<tr><td>{key}</td><td>{val}</td></tr>"#)
        }).collect::<Vec<_>>().join("\n");
        format!(r#"<div class="nawa-card"><h2>قاعدة البيانات ({count} مفتاح)</h2>
        <table class="nawa-table"><thead><tr><th>المفتاح</th><th>القيمة</th></tr></thead><tbody>{rows}</tbody></table></div>"#,
            count = entries.len(), rows = rows)
    };

    // Complete HTML page.
    format!(r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NAWA — نظام تشغيل الويب</title>
    <style>{css}</style>
</head>
<body>
    <nav class="nawa-nav">
        <a class="nawa-nav-brand" href="/">🦀 NAWA</a>
        <div class="nawa-nav-links">
            <a href="/">Dashboard</a>
            <a href="/ssr">SSR Demo</a>
            <a href="/health">Health</a>
            <a href="/metrics">Metrics</a>
            <a href="/api">API</a>
            {auth_links}
        </div>
    </nav>

    <div class="nawa-container nawa-fade-in">
        <div class="nawa-card">
            <h1>🦀 NAWA Web Operating System</h1>
            <p>نظام تشغيل ويب ثوري مكتوب بـ Rust — يعمل في binary واحد بدون أي تبعيات خارجية.</p>
            {stats_html}
        </div>

        {users_table}
        {db_table}

        <div class="nawa-card">
            <h2>API Endpoints</h2>
            <table class="nawa-table">
                <thead><tr><th>Method</th><th>Endpoint</th><th>Description</th></tr></thead>
                <tbody>
                    <tr><td>GET</td><td>/</td><td>Dashboard</td></tr>
                    <tr><td>GET</td><td>/register</td><td>Registration page</td></tr>
                    <tr><td>POST</td><td>/register</td><td>Register + auto-login</td></tr>
                    <tr><td>GET</td><td>/login</td><td>Login page</td></tr>
                    <tr><td>POST</td><td>/login</td><td>Login + JWT cookie</td></tr>
                    <tr><td>GET</td><td>/logout</td><td>Logout</td></tr>
                    <tr><td>GET</td><td>/ssr</td><td>SSR demo (unified engine)</td></tr>
                    <tr><td>GET</td><td>/health</td><td>Health check (JSON)</td></tr>
                    <tr><td>GET</td><td>/uring</td><td>io_uring stats (JSON)</td></tr>
                    <tr><td>GET</td><td>/metrics</td><td>Prometheus metrics</td></tr>
                    <tr><td>GET</td><td>/plugins</td><td>WASM plugins list</td></tr>
                    <tr><td>GET</td><td>/:key</td><td>Get DB value</td></tr>
                    <tr><td>POST</td><td>/:key</td><td>Store DB value</td></tr>
                    <tr><td>DELETE</td><td>/:key</td><td>Delete DB value</td></tr>
                    <tr><td>GET</td><td>/scan/:prefix</td><td>Scan keys with prefix</td></tr>
                    <tr><td>POST</td><td>/auth/register</td><td>Register (JSON API)</td></tr>
                    <tr><td>POST</td><td>/auth/login</td><td>Login (JSON API)</td></tr>
                    <tr><td>GET</td><td>/auth/me</td><td>Current user (Bearer token)</td></tr>
                    <tr><td>GET</td><td>/auth/users</td><td>List users (admin only)</td></tr>
                </tbody>
            </table>
        </div>

        <div class="nawa-card">
            <h2>التقنيات المدمجة</h2>
            <p>كل هذا في binary واحد بحجم 7 MB:</p>
            <div class="nawa-stats">
                <div class="nawa-stat"><div class="nawa-stat-val">Rust</div><div class="nawa-stat-label">100%</div></div>
                <div class="nawa-stat"><div class="nawa-stat-val">0</div><div class="nawa-stat-label">External deps</div></div>
                <div class="nawa-stat"><div class="nawa-stat-val">io_uring</div><div class="nawa-stat-label">Zero-copy I/O</div></div>
                <div class="nawa-stat"><div class="nawa-stat-val">WASM</div><div class="nawa-stat-label">Plugin sandbox</div></div>
                <div class="nawa-stat"><div class="nawa-stat-val">JWT</div><div class="nawa-stat-label">Auth + RBAC</div></div>
                <div class="nawa-stat"><div class="nawa-stat-val">Prometheus</div><div class="nawa-stat-label">Metrics</div></div>
            </div>
        </div>

        <footer style="text-align:center;padding:2rem;color:#555;border-top:1px solid #2a2a2a;margin-top:2rem;">
            <p>© 2026 NAWA Project · مبني بـ Rust · <a href="https://github.com/amir-helal-ali/nawa-web-os" style="color:#f59e0b">GitHub</a></p>
        </footer>
    </div>
</body>
</html>"#,
        css = DESIGN_CSS,
        auth_links = if user_count > 0 {
            r#"<a href="/logout">Logout</a>"#
        } else {
            r#"<a href="/register">Register</a><a href="/login">Login</a>"#
        },
        stats_html = stats_html,
        users_table = users_table,
        db_table = db_table,
    )
}

/// Render the registration page.
pub fn render_register() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>إنشاء حساب — NAWA</title><style>{css}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/login">Login</a></div></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in"><h1>إنشاء حساب جديد</h1><p style="color:#888;margin-bottom:1rem">أول مستخدم يصبح أدمن تلقائياً مع تسجيل دخول تلقائي.</p><form method="POST" action="/register"><label class="nawa-label">اسم المستخدم</label><input type="text" name="username" class="nawa-input" placeholder="username" required><label class="nawa-label">البريد الإلكتروني</label><input type="email" name="email" class="nawa-input" placeholder="email@example.com" required><label class="nawa-label">كلمة المرور</label><input type="password" name="password" class="nawa-input" placeholder="password" required><button type="submit" class="nawa-btn nawa-btn-primary" style="margin-top:1rem;width:100%">إنشاء حساب · دخول تلقائي</button></form></div></div></body></html>"#, css = DESIGN_CSS)
}

/// Render the login page.
pub fn render_login() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>تسجيل الدخول — NAWA</title><style>{css}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/register">Register</a></div></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in"><h1>تسجيل الدخول</h1><form method="POST" action="/login" style="margin-top:1rem"><label class="nawa-label">البريد الإلكتروني</label><input type="email" name="email" class="nawa-input" placeholder="email@example.com" required><label class="nawa-label">كلمة المرور</label><input type="password" name="password" class="nawa-input" placeholder="password" required><button type="submit" class="nawa-btn nawa-btn-primary" style="margin-top:1rem;width:100%">دخول</button></form></div></div></body></html>"#, css = DESIGN_CSS)
}

/// Render an error page.
pub fn render_error(msg: &str, back_url: &str) -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><title>خطأ — NAWA</title><style>{css}</style></head><body><div class="nawa-container"><div class="nawa-card nawa-fade-in" style="text-align:center"><h1 style="color:#dc2626">⚠ {msg}</h1><a href="{back_url}" class="nawa-btn nawa-btn-secondary" style="margin-top:1rem">العودة</a></div></div></body></html>"#, css = DESIGN_CSS, msg = msg, back_url = back_url)
}

/// NAWA Design System CSS — professional dark amber RTL theme.
const DESIGN_CSS: &str = r#"
:root {
  --nawa-bg: #0d0c0a; --nawa-surface: #1a1a1a; --nawa-surface-hover: #222;
  --nawa-border: #2a2a2a; --nawa-text: #e0e0e0; --nawa-text-muted: #888;
  --nawa-primary: #f59e0b; --nawa-primary-hover: #d97706;
  --nawa-accent: #10b981; --nawa-danger: #dc2626;
  --nawa-radius: 12px; --nawa-radius-sm: 8px; --nawa-transition: 0.2s ease;
}
* { margin:0; padding:0; box-sizing:border-box; }
body { font-family:'Noto Sans Arabic',system-ui,sans-serif; background:var(--nawa-bg); color:var(--nawa-text); line-height:1.8; min-height:100vh; -webkit-font-smoothing:antialiased; }
.nawa-nav { display:flex; justify-content:space-between; align-items:center; padding:1rem 2rem; background:var(--nawa-surface); border-bottom:1px solid var(--nawa-border); position:sticky; top:0; z-index:100; backdrop-filter:blur(12px); }
.nawa-nav-brand { color:var(--nawa-primary); font-weight:700; font-size:1.25rem; text-decoration:none; }
.nawa-nav-links { display:flex; gap:1rem; }
.nawa-nav-links a { color:var(--nawa-primary); text-decoration:none; transition:var(--nawa-transition); }
.nawa-nav-links a:hover { opacity:0.7; }
.nawa-container { max-width:900px; margin:0 auto; padding:2rem; }
.nawa-card { background:var(--nawa-surface); border:1px solid var(--nawa-border); border-radius:var(--nawa-radius); padding:1.5rem; margin:1rem 0; transition:border-color var(--nawa-transition); }
.nawa-card:hover { border-color:var(--nawa-primary); }
.nawa-card h1 { color:var(--nawa-primary); margin-bottom:0.5rem; }
.nawa-card h2 { color:var(--nawa-primary); margin-bottom:0.75rem; }
.nawa-btn { display:inline-flex; align-items:center; gap:0.5rem; padding:0.75rem 1.5rem; border:none; border-radius:var(--nawa-radius-sm); cursor:pointer; font-size:1rem; font-family:inherit; transition:all var(--nawa-transition); text-decoration:none; }
.nawa-btn-primary { background:var(--nawa-primary); color:var(--nawa-bg); font-weight:700; }
.nawa-btn-primary:hover { background:var(--nawa-primary-hover); transform:translateY(-1px); }
.nawa-btn-secondary { background:var(--nawa-surface-hover); color:var(--nawa-text); border:1px solid var(--nawa-border); }
.nawa-input { width:100%; padding:0.8rem; margin:0.5rem 0; background:var(--nawa-bg); border:1px solid var(--nawa-border); border-radius:var(--nawa-radius-sm); color:var(--nawa-text); font-size:1rem; font-family:inherit; transition:var(--nawa-transition); }
.nawa-input:focus { border-color:var(--nawa-primary); outline:none; box-shadow:0 0 0 3px rgba(245,158,11,0.15); }
.nawa-label { display:block; margin:0.8rem 0 0.3rem; color:var(--nawa-text-muted); font-size:0.9rem; }
.nawa-table { width:100%; border-collapse:collapse; margin:1rem 0; }
.nawa-table th, .nawa-table td { padding:0.7rem; text-align:right; border-bottom:1px solid var(--nawa-border); }
.nawa-table th { color:var(--nawa-primary); font-size:0.85rem; }
.nawa-table tr:hover { background:var(--nawa-surface-hover); }
.nawa-badge { display:inline-block; padding:0.2rem 0.6rem; border-radius:4px; font-size:0.75rem; }
.nawa-badge-ok { background:rgba(16,185,129,0.15); color:var(--nawa-accent); }
.nawa-badge-warn { background:rgba(245,158,11,0.15); color:var(--nawa-primary); }
.nawa-badge-danger { background:rgba(220,38,38,0.15); color:var(--nawa-danger); }
.nawa-badge-info { background:rgba(59,130,246,0.15); color:#3b82f6; }
.nawa-stats { display:flex; flex-wrap:wrap; gap:2rem; margin:1rem 0; }
.nawa-stat { text-align:center; }
.nawa-stat-val { color:var(--nawa-primary); font-size:1.5rem; font-weight:700; }
.nawa-stat-label { color:var(--nawa-text-muted); font-size:0.8rem; }
@keyframes nawa-fade-in { from { opacity:0; transform:translateY(10px); } to { opacity:1; transform:translateY(0); } }
.nawa-fade-in { animation:nawa-fade-in 0.4s ease; }
code { background:var(--nawa-bg); padding:0.2rem 0.4rem; border-radius:4px; color:var(--nawa-accent); font-family:monospace; }
@media (max-width:768px) { .nawa-nav { flex-direction:column; gap:0.5rem; } .nawa-container { padding:1rem; } .nawa-stats { flex-direction:column; gap:1rem; } }
"#;
