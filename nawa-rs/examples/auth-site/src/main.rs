//! # Auth Site — NAWA Authentication Example
//!
//! Complete web app with:
//! - Registration (first user = admin, auto-verified)
//! - Login (auto-login after registration)
//! - Dashboard (admin sees all users)
//! - User management (admin controls roles, verification, deletion)
//! - Settings page (admin controls project settings)
//! - JWT-based session management
//! - Professional RTL Arabic UI

use nawa_auth::{AuthConfig, AuthStore};
use nawa_db::DbEngine;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  NAWA Auth Site v0.1.0                       ║");
    println!("║  نظام مصادقة متكامل احترافي                  ║");
    println!("╚══════════════════════════════════════════════╝\n");

    let db = Arc::new(DbEngine::open_in_memory());
    let auth = Arc::new(AuthStore::new(
        db.clone(),
        AuthConfig::with_secret("nawa-auth-site-secret-2026"),
    ));
    println!("✓ Auth system initialized");
    println!("✓ Users: {}", auth.user_count());

    let addr = "0.0.0.0:8080";
    let port = addr.split(':').last().unwrap_or("8080");
    println!("\n🚀 Auth site running on http://localhost:{port}");
    println!("\n   Register:  http://localhost:{port}/register");
    println!("   Login:     http://localhost:{port}/login");
    println!("   Dashboard: http://localhost:{port}/dashboard");
    println!("   Settings:  http://localhost:{port}/settings\n");
    println!("Press Ctrl+C to stop\n");

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let auth = auth.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, auth).await {
                eprintln!("error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    auth: Arc<AuthStore>,
) -> anyhow::Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = reader.read(&mut tmp).await?;
        if n == 0 { return Ok(()); }
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") { break; }
        if buf.len() > 256 * 1024 { return Ok(()); }
    }

    let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let header_str = String::from_utf8_lossy(&buf[..header_end]);
    let mut lines = header_str.split("\r\n");
    let req_line = lines.next().unwrap_or("");
    let mut parts = req_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path_full = parts.next().unwrap_or("/");
    let path = path_full.split('?').next().unwrap_or("/");

    // Parse cookies for JWT.
    let cookie_header = header_str.lines()
        .find(|l| l.to_lowercase().starts_with("cookie:"))
        .unwrap_or("");
    let token = extract_cookie(cookie_header, "nawa_token");

    // Parse body.
    let body = if method == "POST" || method == "PUT" {
        let cl: usize = header_str.lines()
            .find(|l| l.to_lowercase().starts_with("content-length:"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        if cl > 0 {
            let leftover = &buf[header_end + 4..];
            let mut body = leftover.to_vec();
            if body.len() < cl {
                let mut rest = vec![0u8; cl - body.len()];
                reader.read_exact(&mut rest).await?;
                body.extend_from_slice(&rest);
            }
            body.truncate(cl);
            String::from_utf8_lossy(&body).to_string()
        } else { String::new() }
    } else { String::new() };

    // Parse form data or JSON.
    let form = parse_form(&body);

    // Route.
    let (status, content_type, response_body, set_cookie) = route(method, path, &form, &token, &auth);

    // Build response.
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n",
        response_body.len()
    );
    if let Some(cookie) = set_cookie {
        response.push_str(&format!("Set-Cookie: nawa_token={cookie}; Path=/; HttpOnly; Max-Age=604800\r\n"));
    }
    response.push_str("X-Powered-By: NAWA-Auth/0.1.0\r\n\r\n");
    writer.write_all(response.as_bytes()).await?;
    writer.write_all(response_body.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

fn route(
    method: &str,
    path: &str,
    form: &std::collections::HashMap<String, String>,
    token: &Option<String>,
    auth: &AuthStore,
) -> (String, String, String, Option<String>) {
    // Get current user from token.
    let current_user = token.as_ref()
        .and_then(|t| auth.verify_token(t).ok())
        .and_then(|claims| auth.get_user(&claims.sub).ok());

    let empty = String::new();
    let redirect = |url: &str| -> (String, String, String, Option<String>) {
        ("302 Found".into(), "text/html".into(),
         format!(r#"<html><head><meta http-equiv="refresh" content="0;url={url}"></head></html>"#), None)
    };

    match (method, path) {
        // ─── Pages ─────────────────────────────────────────
        ("GET", "/") => {
            let html = if let Some(u) = &current_user {
                render_dashboard(u, auth)
            } else {
                render_landing()
            };
            ("200 OK".into(), "text/html; charset=utf-8".into(), html, None)
        }

        ("GET", "/register") => {
            if current_user.is_some() { return redirect("/dashboard"); }
            ("200 OK".into(), "text/html; charset=utf-8".into(), render_register(), None)
        }

        ("POST", "/register") => {
            let username = form.get("username").map(|s| s.as_str()).unwrap_or("");
            let email = form.get("email").map(|s| s.as_str()).unwrap_or("");
            let password = form.get("password").map(|s| s.as_str()).unwrap_or("");

            match auth.register(username, email, password) {
                Ok(result) => {
                    ("200 OK".into(), "text/html".into(),
                     r#"<html><head><meta http-equiv="refresh" content="0;url=/dashboard"></head><body>Redirecting...</body></html>"#.to_string(),
                     Some(result.token))
                }
                Err(e) => ("200 OK".into(), "text/html; charset=utf-8".into(),
                    render_error(&e.to_string(), "/register"), None),
            }
        }

        ("GET", "/login") => {
            if current_user.is_some() { return redirect("/dashboard"); }
            ("200 OK".into(), "text/html; charset=utf-8".into(), render_login(), None)
        }

        ("POST", "/login") => {
            let email = form.get("email").map(|s| s.as_str()).unwrap_or("");
            let password = form.get("password").map(|s| s.as_str()).unwrap_or("");

            match auth.login(email, password) {
                Ok(result) => ("200 OK".into(), "text/html".into(),
                    r#"<html><head><meta http-equiv="refresh" content="0;url=/dashboard"></head></html>"#.to_string(),
                    Some(result.token)),
                Err(e) => ("200 OK".into(), "text/html; charset=utf-8".into(),
                    render_error(&e.to_string(), "/login"), None),
            }
        }

        ("GET", "/logout") => {
            ("200 OK".into(), "text/html".into(),
             r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head><body>Logging out...</body></html>"#.to_string(),
             Some(empty)) // clear cookie
        }

        ("GET", "/dashboard") => {
            match &current_user {
                Some(user) => ("200 OK".into(), "text/html; charset=utf-8".into(),
                    render_dashboard(user, auth), None),
                None => redirect("/login"),
            }
        }

        ("GET", "/settings") => {
            match &current_user {
                Some(user) if user.role == "admin" => {
                    ("200 OK".into(), "text/html; charset=utf-8".into(),
                        render_settings(user, auth), None)
                }
                Some(_) => ("200 OK".into(), "text/html; charset=utf-8".into(),
                    render_error("صلاحية الأدمن مطلوبة", "/dashboard"), None),
                None => redirect("/login"),
            }
        }

        ("POST", "/settings") => {
            match &current_user {
                Some(user) if user.role == "admin" => {
                    let mut settings = auth.get_settings().unwrap_or_default();
                    settings.project_name = form.get("project_name").cloned().unwrap_or_default();
                    settings.registration_open = form.contains_key("registration_open");
                    settings.verification_required = form.contains_key("verification_required");
                    settings.max_users = form.get("max_users").and_then(|s| s.parse().ok());
                    if let Some(expiry) = form.get("jwt_expiry_secs").and_then(|s| s.parse::<u64>().ok()) {
                        settings.jwt_expiry_secs = expiry;
                    }
                    let _ = auth.update_settings(&user.id, &settings);
                    ("200 OK".into(), "text/html; charset=utf-8".into(),
                        render_settings(user, auth), None)
                }
                _ => ("403 Forbidden".into(), "text/html".into(),
                    render_error("صلاحية الأدمن مطلوبة", "/dashboard"), None),
            }
        }

        // ─── Admin Actions ─────────────────────────────────
        ("POST", "/admin/verify") => {
            let target_id = form.get("user_id").map(|s| s.as_str()).unwrap_or("");
            if let Some(admin) = &current_user {
                if admin.role == "admin" {
                    let _ = auth.verify_user(&admin.id, target_id);
                }
            }
            redirect("/dashboard")
        }

        ("POST", "/admin/role") => {
            let target_id = form.get("user_id").map(|s| s.as_str()).unwrap_or("");
            let new_role = form.get("role").map(|s| s.as_str()).unwrap_or("user");
            if let Some(admin) = &current_user {
                if admin.role == "admin" {
                    let _ = auth.change_role(&admin.id, target_id, new_role);
                }
            }
            redirect("/dashboard")
        }

        ("POST", "/admin/delete") => {
            let target_id = form.get("user_id").map(|s| s.as_str()).unwrap_or("");
            if let Some(admin) = &current_user {
                if admin.role == "admin" {
                    let _ = auth.delete_user(&admin.id, target_id);
                }
            }
            redirect("/dashboard")
        }

        // ─── API ───────────────────────────────────────────
        ("GET", "/api/me") => {
            match &current_user {
                Some(user) => {
                    let json = serde_json::json!({
                        "id": user.id, "username": user.username,
                        "email": user.email, "role": user.role, "verified": user.verified
                    });
                    ("200 OK".into(), "application/json".into(),
                        serde_json::to_string(&json).unwrap(), None)
                }
                None => ("401 Unauthorized".into(), "application/json".into(),
                    r#"{"error":"not authenticated"}"#.into(), None),
            }
        }

        ("GET", "/api/users") => {
            match &current_user {
                Some(user) if user.role == "admin" => {
                    let users = auth.list_users().unwrap_or_default();
                    // SECURITY: strip password_hash from response.
                    let safe_users: Vec<_> = users.iter().map(|u| {
                        serde_json::json!({
                            "id": u.id,
                            "username": u.username,
                            "email": u.email,
                            "role": u.role,
                            "verified": u.verified,
                            "created_at": u.created_at,
                            "last_login": u.last_login,
                        })
                    }).collect();
                    let json = serde_json::json!({ "users": safe_users, "count": safe_users.len() });
                    ("200 OK".into(), "application/json".into(),
                        serde_json::to_string(&json).unwrap(), None)
                }
                _ => ("403 Forbidden".into(), "application/json".into(),
                    r#"{"error":"admin required"}"#.into(), None),
            }
        }

        ("GET", "/api/settings") => {
            let settings = auth.get_settings().unwrap_or_default();
            ("200 OK".into(), "application/json".into(),
                serde_json::to_string(&settings).unwrap(), None)
        }

        _ => ("404 Not Found".into(), "text/html; charset=utf-8".into(),
            render_404(), None),
    }
}

// ─── HTML Rendering ──────────────────────────────────────────

fn render_landing() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>NAWA Auth — مرحباً</title><style>{STYLES}</style></head><body><div class="container"><div class="hero"><h1>🦀 NAWA Auth</h1><p>نظام مصادقة متكامل احترافي بُني بـ Rust</p><p class="subtitle">أول مستخدم = أدمن · تسجيل دخول تلقائي · تحكم كامل في الصلاحيات</p><div class="actions"><a href="/register" class="btn btn-primary">إنشاء حساب</a><a href="/login" class="btn btn-secondary">تسجيل الدخول</a></div></div></div></body></html>"#)
}

fn render_register() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>إنشاء حساب — NAWA Auth</title><style>{STYLES}</style></head><body><div class="container"><div class="auth-card"><h1>إنشاء حساب جديد</h1><p class="subtitle">أول مستخدم يصبح أدمن تلقائياً</p><form method="POST" action="/register"><input type="text" name="username" placeholder="اسم المستخدم" required><input type="email" name="email" placeholder="البريد الإلكتروني" required><input type="password" name="password" placeholder="كلمة المرور" required><button type="submit" class="btn btn-primary">إنشاء حساب · تسجيل دخول تلقائي</button></form><p class="link"><a href="/login">لديك حساب؟ سجل دخول</a></p></div></div></body></html>"#)
}

fn render_login() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>تسجيل الدخول — NAWA Auth</title><style>{STYLES}</style></head><body><div class="container"><div class="auth-card"><h1>تسجيل الدخول</h1><form method="POST" action="/login"><input type="email" name="email" placeholder="البريد الإلكتروني" required><input type="password" name="password" placeholder="كلمة المرور" required><button type="submit" class="btn btn-primary">دخول</button></form><p class="link"><a href="/register">ليس لديك حساب؟ أنشئ واحداً</a></p></div></div></body></html>"#)
}

fn render_dashboard(user: &nawa_auth::User, auth: &AuthStore) -> String {
    let is_admin = user.role == "admin";
    let all_users = if is_admin { auth.list_users().unwrap_or_default() } else { vec![] };

    let users_table = if is_admin {
        let rows: String = all_users.iter().map(|u| {
            let verified_badge = if u.verified {
                r#"<span class="badge badge-ok">موثّق</span>"#
            } else {
                r#"<span class="badge badge-warn">غير موثّق</span>"#
            };
            let role_badge = if u.role == "admin" {
                r#"<span class="badge badge-admin">أدمن</span>"#
            } else {
                r#"<span class="badge badge-user">مستخدم</span>"#
            };
            let actions: String = if u.id != user.id {
                format!(r#"<form method="POST" action="/admin/verify" style="display:inline"><input type="hidden" name="user_id" value="{id}"><button type="submit" class="btn-sm btn-verify">توثيق</button></form>
                <form method="POST" action="/admin/role" style="display:inline"><input type="hidden" name="user_id" value="{id}"><select name="role" onchange="this.form.submit()"><option value="user" {sel_user}>مستخدم</option><option value="admin" {sel_admin}>أدمن</option></select></form>
                <form method="POST" action="/admin/delete" style="display:inline" onsubmit="return confirm('حذف {username}؟')"><input type="hidden" name="user_id" value="{id}"><button type="submit" class="btn-sm btn-delete">حذف</button></form>"#,
                    id = u.id, username = u.username,
                    sel_user = if u.role == "user" { "selected" } else { "" },
                    sel_admin = if u.role == "admin" { "selected" } else { "" })
            } else {
                r#"<span class="you">(أنت)</span>"#.to_string()
            };
            format!(r#"<tr><td>{username}</td><td>{email}</td><td>{role_badge}</td><td>{verified_badge}</td><td>{actions}</td></tr>"#,
                username = u.username, email = u.email)
        }).collect();
        format!(r#"<div class="card"><h2>إدارة المستخدمين</h2><table><thead><tr><th>الاسم</th><th>البريد</th><th>الدور</th><th>الحالة</th><th>إجراءات</th></tr></thead><tbody>{rows}</tbody></table></div>"#, rows = rows)
    } else { String::new() };

    let admin_link = if is_admin {
        r#"<a href="/settings" class="btn btn-secondary">⚙ الإعدادات</a>"#
    } else { "" };

    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>لوحة التحكم — NAWA Auth</title><style>{STYLES}</style></head><body><div class="container"><div class="nav"><a href="/dashboard" class="nav-brand">NAWA Dashboard</a><div class="nav-right"><span class="user-info">{username} ({role})</span>{admin_link}<a href="/logout" class="btn btn-sm btn-danger">خروج</a></div></div><div class="card"><h2>مرحباً، {username}!</h2><div class="stats"><div class="stat"><div class="stat-val">{role}</div><div class="stat-label">الدور</div></div><div class="stat"><div class="stat-val">{verified}</div><div class="stat-label">الحالة</div></div><div class="stat"><div class="stat-val">{total}</div><div class="stat-label">إجمالي المستخدمين</div></div></div></div>{users_table}</div></body></html>"#,
        username = user.username, role = user.role,
        verified = if user.verified { "موثّق" } else { "غير موثّق" },
        total = auth.user_count())
}

fn render_settings(user: &nawa_auth::User, auth: &AuthStore) -> String {
    let settings = auth.get_settings().unwrap_or_default();
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>الإعدادات — NAWA Auth</title><style>{STYLES}</style></head><body><div class="container"><div class="nav"><a href="/dashboard" class="nav-brand">← العودة</a><span>الإعدادات</span></div><div class="card"><h2>⚙ إعدادات المشروع</h2><form method="POST" action="/settings"><label>اسم المشروع</label><input type="text" name="project_name" value="{project_name}"><label><input type="checkbox" name="registration_open" {reg_open}> التسجيل مفتوح</label><label><input type="checkbox" name="verification_required" {ver_req}> التوثيق إلزامي</label><label>الحد الأقصى للمستخدمين (فارغ = بلا حد)</label><input type="number" name="max_users" value="{max_users}"><label>مدة صلاحية JWT (بالثواني)</label><input type="number" name="jwt_expiry_secs" value="{jwt_expiry}"><button type="submit" class="btn btn-primary">حفظ الإعدادات</button></form></div></div></body></html>"#,
        project_name = settings.project_name,
        reg_open = if settings.registration_open { "checked" } else { "" },
        ver_req = if settings.verification_required { "checked" } else { "" },
        max_users = settings.max_users.map(|m| m.to_string()).unwrap_or_default(),
        jwt_expiry = settings.jwt_expiry_secs)
}

fn render_error(msg: &str, back_url: &str) -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><title>خطأ</title><style>{STYLES}</style></head><body><div class="container"><div class="auth-card"><h1 class="error">⚠ {msg}</h1><a href="{back_url}" class="btn btn-secondary">العودة</a></div></div></body></html>"#)
}

fn render_404() -> String {
    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><title>404</title><style>{STYLES}</style></head><body><div class="container"><div class="hero"><h1>404</h1><p>الصفحة غير موجودة</p><a href="/" class="btn btn-primary">العودة للرئيسية</a></div></div></body></html>"#)
}

const STYLES: &str = r#"
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:'Noto Sans Arabic',system-ui,sans-serif;background:#0d0c0a;color:#e0e0e0;line-height:1.8;min-height:100vh}
.container{max-width:900px;margin:0 auto;padding:2rem}
.hero{text-align:center;padding:4rem 0}
.hero h1{color:#f59e0b;font-size:3rem;margin-bottom:1rem}
.subtitle{color:#888;margin:0.5rem 0}
.auth-card{max-width:450px;margin:3rem auto;padding:2rem;background:#1a1a1a;border-radius:16px;border:1px solid #2a2a2a}
.auth-card h1{color:#f59e0b;margin-bottom:1rem;text-align:center}
input[type="text"],input[type="email"],input[type="password"],input[type="number"],select{width:100%;padding:0.8rem;margin:0.5rem 0;background:#0d0c0a;border:1px solid #3a3a3a;border-radius:8px;color:#e0e0e0;font-size:1rem}
input:focus,select:focus{border-color:#f59e0b;outline:none}
label{display:block;margin:0.8rem 0 0.3rem;color:#aaa;font-size:0.9rem}
label input[type="checkbox"]{display:inline-block;width:auto;margin-left:0.5rem}
.btn{display:inline-block;padding:0.8rem 1.5rem;border:none;border-radius:8px;cursor:pointer;text-decoration:none;font-size:1rem;transition:all 0.2s;margin:0.5rem 0.5rem 0.5rem 0}
.btn-primary{background:#f59e0b;color:#0d0c0a;font-weight:bold}
.btn-primary:hover{background:#d97706}
.btn-secondary{background:#2a2a2a;color:#e0e0e0;border:1px solid #3a3a3a}
.btn-secondary:hover{background:#3a3a3a}
.btn-sm{padding:0.3rem 0.8rem;font-size:0.85rem}
.btn-danger{background:#dc2626;color:#fff}
.btn-danger:hover{background:#b91c1c}
.btn-verify{background:#10b981;color:#0d0c0a}
.btn-delete{background:#dc2626;color:#fff}
.actions{margin-top:2rem}
.link{margin-top:1.5rem;text-align:center}
.link a{color:#f59e0b;text-decoration:none}
.error{color:#dc2626}
.nav{display:flex;justify-content:space-between;align-items:center;padding:1rem 0;border-bottom:1px solid #2a2a2a;margin-bottom:2rem}
.nav-brand{color:#f59e0b;text-decoration:none;font-weight:bold}
.nav-right{display:flex;align-items:center;gap:1rem}
.user-info{color:#888;font-size:0.9rem}
.card{padding:1.5rem;background:#1a1a1a;border-radius:12px;border:1px solid #2a2a2a;margin:1rem 0}
.card h2{color:#f59e0b;margin-bottom:1rem}
.stats{display:flex;gap:2rem;margin-top:1rem}
.stat{text-align:center}
.stat-val{color:#f59e0b;font-size:1.5rem;font-weight:bold}
.stat-label{color:#888;font-size:0.8rem}
table{width:100%;border-collapse:collapse;margin-top:1rem}
th,td{padding:0.6rem;text-align:right;border-bottom:1px solid #2a2a2a}
th{color:#f59e0b;font-size:0.85rem}
.badge{display:inline-block;padding:0.2rem 0.6rem;border-radius:4px;font-size:0.75rem}
.badge-ok{background:#10b98115;color:#10b981}
.badge-warn{background:#f59e0b15;color:#f59e0b}
.badge-admin{background:#dc262615;color:#dc2626}
.badge-user{background:#3b82f615;color:#3b82f6}
.you{color:#555;font-size:0.85rem;font-style:italic}
"#;

// ─── Helpers ─────────────────────────────────────────────────

fn extract_cookie(cookie_header: &str, name: &str) -> Option<String> {
    // Remove "Cookie:" prefix if present.
    let header_value = if cookie_header.to_lowercase().starts_with("cookie:") {
        cookie_header[7..].trim()
    } else {
        cookie_header.trim()
    };
    header_value
        .split(';')
        .find_map(|c| {
            let c = c.trim();
            let prefix = format!("{name}=");
            c.strip_prefix(&prefix).map(|v| v.to_string())
        })
        .filter(|v| !v.is_empty())
}

fn parse_form(body: &str) -> std::collections::HashMap<String, String> {
    let mut form = std::collections::HashMap::new();
    for pair in body.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let key = url_decode(k);
            let val = url_decode(v);
            form.insert(key, val);
        }
    }
    form
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '+' { out.push(' '); }
        else if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                out.push(byte as char);
            }
        } else { out.push(c); }
    }
    out
}
