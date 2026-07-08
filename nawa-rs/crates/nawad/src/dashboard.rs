//! Dashboard renderer — professional, beautiful, fully integrated.
//!
//! Includes: main dashboard, register, login, settings, admin actions.

use nawa_db::DbEngine;
use nawa_auth::{AuthStore, User};
use nawa_uring::NawaUring;

/// Render the main dashboard.
pub fn render_dashboard(db: &DbEngine, auth: &AuthStore, uring: &NawaUring, current_user: Option<&User>) -> String {
    let db_stats = db.stats();
    let user_count = auth.user_count();

    let stats_html = format!(r#"
    <div class="nawa-stats">
        <div class="nawa-stat"><div class="nawa-stat-val">{keys}</div><div class="nawa-stat-label">DB Keys</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{users}</div><div class="nawa-stat-label">Users</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{puts}</div><div class="nawa-stat-label">DB Puts</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{gets}</div><div class="nawa-stat-label">DB Gets</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{real}</div><div class="nawa-stat-label">io_uring</div></div>
        <div class="nawa-stat"><div class="nawa-stat-val">{entries}</div><div class="nawa-stat-label">Ring Size</div></div>
    </div>"#,
        keys = db.len(), users = user_count, puts = db_stats.puts, gets = db_stats.gets,
        real = if uring.is_real_uring() { "✓" } else { "—" }, entries = uring.config().entries);

    // Auth links based on login state.
    let (auth_links, welcome_html) = if let Some(user) = current_user {
        let admin_link = if user.role == "admin" {
            r#"<a href="/settings">⚙ Settings</a>"#
        } else { "" };
        (
            format!(r#"<span style="color:#888">{name}</span> {admin} <a href="/logout">Logout</a>"#,
                name = user.username, admin = admin_link),
            format!(r#"<div class="nawa-card"><h1>🦀 مرحباً، {name}!</h1><p>دورك: <span class="nawa-badge nawa-badge-{badge}">{role}</span></p>{stats_html}</div>"#,
                name = user.username, role = user.role,
                badge = if user.role == "admin" { "danger" } else { "info" },
                stats_html = stats_html),
        )
    } else {
        (
            r#"<a href="/register">Register</a><a href="/login">Login</a>"#.to_string(),
            format!(r#"<div class="nawa-card"><h1>🦀 NAWA Web Operating System</h1><p>نظام تشغيل ويب ثوري مكتوب بـ Rust — يعمل في binary واحد بدون أي تبعيات خارجية.</p>{stats_html}<p style="margin-top:1rem"><a href="/register" class="nawa-btn nawa-btn-primary">ابدأ الآن — أول مستخدم = أدمن</a></p></div>"#, stats_html = stats_html),
        )
    };

    // User management table (admin only).
    let users_table = if user_count > 0 {
        if let Some(user) = current_user {
            if user.role == "admin" {
                let users = auth.list_users().unwrap_or_default();
                let rows: String = users.iter().map(|u| {
                    let role_badge = if u.role == "admin" {
                        r#"<span class="nawa-badge nawa-badge-danger">admin</span>"#
                    } else { r#"<span class="nawa-badge nawa-badge-info">user</span>"# };
                    let verified_badge = if u.verified {
                        r#"<span class="nawa-badge nawa-badge-ok">موثّق</span>"#
                    } else { r#"<span class="nawa-badge nawa-badge-warn">غير موثّق</span>"# };
                    let actions = if u.id != user.id {
                        format!(r#"
                        <form method="POST" action="/admin/verify" style="display:inline"><input type="hidden" name="user_id" value="{id}"><button type="submit" class="nawa-btn nawa-btn-sm" style="background:#10b981;color:#000">توثيق</button></form>
                        <form method="POST" action="/admin/role" style="display:inline"><input type="hidden" name="user_id" value="{id}"><input type="hidden" name="role" value="{new_role}"><button type="submit" class="nawa-btn nawa-btn-sm nawa-btn-secondary">{role_label}</button></form>
                        <form method="POST" action="/admin/delete" style="display:inline" onsubmit="return confirm('حذف {username}؟')"><input type="hidden" name="user_id" value="{id}"><button type="submit" class="nawa-btn nawa-btn-sm nawa-btn-danger">حذف</button></form>"#,
                            id = u.id, username = u.username,
                            new_role = if u.role == "admin" { "user" } else { "admin" },
                            role_label = if u.role == "admin" { "→ user" } else { "→ admin" })
                    } else { r#"<span style="color:#555">(أنت)</span>"#.to_string() };
                    format!(r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                        u.username, u.email, role_badge, verified_badge, actions)
                }).collect::<Vec<_>>().join("\n");
                format!(r#"<div class="nawa-card"><h2>👥 إدارة المستخدمين</h2><table class="nawa-table"><thead><tr><th>الاسم</th><th>البريد</th><th>الدور</th><th>الحالة</th><th>إجراءات</th></tr></thead><tbody>{rows}</tbody></table></div>"#, rows = rows)
            } else {
                String::new()
            }
        } else { String::new() }
    } else { String::new() };

    // DB entries table.
    let entries = db.scan_prefix("", 50);
    let db_table = if entries.is_empty() {
        r#"<div class="nawa-card"><h2>📊 قاعدة البيانات</h2><p>لا توجد بيانات. استخدم <code>POST /:key</code> لإضافة.</p></div>"#.to_string()
    } else {
        let rows: String = entries.iter().map(|(k, v)| {
            let key = String::from_utf8_lossy(k);
            let val: String = v.display().chars().take(60).collect();
            format!(r#"<tr><td>{key}</td><td>{val}</td></tr>"#)
        }).collect::<Vec<_>>().join("\n");
        format!(r#"<div class="nawa-card"><h2>📊 قاعدة البيانات ({count} مفتاح)</h2><table class="nawa-table"><thead><tr><th>المفتاح</th><th>القيمة</th></tr></thead><tbody>{rows}</tbody></table></div>"#, count = entries.len(), rows = rows)
    };

    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>NAWA — نظام تشغيل الويب</title><style>{CSS}</style></head><body>
<nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/">Dashboard</a><a href="/ssr">SSR</a><a href="/system">System</a><a href="/metrics">Metrics</a><a href="/api">API</a>{auth_links}</div></nav>
<div class="nawa-container nawa-fade-in">
{welcome_html}
{users_table}
{db_table}
<div class="nawa-card"><h2>🔧 التقنيات المدمجة</h2><div class="nawa-stats">
<div class="nawa-stat"><div class="nawa-stat-val">Rust</div><div class="nawa-stat-label">100%</div></div>
<div class="nawa-stat"><div class="nawa-stat-val">0</div><div class="nawa-stat-label">External deps</div></div>
<div class="nawa-stat"><div class="nawa-stat-val">io_uring</div><div class="nawa-stat-label">Zero-copy</div></div>
<div class="nawa-stat"><div class="nawa-stat-val">WASM</div><div class="nawa-stat-label">Plugins</div></div>
<div class="nawa-stat"><div class="nawa-stat-val">JWT</div><div class="nawa-stat-label">Auth+RBAC</div></div>
<div class="nawa-stat"><div class="nawa-stat-val">WS</div><div class="nawa-stat-label">Real-time</div></div>
</div></div>
<footer><p>© 2026 NAWA · <a href="https://github.com/amir-helal-ali/nawa-web-os">GitHub</a></p></footer>
<div id="nawa-notifications" style="position:fixed;bottom:1rem;left:1rem;z-index:9999;display:flex;flex-direction:column;gap:0.5rem"></div>
</div>
<script>{WS_JS}</script>
</body></html>"#,
        auth_links = auth_links, welcome_html = welcome_html, users_table = users_table, db_table = db_table,
        WS_JS = WS_CLIENT_JS)
}

/// Render registration page.
pub fn render_register() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>إنشاء حساب — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/login">Login</a></div></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in"><h1>إنشاء حساب جديد</h1><p style="color:#888;margin-bottom:1rem">أول مستخدم يصبح أدمن تلقائياً مع تسجيل دخول تلقائي.</p><form method="POST" action="/register"><label class="nawa-label">اسم المستخدم</label><input type="text" name="username" class="nawa-input" placeholder="username" required><label class="nawa-label">البريد الإلكتروني</label><input type="email" name="email" class="nawa-input" placeholder="email@example.com" required><label class="nawa-label">كلمة المرور</label><input type="password" name="password" class="nawa-input" placeholder="password" required><button type="submit" class="nawa-btn nawa-btn-primary" style="margin-top:1rem;width:100%">إنشاء حساب · دخول تلقائي</button></form></div></div></body></html>"#)
}

/// Render login page.
pub fn render_login() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>تسجيل الدخول — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/register">Register</a></div></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in"><h1>تسجيل الدخول</h1><form method="POST" action="/login" style="margin-top:1rem"><label class="nawa-label">البريد الإلكتروني</label><input type="email" name="email" class="nawa-input" placeholder="email@example.com" required><label class="nawa-label">كلمة المرور</label><input type="password" name="password" class="nawa-input" placeholder="password" required><button type="submit" class="nawa-btn nawa-btn-primary" style="margin-top:1rem;width:100%">دخول</button></form></div></div></body></html>"#)
}

/// Render settings page (admin only).
pub fn render_settings(auth: &AuthStore, current_user: &User) -> String {
    let settings = auth.get_settings().unwrap_or_default();
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>الإعدادات — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/">Dashboard</a><span style="color:#888">{name}</span> <a href="/logout">Logout</a></div></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in"><h1>⚙ إعدادات المشروع</h1><form method="POST" action="/settings"><label class="nawa-label">اسم المشروع</label><input type="text" name="project_name" class="nawa-input" value="{project_name}"><label class="nawa-label"><input type="checkbox" name="registration_open" {reg_open} style="display:inline;width:auto;margin-left:0.5rem"> التسجيل مفتوح</label><label class="nawa-label"><input type="checkbox" name="verification_required" {ver_req} style="display:inline;width:auto;margin-left:0.5rem"> التوثيق إلزامي للمستخدمين الجدد</label><label class="nawa-label">الحد الأقصى للمستخدمين (فارغ = بلا حد)</label><input type="number" name="max_users" class="nawa-input" value="{max_users}"><label class="nawa-label">مدة صلاحية JWT (ثانية)</label><input type="number" name="jwt_expiry_secs" class="nawa-input" value="{jwt_expiry}"><button type="submit" class="nawa-btn nawa-btn-primary" style="margin-top:1rem">💾 حفظ الإعدادات</button></form></div></div></body></html>"#,
        name = current_user.username,
        project_name = settings.project_name,
        reg_open = if settings.registration_open { "checked" } else { "" },
        ver_req = if settings.verification_required { "checked" } else { "" },
        max_users = settings.max_users.map(|m| m.to_string()).unwrap_or_default(),
        jwt_expiry = settings.jwt_expiry_secs)
}

/// Render error page.
pub fn render_error(msg: &str, back_url: &str) -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><title>خطأ — NAWA</title><style>{CSS}</style></head><body><div class="nawa-container"><div class="nawa-card nawa-fade-in" style="text-align:center"><h1 style="color:#dc2626">⚠ {msg}</h1><a href="{back_url}" class="nawa-btn nawa-btn-secondary" style="margin-top:1rem">العودة</a></div></div></body></html>"#, msg = msg, back_url = back_url)
}


/// Render a beautiful 404 page.
#[allow(dead_code)]
pub fn render_404(path: &str) -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>404 — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in" style="text-align:center;padding:3rem"><h1 style="font-size:4rem;color:var(--nawa-primary)">404</h1><p style="font-size:1.2rem;color:var(--nawa-text-muted);margin:1rem 0">الصفحة غير موجودة</p><code style="color:var(--nawa-text-muted)">{path}</code><br><a href="/" class="nawa-btn nawa-btn-primary" style="margin-top:2rem">← العودة للرئيسية</a></div></div></body></html>"#, path = path)
}

/// Render a beautiful 500 page.
#[allow(dead_code)]
pub fn render_500(error: &str) -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>خطأ — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in" style="text-align:center;padding:3rem"><h1 style="font-size:4rem;color:var(--nawa-danger)">⚠</h1><p style="font-size:1.2rem;margin:1rem 0">خطأ داخلي في الخادم</p><code style="color:var(--nawa-text-muted);word-break:break-all">{error}</code><br><a href="/" class="nawa-btn nawa-btn-secondary" style="margin-top:2rem">← العودة</a></div></div></body></html>"#, error = error)
}

/// Render password reset request page.
pub fn render_password_reset() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>استعادة كلمة المرور — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/login">Login</a></div></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in"><h1>🔑 استعادة كلمة المرور</h1><p style="color:#888;margin-bottom:1rem">أدخل بريدك الإلكتروني لاستلام رمز الاستعادة.</p><form method="POST" action="/password-reset"><label class="nawa-label">البريد الإلكتروني</label><input type="email" name="email" class="nawa-input" placeholder="email@example.com" required><button type="submit" class="nawa-btn nawa-btn-primary" style="margin-top:1rem;width:100%">إرسال رمز الاستعادة</button></form></div></div></body></html>"#)
}

/// Render password reset confirmation page.
pub fn render_password_reset_confirm(email: &str) -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>رمز الاستعادة — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a></nav><div class="nawa-container"><div class="nawa-card nawa-fade-in"><h1>✅ تم إرسال الرمز</h1><p style="color:#888;margin-bottom:1rem">تم إرسال رمز الاستعادة إلى <strong>{email}</strong>.</p><p style="color:var(--nawa-text-muted);font-size:0.9rem">في النظام الحقيقي، سيُرسل الرمز عبر البريد. في النواة الحالية، استخدم الـ API:</p><code style="display:block;padding:1rem;margin:1rem 0;background:var(--nawa-bg);border-radius:8px">POST /auth/reset-password&#10;{{"email":"{email}","new_password":"..."}}</code><a href="/login" class="nawa-btn nawa-btn-primary">العودة لتسجيل الدخول</a></div></div></body></html>"#, email = email)
}

/// WebSocket client JS — real-time push notifications (NO polling).
const WS_CLIENT_JS: &str = r#"
(function(){
    var wsPort = parseInt(window.location.port || "8080") + 1;
    var wsUrl = "ws://" + window.location.hostname + ":" + wsPort;
    var container = document.getElementById("nawa-notifications");
    var reconnectDelay = 1000;
    function connect(){
        var ws = new WebSocket(wsUrl);
        ws.onopen = function(){ reconnectDelay=1000; showNotif("🟢","WebSocket connected","ok"); };
        ws.onmessage = function(ev){
            try{ var d=JSON.parse(ev.data); handleNotif(d); }catch(e){}
        };
        ws.onclose = function(){
            setTimeout(connect, reconnectDelay);
            reconnectDelay = Math.min(reconnectDelay*2, 10000);
        };
        ws.onerror = function(){ ws.close(); };
    }
    function handleNotif(d){
        if(d.event==="connected") return;
        var icon="📢", cls="info";
        if(d.event==="user_registered") icon="👤";
        else if(d.event==="user_login") icon="🔑";
        else if(d.event==="db_write") icon="💾";
        else if(d.event==="db_delete") icon="🗑";
        else if(d.event==="user_verified") icon="✅";
        else if(d.event==="user_deleted") icon="⚠";
        else if(d.event==="role_changed") icon="🔄";
        else if(d.event==="settings_updated") icon="⚙";
        else if(d.event==="profile_updated") icon="📝";
        showNotif(icon+" "+d.event, JSON.stringify(d.data), cls);
    }
    function showNotif(title, body, cls){
        var div=document.createElement("div");
        div.style.cssText="padding:0.8rem 1.2rem;background:var(--nawa-surface);border:1px solid var(--nawa-border);border-radius:8px;color:var(--nawa-text);font-size:0.85rem;max-width:350px;animation:nawa-fade-in 0.3s;box-shadow:0 4px 12px rgba(0,0,0,0.5)";
        if(cls==="ok") div.style.borderColor="var(--nawa-accent)";
        div.innerHTML="<strong>"+title+"</strong><br><span style='color:var(--nawa-text-muted);font-size:0.75rem'>"+body+"</span>";
        container.appendChild(div);
        setTimeout(function(){ div.style.transition="opacity 0.5s"; div.style.opacity="0"; setTimeout(function(){div.remove();},500); },5000);
    }
    connect();
})();
"#;

/// NAWA Design System CSS.
const CSS: &str = r#"
:root{--nawa-bg:#0d0c0a;--nawa-surface:#1a1a1a;--nawa-surface-hover:#222;--nawa-border:#2a2a2a;--nawa-text:#e0e0e0;--nawa-text-muted:#888;--nawa-primary:#f59e0b;--nawa-primary-hover:#d97706;--nawa-accent:#10b981;--nawa-danger:#dc2626;--nawa-radius:12px;--nawa-radius-sm:8px;--nawa-transition:0.2s ease}
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:'Noto Sans Arabic',system-ui,sans-serif;background:var(--nawa-bg);color:var(--nawa-text);line-height:1.8;min-height:100vh;-webkit-font-smoothing:antialiased}
.nawa-nav{display:flex;justify-content:space-between;align-items:center;padding:1rem 2rem;background:var(--nawa-surface);border-bottom:1px solid var(--nawa-border);position:sticky;top:0;z-index:100;backdrop-filter:blur(12px)}
.nawa-nav-brand{color:var(--nawa-primary);font-weight:700;font-size:1.25rem;text-decoration:none}
.nawa-nav-links{display:flex;align-items:center;gap:1rem}
.nawa-nav-links a{color:var(--nawa-primary);text-decoration:none;transition:var(--nawa-transition)}
.nawa-nav-links a:hover{opacity:0.7}
.nawa-container{max-width:900px;margin:0 auto;padding:2rem}
.nawa-card{background:var(--nawa-surface);border:1px solid var(--nawa-border);border-radius:var(--nawa-radius);padding:1.5rem;margin:1rem 0;transition:border-color var(--nawa-transition)}
.nawa-card:hover{border-color:var(--nawa-primary)}
.nawa-card h1{color:var(--nawa-primary);margin-bottom:0.5rem}
.nawa-card h2{color:var(--nawa-primary);margin-bottom:0.75rem}
.nawa-btn{display:inline-flex;align-items:center;gap:0.5rem;padding:0.75rem 1.5rem;border:none;border-radius:var(--nawa-radius-sm);cursor:pointer;font-size:1rem;font-family:inherit;transition:all var(--nawa-transition);text-decoration:none}
.nawa-btn-primary{background:var(--nawa-primary);color:var(--nawa-bg);font-weight:700}
.nawa-btn-primary:hover{background:var(--nawa-primary-hover);transform:translateY(-1px)}
.nawa-btn-secondary{background:var(--nawa-surface-hover);color:var(--nawa-text);border:1px solid var(--nawa-border)}
.nawa-btn-danger{background:var(--nawa-danger);color:#fff}
.nawa-btn-sm{padding:0.3rem 0.8rem;font-size:0.8rem}
.nawa-input{width:100%;padding:0.8rem;margin:0.5rem 0;background:var(--nawa-bg);border:1px solid var(--nawa-border);border-radius:var(--nawa-radius-sm);color:var(--nawa-text);font-size:1rem;font-family:inherit;transition:var(--nawa-transition)}
.nawa-input:focus{border-color:var(--nawa-primary);outline:none;box-shadow:0 0 0 3px rgba(245,158,11,0.15)}
.nawa-label{display:block;margin:0.8rem 0 0.3rem;color:var(--nawa-text-muted);font-size:0.9rem}
.nawa-table{width:100%;border-collapse:collapse;margin:1rem 0}
.nawa-table th,.nawa-table td{padding:0.7rem;text-align:right;border-bottom:1px solid var(--nawa-border)}
.nawa-table th{color:var(--nawa-primary);font-size:0.85rem}
.nawa-table tr:hover{background:var(--nawa-surface-hover)}
.nawa-badge{display:inline-block;padding:0.2rem 0.6rem;border-radius:4px;font-size:0.75rem}
.nawa-badge-ok{background:rgba(16,185,129,0.15);color:var(--nawa-accent)}
.nawa-badge-warn{background:rgba(245,158,11,0.15);color:var(--nawa-primary)}
.nawa-badge-danger{background:rgba(220,38,38,0.15);color:var(--nawa-danger)}
.nawa-badge-info{background:rgba(59,130,246,0.15);color:#3b82f6}
.nawa-stats{display:flex;flex-wrap:wrap;gap:2rem;margin:1rem 0}
.nawa-stat{text-align:center}
.nawa-stat-val{color:var(--nawa-primary);font-size:1.5rem;font-weight:700}
.nawa-stat-label{color:var(--nawa-text-muted);font-size:0.8rem}
@keyframes nawa-fade-in{from{opacity:0;transform:translateY(10px)}to{opacity:1;transform:translateY(0)}}
.nawa-fade-in{animation:nawa-fade-in 0.4s ease}
code{background:var(--nawa-bg);padding:0.2rem 0.4rem;border-radius:4px;color:var(--nawa-accent);font-family:monospace}
footer{text-align:center;padding:2rem;color:#555;border-top:1px solid var(--nawa-border);margin-top:2rem}
footer a{color:var(--nawa-primary);text-decoration:none}
@media(max-width:768px){.nawa-nav{flex-direction:column;gap:0.5rem}.nawa-container{padding:1rem}.nawa-stats{flex-direction:column;gap:1rem}}
"#;

/// Render user profile page (view + edit).
pub fn render_profile(user: &User) -> String {
    let verified_badge = if user.verified {
        r#"<span class="nawa-badge nawa-badge-ok">موثّق</span>"#
    } else {
        r#"<span class="nawa-badge nawa-badge-warn">غير موثّق</span>"#
    };
    let role_badge = if user.role == "admin" {
        r#"<span class="nawa-badge nawa-badge-danger">admin</span>"#
    } else {
        r#"<span class="nawa-badge nawa-badge-info">user</span>"#
    };
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>البروفايل — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/">Dashboard</a><a href="/profile">Profile</a><span style="color:#888">{name}</span> <a href="/logout">Logout</a></div></nav><div class="nawa-container nawa-fade-in"><div class="nawa-card"><h1>👤 البروفايل</h1><div class="nawa-stats"><div class="nawa-stat"><div class="nawa-stat-val">{username}</div><div class="nawa-stat-label">اسم المستخدم</div></div><div class="nawa-stat"><div class="nawa-stat-val">{role_badge}</div><div class="nawa-stat-label">الدور</div></div><div class="nawa-stat"><div class="nawa-stat-val">{verified_badge}</div><div class="nawa-stat-label">الحالة</div></div></div></div><div class="nawa-card"><h2>📝 تعديل البيانات</h2><form method="POST" action="/profile"><label class="nawa-label">اسم المستخدم</label><input type="text" name="username" class="nawa-input" value="{username}" required><label class="nawa-label">البريد الإلكتروني</label><input type="email" name="email" class="nawa-input" value="{email}" required><label class="nawa-label">كلمة مرور جديدة (اتركها فارغة لعدم التغيير)</label><input type="password" name="new_password" class="nawa-input" placeholder="••••••"><button type="submit" class="nawa-btn nawa-btn-primary" style="margin-top:1rem">💾 حفظ التغييرات</button></form></div><div class="nawa-card"><h2>📊 معلومات الحساب</h2><table class="nawa-table"><tr><th>ID</th><td><code>{id}</code></td></tr><tr><th>تاريخ الإنشاء</th><td>{created}</td></tr><tr><th>آخر دخول</th><td>{last_login}</td></tr></table></div></div></body></html>"#,
        name = user.username,
        username = user.username,
        email = user.email,
        id = user.id,
        created = user.created_at.split('T').next().unwrap_or("?"),
        last_login = user.last_login.as_ref().and_then(|s| s.split('T').next()).unwrap_or("—"),
        role_badge = role_badge,
        verified_badge = verified_badge)
}

/// Render system info page.
pub fn render_system(db: &DbEngine, auth: &AuthStore, uring: &NawaUring) -> String {
    let db_stats = db.stats();
    let uring_stats = uring.stats();
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>System — NAWA</title><style>{CSS}</style></head><body><nav class="nawa-nav"><a class="nawa-nav-brand" href="/">🦀 NAWA</a><div class="nawa-nav-links"><a href="/">Dashboard</a><a href="/system">System</a><a href="/metrics">Metrics</a></div></nav><div class="nawa-container nawa-fade-in"><div class="nawa-card"><h1>🖥 معلومات النظام</h1><table class="nawa-table"><tr><th>الإصدار</th><td><code>0.1.0-alpha</code></td></tr><tr><th>المنصة</th><td>{os} / {arch}</td></tr><tr><th>io_uring</th><td>{real_uring} (entries: {entries})</td></tr><tr><th>SQPOLL</th><td>{sqpoll}</td></tr></table></div><div class="nawa-card"><h2>📊 إحصائيات قاعدة البيانات</h2><table class="nawa-table"><tr><th>المفاتيح</th><td>{keys}</td></tr><tr><th>MemTable bytes</th><td>{memtable}</td></tr><tr><th>عمليات PUT</th><td>{puts}</td></tr><tr><th>عمليات GET</th><td>{gets}</td></tr><tr><th>عمليات DELETE</th><td>{deletes}</td></tr><tr><th>عمليات SCAN</th><td>{scans}</td></tr><tr><th>Flushes</th><td>{flushes}</td></tr></table></div><div class="nawa-card"><h2>⚡ إحصائيات io_uring</h2><table class="nawa-table"><tr><th>Submitted</th><td>{submitted}</td></tr><tr><th>Completed</th><td>{completed}</td></tr><tr><th>In-flight</th><td>{in_flight}</td></tr><tr><th>Errors</th><td>{uring_errors}</td></tr></table></div><div class="nawa-card"><h2>👥 المستخدمون</h2><table class="nawa-table"><tr><th>إجمالي المستخدمين</th><td>{users}</td></tr></table></div><div class="nawa-card"><h2>🔧 المكونات المدمجة</h2><div class="nawa-stats"><div class="nawa-stat"><div class="nawa-stat-val">✓</div><div class="nawa-stat-label">NAWA-DB</div></div><div class="nawa-stat"><div class="nawa-stat-val">✓</div><div class="nawa-stat-label">Auth+RBAC</div></div><div class="nawa-stat"><div class="nawa-stat-val">✓</div><div class="nawa-stat-label">Engine</div></div><div class="nawa-stat"><div class="nawa-stat-val">✓</div><div class="nawa-stat-label">io_uring</div></div><div class="nawa-stat"><div class="nawa-stat-val">✓</div><div class="nawa-stat-label">WASM</div></div><div class="nawa-stat"><div class="nawa-stat-val">✓</div><div class="nawa-stat-label">Metrics</div></div></div></div></div></body></html>"#,
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        real_uring = if uring.is_real_uring() { "✓ مفعّل" } else { "— fallback" },
        entries = uring.config().entries,
        sqpoll = if uring.is_sqpoll_enabled() { "✓" } else { "—" },
        keys = db.len(),
        memtable = db.memtable_size(),
        puts = db_stats.puts,
        gets = db_stats.gets,
        deletes = db_stats.deletes,
        scans = db_stats.scans,
        flushes = db_stats.memtable_flushes,
        submitted = uring_stats.submitted,
        completed = uring_stats.completed,
        in_flight = uring_stats.in_flight,
        uring_errors = uring_stats.errors,
        users = auth.user_count())
}
