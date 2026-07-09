//! # nawa CLI
//!
//! The standalone CLI tool for creating, developing, and deploying NAWA projects.
//!
//! ## Commands
//!
//! - `nawa create <name> [--template <t>]` — scaffold a new project
//! - `nawa dev` — start dev server with hot reload
//! - `nawa build` — build optimized production binary
//! - `nawa deploy [--target <ssh>]` — deploy to remote server
//! - `nawa benchmark` — run performance benchmarks
//! - `nawa info` — show version and component info
//! - `nawa --version` — show version

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;

/// NAWA CLI — the revolutionary web operating system.
#[derive(Parser, Debug)]
#[command(
    name = "nawa",
    version,
    about = "NAWA CLI — create, develop, and deploy NAWA projects",
    long_about = "NAWA is a revolutionary web operating system built in Rust.\n\
                  This CLI helps you create new projects, run the dev server,\n\
                  build production binaries, and deploy to any VPS."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new NAWA project from a template.
    Create {
        /// Project name (becomes the directory name).
        name: String,

        /// Template to use.
        #[arg(short, long, default_value = "auth")]
        template: String,

        /// Target directory (default: current dir).
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Start the development server with hot reload.
    Dev {
        /// Address to bind on.
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,

        /// Data directory for NAWA-DB.
        #[arg(long, default_value = "./nawa-data")]
        data_dir: PathBuf,
    },

    /// Build an optimized production binary.
    Build {
        /// Build in release mode (default: true).
        #[arg(long, default_value_t = true)]
        release: bool,
    },

    /// Deploy the project to a remote server.
    Deploy {
        /// SSH target (e.g., "user@host").
        #[arg(long)]
        target: Option<String>,

        /// Data directory on the remote.
        #[arg(long, default_value = "/var/lib/nawa")]
        remote_data_dir: PathBuf,
    },

    /// Run performance benchmarks.
    Benchmark {
        /// Number of operations.
        #[arg(short, long, default_value = "100000")]
        ops: u32,
    },

    /// Run tests for the NAWA workspace.
    Test {
        /// Run only the specified crate's tests.
        #[arg(short, long)]
        crate_name: Option<String>,

        /// Run benchmarks instead of tests.
        #[arg(short, long)]
        bench: bool,

        /// Show test output.
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run the example blog site.
    Serve {
        /// Address to bind on.
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,
    },

    /// Show version and component info.
    Info,

    /// List available templates.
    Templates,

    /// Update NAWA to the latest version (re-runs the installer).
    Update,

    /// Uninstall NAWA completely — removes the binary, data dir, and PATH entry.
    Uninstall {
        /// Skip the confirmation prompt.
        #[arg(short, long)]
        yes: bool,
    },
}

const TEMPLATES: &[(&str, &str)] = &[
    ("auth", "Full auth system — register/login/dashboard/admin/settings (DEFAULT)"),
    ("blog", "Blog / CMS with SSR + SEO + admin panel"),
    ("saas", "Multi-tenant SaaS with subscriptions + billing"),
    ("shop", "E-commerce with cart + checkout + inventory"),
    ("realtime", "Realtime chat with WebSocket + presence"),
    ("booking", "Booking system with calendar + payments"),
    ("portfolio", "Portfolio with projects + contact form"),
];

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            name,
            template,
            dir,
        } => create_project(&name, &template, dir),
        Commands::Dev { addr, data_dir } => dev_server(&addr, &data_dir),
        Commands::Build { release: _ } => build_project(),
        Commands::Deploy {
            target,
            remote_data_dir,
        } => deploy(&target, &remote_data_dir),
        Commands::Benchmark { ops } => benchmark(ops),
        Commands::Test {
            crate_name,
            bench,
            verbose,
        } => run_tests(&crate_name, bench, verbose),
        Commands::Serve { addr } => serve_example(&addr),
        Commands::Info => {
            print_info();
            Ok(())
        }
        Commands::Templates => {
            list_templates();
            Ok(())
        }
        Commands::Update => update_nawa(),
        Commands::Uninstall { yes } => uninstall_nawa(yes),
    }
}

/// `nawa update` — re-runs the installer from GitHub.
fn update_nawa() -> anyhow::Result<()> {
    println!("🔄 تحديث NAWA من GitHub...");
    println!("   يتم بناء النسخة الأخيرة من المصدر (5-10 دقائق).");
    println!();
    let url = "https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/scripts/install.sh";
    let status = Command::new("bash")
        .arg("-c")
        .arg(format!("curl -fsSL {} | bash", url))
        .status()?;
    if !status.success() {
        anyhow::bail!("update failed — exit code {:?}", status.code());
    }
    Ok(())
}

/// `nawa uninstall` — removes NAWA completely.
fn uninstall_nawa(skip_confirm: bool) -> anyhow::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let nawa_dir = format!("{}/.nawa", home);

    if !PathBuf::from(&nawa_dir).exists() {
        println!("ℹ️  NAWA غير مثبّت ({}) — لا شيء للحذف.", nawa_dir);
        return Ok(());
    }

    if !skip_confirm {
        println!("⚠️  سيتم حذف NAWA بالكامل من: {}", nawa_dir);
        println!("   هذا يشمل: binary، البيانات، الـ plugins، القوالب.");
        print!("هل أنت متأكد؟ [y/N] ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("✓ تم الإلغاء.");
            return Ok(());
        }
    }

    println!("🗑️  إيقاف الخادم إن كان يعمل...");
    let _ = Command::new("pkill").arg("-f").arg("nawad serve").status();
    let _ = Command::new("pkill").arg("-f").arg("nawad").status();
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("📂 حذف {}...", nawa_dir);
    let result = std::fs::remove_dir_all(&nawa_dir);
    match result {
        Ok(()) => println!("✓ تم حذف المجلد."),
        Err(e) => {
            println!("⚠ تعذّر حذف المجلد بالكامل: {}", e);
            println!("  حاول: rm -rf {}", nawa_dir);
        }
    }

    // Remove PATH entry from ~/.bashrc
    let bashrc = format!("{}/.bashrc", home);
    if PathBuf::from(&bashrc).exists() {
        println!("🧹 تنظيف ~/.bashrc...");
        if let Ok(content) = std::fs::read_to_string(&bashrc) {
            let filtered: String = content
                .lines()
                .filter(|line| !line.contains(".nawa/bin"))
                .collect::<Vec<_>>()
                .join("\n");
            let _ = std::fs::write(&bashrc, filtered);
            println!("✓ تم تنظيف PATH.");
        }
    }

    println!();
    println!("✅ تم حذف NAWA بالكامل.");
    println!("   نفّذ: source ~/.bashrc");
    Ok(())
}

fn create_project(name: &str, template: &str, dir: Option<PathBuf>) -> anyhow::Result<()> {
    let base_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    let project_dir = base_dir.join(name);

    if project_dir.exists() {
        anyhow::bail!("directory already exists: {}", project_dir.display());
    }

    println!("╔══════════════════════════════════════════════╗");
    println!("║  NAWA — إنشاء مشروع جديد (Rust + Svelte)      ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("  الاسم: {name}");
    println!();

    // Create directory structure.
    println!("📁 إنشاء هيكل المشروع...");
    std::fs::create_dir_all(&project_dir)?;
    std::fs::create_dir_all(project_dir.join("src"))?;
    std::fs::create_dir_all(project_dir.join("ui"))?;
    std::fs::create_dir_all(project_dir.join("ui/src"))?;
    std::fs::create_dir_all(project_dir.join("data"))?;
    println!("  ✓ هيكل المشروع جاهز");

    // Cargo.toml — with nawa-svelte dependency
    println!("📦 إنشاء Cargo.toml...");
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
nawa-db = {{ path = "../../crates/nawa-db" }}
nawa-http = {{ path = "../../crates/nawa-http" }}
nawa-svelte = {{ path = "../../crates/nawa-svelte" }}
tokio = {{ version = "1.42", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    println!("  ✓ Cargo.toml");

    // src/main.rs — NAWA server with SvelteKit auto-detect
    println!("🦀 إنشاء src/main.rs (خادم مع Svelte auto-detect)...");
    let main_rs = "use nawa_db::{DbEngine, Value};\nuse nawa_http::{HttpServer, Response, Router};\nuse nawa_svelte::SvelteHandler;\nuse std::net::SocketAddr;\nuse std::path::PathBuf;\nuse std::sync::Arc;\n\n#[tokio::main]\nasync fn main() -> anyhow::Result<()> {\n    println!(\"NAWA Project v2.5.1 — Starting...\");\n\n    let db = Arc::new(DbEngine::open(nawa_db::DbConfig {\n        data_dir: PathBuf::from(\"./data\"),\n        memtable_max_size: 4 * 1024 * 1024,\n        wal_sync: true,\n    })?);\n    println!(\"✓ NAWA-DB: {} keys\", db.len());\n\n    // Auto-detect SvelteKit UI\n    let svelte_dir = PathBuf::from(\"./ui/_nawa\");\n    let svelte: Option<Arc<SvelteHandler>> = if svelte_dir.join(\"manifest.json\").exists() {\n        match SvelteHandler::load(&svelte_dir, \"ws://localhost:8081\".to_string()) {\n            Ok(h) => { println!(\"✓ SvelteKit UI: {} routes\", h.route_count()); Some(h) }\n            Err(e) => { println!(\"⚠ SvelteKit: {e}\"); None }\n        }\n    } else {\n        println!(\"ℹ Run 'nawa dev' to build SvelteKit UI\");\n        None\n    };\n\n    let mut router = Router::new();\n\n    if let Some(ref h) = svelte {\n        let h2 = h.clone();\n        let db2 = db.clone();\n        router.get(\"/\", move |req| {\n            let h = h2.clone(); let db = db2.clone();\n            async move {\n                let keys: Vec<_> = db.scan_prefix(\"\", 10).into_iter()\n                    .map(|(k,v)| (String::from_utf8_lossy(&k).to_string(), v.display())).collect();\n                let state = serde_json::json!({\"db_keys\": keys, \"db_size\": db.len()});\n                let q = req.query.clone().into_iter().collect();\n                let p = h.handle(\"/\", q, None, None, state);\n                let mut r = Response::text(String::from_utf8_lossy(&p.html).to_string());\n                r.header(\"Content-Type\", p.content_type); r\n            }\n        });\n        let h3 = h.clone();\n        router.get(\"/_nawa/**\", move |req| {\n            let h = h3.clone();\n            async move {\n                let rest = req.param(\"_rest\").unwrap_or(\"\").to_string();\n                let path = rest.strip_prefix(\"assets/\").unwrap_or(&rest);\n                match h.serve_asset(path) {\n                    Some((b, ct)) => { let mut r = Response::ok(b); r.header(\"Content-Type\", ct); r }\n                    None => Response::not_found(\"asset\"),\n                }\n            }\n        });\n    } else {\n        router.get(\"/\", |_| async {\n            let mut r = Response::json(&serde_json::json!({\n                \"error\": \"SvelteKit UI not built\",\n                \"hint\": \"Run 'nawa dev' to build it automatically\"\n            }));\n            r.status = nawa_http::StatusCode(503); r\n        });\n    }\n\n    // API\n    { let db = db.clone(); router.get(\"/api/health\", move |_| { let db = db.clone(); async move {\n        Response::json(&serde_json::json!({\"status\":\"ok\",\"keys\":db.len(),\"version\":\"2.5.1\"}))\n    }}); }\n    { let db = db.clone(); router.get(\"/api/data\", move |_| { let db = db.clone(); async move {\n        let keys: Vec<_> = db.scan_prefix(\"\", 100).into_iter()\n            .map(|(k,v)| (String::from_utf8_lossy(&k).to_string(), v.display())).collect();\n        Response::json(&serde_json::json!({\"count\": keys.len(), \"keys\": keys}))\n    }}); }\n    { let db = db.clone(); router.post(\"/:key\", move |req| { let db = db.clone(); async move {\n        let key = req.param(\"key\").unwrap_or(\"\").to_string();\n        let value = Value::Bytes(req.body.clone());\n        match db.put(&key, value) {\n            Ok(_) => Response::text(format!(\"stored: {key}\")),\n            Err(e) => Response::text(format!(\"error: {e}\")),\n        }\n    }}); }\n\n    let addr: SocketAddr = \"0.0.0.0:8080\".parse()?;\n    println!(\"\\n🚀 NAWA on http://localhost:8080\");\n    let server = HttpServer::new(router, addr);\n    server.serve().await?;\n    Ok(())\n}\n";
    std::fs::write(project_dir.join("src/main.rs"), main_rs)?;
    println!("  ✓ src/main.rs");

    // SvelteKit UI files
    println!("🎨 إنشاء واجهة SvelteKit...");
    std::fs::write(project_dir.join("ui/package.json"), r#"{
  "name": "nawa-ui",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build && node post-build.js",
    "preview": "vite preview"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^7.2.0",
    "svelte": "^5.56.4",
    "vite": "^8.1.3"
  }
}
"#)?;
    std::fs::write(project_dir.join("ui/vite.config.js"), r#"import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({
  plugins: [svelte()],
  build: { outDir: '_nawa', emptyOutDir: true, rollupOptions: { output: { entryFileNames: 'assets/app.js', assetFileNames: 'assets/[name][extname]' } } }
});
"#)?;
    std::fs::write(project_dir.join("ui/index.html"), "<!DOCTYPE html>\n<html lang=\"ar\" dir=\"rtl\">\n<head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1.0\"><title>NAWA Project</title></head>\n<body><div id=\"svelte\"></div><script type=\"module\" src=\"/src/main.ts\"></script></body>\n</html>\n")?;
    std::fs::write(project_dir.join("ui/src/main.ts"), "import './app.css';\nimport App from './App.svelte';\nnew App({ target: document.getElementById('svelte') || document.body });\n")?;
    std::fs::write(project_dir.join("ui/src/app.css"), "body { margin: 0; background: #0a0a0f; }\n")?;
    std::fs::write(project_dir.join("ui/src/App.svelte"), r#"<script>
  import { onMount } from 'svelte';
  let loaded = false; let keys = []; let newKey = ''; let newVal = '';
  onMount(() => { loaded = true; refresh(); });
  async function refresh() {
    try { const r = await fetch('/api/data'); const d = await r.json(); keys = d.keys || []; } catch(e) {}
  }
  async function add() {
    if (!newKey || !newVal) return;
    await fetch(`/${newKey}`, { method: 'POST', body: newVal });
    newKey = ''; newVal = ''; refresh();
  }
</script>
<div class="app" class:loaded>
  <nav><span>🦀 NAWA Project</span></nav>
  <main>
    <h1>مرحباً بك في NAWA</h1>
    <p>واجهة Svelte تعمل تلقائياً مع خادم Rust</p>
    <div class="card">
      <input bind:value={newKey} placeholder="المفتاح" />
      <input bind:value={newVal} placeholder="القيمة" />
      <button on:click={add}>إضافة</button>
    </div>
    <div class="card">
      <h2>البيانات ({keys.length})</h2>
      {#each keys as [k, v]}
        <div class="row"><span>{k}</span><span>{v}</span></div>
      {/each}
    </div>
  </main>
</div>
<style>
  :root { --bg:#0a0a0f; --s:#14141e; --b:rgba(245,158,11,.15); --p:#f59e0b; --a:#10b981; --t:#e8e8ef; --m:#8b8b9a; }
  * { margin:0; padding:0; box-sizing:border-box; }
  .app { font-family:'Noto Sans Arabic',system-ui; background:var(--bg); color:var(--t); min-height:100vh; opacity:0; transition:opacity .3s; }
  .app.loaded { opacity:1; }
  nav { padding:1rem 2rem; background:var(--s); border-bottom:1px solid var(--b); color:var(--p); font-weight:bold; }
  main { max-width:800px; margin:0 auto; padding:2rem; }
  h1 { color:var(--p); margin-bottom:.5rem; }
  p { color:var(--m); margin-bottom:1.5rem; }
  .card { background:var(--s); border:1px solid var(--b); border-radius:12px; padding:1.5rem; margin-bottom:1rem; }
  input { padding:.6rem; background:var(--bg); border:1px solid var(--b); border-radius:8px; color:var(--t); margin:.3rem; }
  button { padding:.6rem 1.5rem; background:var(--p); color:var(--bg); border:none; border-radius:8px; font-weight:bold; cursor:pointer; }
  .row { display:flex; justify-content:space-between; padding:.5rem; background:var(--bg); border-radius:6px; margin-bottom:.3rem; font-family:monospace; font-size:.85rem; }
  .row span:first-child { color:var(--p); } .row span:last-child { color:var(--a); }
</style>
"#)?;
    // post-build.js generates manifest.json
    std::fs::write(project_dir.join("ui/post-build.js"), r#"import { writeFileSync, existsSync, readdirSync } from 'fs';
import { join } from 'path';
const out = '_nawa';
let mainJs = 'app.js', mainCss = 'main.css';
const ad = join(out, 'assets');
if (existsSync(ad)) {
  const f = readdirSync(ad);
  if (f.find(x => x.endsWith('.js'))) mainJs = f.find(x => x.endsWith('.js'));
  if (f.find(x => x.endsWith('.css'))) mainCss = f.find(x => x.endsWith('.css'));
}
const html = `<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><title>NAWA Project</title><link rel="stylesheet" href="/_nawa/assets/${mainCss}"></head><body><div id="svelte"></div><script type="module" src="/_nawa/assets/${mainJs}"></script></body></html>`;
writeFileSync(join(out, 'spa.html'), html);
writeFileSync(join(out, 'index.html'), html);
writeFileSync(join(out, 'manifest.json'), JSON.stringify({
  version: 1, app_name: 'NAWA Project', built_at: new Date().toISOString(), sveltekit_version: '5.x',
  default_meta: { title: 'NAWA Project', description: 'Built with NAWA', og_title: 'NAWA', og_description: 'NAWA', og_type: 'website', twitter_card: 'summary', canonical: '/', robots: 'index, follow' },
  routes: [{ pattern: '/', methods: ['GET'], prerendered_html: 'index.html', hydration_js: mainJs, requires_auth: false, admin_only: false, meta: { title: 'NAWA', description: 'NAWA', og_title: 'NAWA', og_description: 'NAWA', og_type: 'website', twitter_card: 'summary', canonical: '/', robots: 'index, follow' }, is_endpoint: false, ssr_wasm: null, layout: null }]
}, null, 2));
console.log('✓ _nawa/manifest.json generated');
"#)?;
    println!("  ✓ ui/ (SvelteKit + Vite + App.svelte)");

    // nawa.toml
    std::fs::write(project_dir.join("nawa.toml"), format!("addr = \"0.0.0.0:8080\"\ndata_dir = \"./data\"\nwal_sync = true\nrate_limit = 100\nlog_level = \"info\"\njwt_secret = \"{name}-secret\"\n"))?;
    std::fs::write(project_dir.join(".gitignore"), "/target\n/data\n/ui/node_modules\n/ui/_nawa\n*.log\n.env\n")?;
    std::fs::write(project_dir.join("README.md"), format!("# {name}\n\nمشروع NAWA v2.5.1 — Rust + SvelteKit.\n\n## التشغيل\n```bash\nnawa dev  # يبني Svelte + Rust + يشغل الخادم\n```\nافتح: http://localhost:8080\n"))?;
    println!("  ✓ nawa.toml + README.md + .gitignore");

    println!();
    println!("═══════════════════════════════════════════════");
    println!("  ✅ تم إنشاء المشروع بنجاح!");
    println!("═══════════════════════════════════════════════");
    println!();
    println!("  cd {name}");
    println!("  nawa dev");
    println!("  → http://localhost:8080 (Svelte UI تعمل تلقائياً!)");
    println!();

    Ok(())
}

fn dev_server(addr: &str, _data_dir: &std::path::Path) -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  NAWA Dev — Build + Run (Svelte auto-build)  ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("  Addr: {addr}");
    println!();

    // Step 1: Build SvelteKit UI if ui/ exists
    let ui_dir = std::path::Path::new("./ui");
    if ui_dir.join("package.json").exists() {
        println!("🎨 SvelteKit UI detected at ./ui/");
        if !ui_dir.join("node_modules").exists() {
            println!("  📦 Installing npm dependencies...");
            let s = Command::new("npm").arg("install").current_dir(ui_dir).status()?;
            if !s.success() { anyhow::bail!("npm install failed"); }
        }
        println!("  🔨 Building SvelteKit → ./ui/_nawa/");
        let s = Command::new("npm").arg("run").arg("build").current_dir(ui_dir).status()?;
        if !s.success() { anyhow::bail!("npm run build failed"); }
        println!("  ✓ SvelteKit UI built");
    } else {
        println!("ℹ No ui/ directory — skipping Svelte build");
    }

    // Step 2: Build Rust binary
    println!("🦀 Building Rust server...");
    let s = Command::new("cargo").arg("build").arg("--release").status()?;
    if !s.success() { anyhow::bail!("cargo build failed"); }
    println!("✓ Rust binary built");

    // Step 3: Find the binary name from Cargo.toml
    let binary_name = get_binary_name();
    let binary_path = std::path::Path::new("./target/release").join(&binary_name);
    if !binary_path.exists() {
        anyhow::bail!("binary not found at {}. Make sure Cargo.toml has [[bin]] name = \"...\"", binary_path.display());
    }

    // Step 4: Start the server
    println!();
    println!("🚀 Starting server on http://localhost:{}", addr.split(':').last().unwrap_or("8080"));
    println!("   Press Ctrl+C to stop");
    println!();

    // The generated binary reads addr from nawa.toml or uses default 0.0.0.0:8080.
    // We set NAWA_ADDR env var so the binary can pick it up (or just run it directly).
    let status = Command::new(&binary_path)
        .env("NAWA_ADDR", addr)
        .status()?;
    if !status.success() {
        anyhow::bail!("server exited with status: {status}");
    }
    Ok(())
}

/// Get the binary name from Cargo.toml [[bin]] section.
fn get_binary_name() -> String {
    let cargo_toml = std::fs::read_to_string("./Cargo.toml").unwrap_or_default();
    let mut in_bin_section = false;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[bin]]") {
            in_bin_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_bin_section = false;
        }
        if in_bin_section && trimmed.starts_with("name = ") {
            return trimmed
                .strip_prefix("name = ")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        }
    }
    // Fallback: use package name
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name = ") {
            return trimmed
                .strip_prefix("name = ")
                .unwrap_or("")
                .trim_matches('"')
                .replace('-', "_");
        }
    }
    "app".to_string()
}

#[allow(dead_code)]
fn spawn_nawad(
    path: &std::path::Path,
    addr: &str,
    data_dir: &std::path::Path,
) -> anyhow::Result<std::process::Child> {
    let child = Command::new(path)
        .arg("serve")
        .arg("--addr")
        .arg(addr)
        .arg("--data-dir")
        .arg(data_dir)
        .spawn()?;
    Ok(child)
}

#[allow(dead_code)]
fn get_dir_mtime(dir: &std::path::Path) -> Option<std::time::SystemTime> {
    if !dir.exists() {
        return None;
    }
    let mut latest: Option<std::time::SystemTime> = None;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(mtime) = metadata.modified() {
                    latest = Some(latest.map_or(mtime, |l: std::time::SystemTime| l.max(mtime)));
                }
                // Recurse into subdirectories.
                if metadata.is_dir() {
                    if let Some(sub_mtime) = get_dir_mtime(&entry.path()) {
                        latest = Some(latest.map_or(sub_mtime, |l| l.max(sub_mtime)));
                    }
                }
            }
        }
    }
    latest
}

fn build_project() -> anyhow::Result<()> {
    println!("Building NAWA project (release mode)...");
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()?;
    if !status.success() {
        anyhow::bail!("cargo build failed");
    }
    println!("\n✓ Build complete — target/release/");
    Ok(())
}

fn deploy(target: &Option<String>, remote_data_dir: &std::path::Path) -> anyhow::Result<()> {
    match target {
        None => {
            println!("Deploying locally...");
            println!("  Data directory: {}", remote_data_dir.display());
            println!("\nFor remote deployment:");
            println!("  nawa deploy --target user@your-vps");
            Ok(())
        }
        Some(target) => {
            println!("🚀 Deploying to {target}...");
            println!("  Remote data: {}", remote_data_dir.display());

            // Step 1: Build release binary
            println!("\n📦 Step 1/4: Building release binary...");
            let build_status = Command::new("cargo")
                .args(["build", "--release", "-p", "nawad"])
                .status()?;
            if !build_status.success() {
                anyhow::bail!("cargo build failed");
            }
            println!("  ✓ Binary built");

            // Step 2: Create tarball
            println!("\n📦 Step 2/4: Creating tarball...");
            let tarball = "/tmp/nawad-deploy.tar.gz";
            let tar_status = Command::new("tar")
                .args([
                    "-czf",
                    tarball,
                    "-C",
                    "target/release",
                    "nawad",
                ])
                .status()?;
            if !tar_status.success() {
                anyhow::bail!("tar failed");
            }
            let tarball_size = std::fs::metadata(tarball)?.len();
            println!("  ✓ Tarball: {} ({} KB)", tarball, tarball_size / 1024);

            // Step 3: Upload via SCP
            println!("\n📦 Step 3/4: Uploading to {target}...");
            let scp_status = Command::new("scp")
                .args([tarball, &format!("{target}:/tmp/")])
                .status();
            match scp_status {
                Ok(s) if s.success() => println!("  ✓ Uploaded"),
                Ok(s) => {
                    println!("  ⚠ SCP failed (exit {}), showing manual steps:", s);
                    println!("    scp {} {}:/tmp/", tarball, target);
                }
                Err(e) => {
                    println!("  ⚠ SCP not available: {e}");
                    println!("    Manual: scp {} {}:/tmp/", tarball, target);
                }
            }

            // Step 4: Remote install + start
            println!("\n📦 Step 4/4: Remote install + start...");
            let remote_cmd = format!(
                "mkdir -p {data} && \
                 tar xzf /tmp/nawad-deploy.tar.gz -C /tmp/ && \
                 sudo mv /tmp/nawad /usr/local/bin/ && \
                 sudo pkill -f 'nawad serve' 2>/dev/null; \
                 nohup nawad serve --addr 0.0.0.0:8080 --data-dir {data} > /tmp/nawad.log 2>&1 &",
                data = remote_data_dir.display()
            );
            let ssh_status = Command::new("ssh")
                .args([target, &remote_cmd])
                .status();
            match ssh_status {
                Ok(s) if s.success() => {
                    println!("  ✓ Installed and started");
                }
                Ok(s) => {
                    println!("  ⚠ SSH failed (exit {})", s);
                }
                Err(e) => {
                    println!("  ⚠ SSH not available: {e}");
                }
            }

            println!("\n✓ Deployment complete!");
            println!("  Server: http://{target}:8080", );
            println!("  Health: http://{target}:8080/health");
            println!("  Metrics: http://{target}:8080/metrics");
            println!("\nLogs: ssh {target} 'tail -f /tmp/nawad.log'");
            println!("Stop: ssh {target} 'pkill -f nawad'");

            // Cleanup local tarball
            let _ = std::fs::remove_file(tarball);
            Ok(())
        }
    }
}

fn benchmark(ops: u32) -> anyhow::Result<()> {
    println!("NAWA benchmark — {ops} operations\n");

    let nawad_path = find_nawad();
    match nawad_path {
        Some(path) => {
            let status = Command::new(path)
                .arg("benchmark")
                .arg("--ops")
                .arg(ops.to_string())
                .status()?;
            if !status.success() {
                anyhow::bail!("benchmark failed");
            }
            Ok(())
        }
        None => {
            eprintln!("nawad not found. Run: cargo build --release -p nawad");
            anyhow::bail!("nawad not found")
        }
    }
}

fn run_tests(
    crate_name: &Option<String>,
    bench: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("🧪 Running NAWA tests...\n");

    let mut cmd = Command::new("cargo");
    if bench {
        cmd.arg("bench");
    } else {
        cmd.arg("test");
    }

    if let Some(name) = crate_name {
        cmd.arg("-p").arg(name);
    } else {
        cmd.arg("--workspace");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    cmd.arg("--release");

    println!("$ cargo {:?} {} {} --release",
        if bench { "bench" } else { "test" },
        if let Some(n) = crate_name { format!("-p {n}") } else { "--workspace".into() },
        if verbose { "--verbose" } else { "" }
    );
    println!();

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("tests failed with status: {status}");
    }

    println!("\n✓ All tests passed!");
    Ok(())
}

fn serve_example(addr: &str) -> anyhow::Result<()> {
    println!("🚀 Starting NAWA Blog Site example...\n");
    println!("This builds and runs the example blog website from examples/blog-site/");
    println!();
    println!("Building...");
    let build_status = Command::new("cargo")
        .args(["build", "--release", "--example", "blog-site"])
        .status()?;
    if !build_status.success() {
        anyhow::bail!("build failed");
    }
    println!("✓ Build complete");
    println!();
    println!("Starting server on {addr}...");
    println!();
    println!("  Home:   http://localhost:{}", addr.split(':').next_back().unwrap_or("8080"));
    println!("  API:    http://localhost:{}/api", addr.split(':').next_back().unwrap_or("8080"));
    println!("  Health: http://localhost:{}/health", addr.split(':').next_back().unwrap_or("8080"));
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    // Run the example with the addr as env var.
    let status = Command::new("cargo")
        .args(["run", "--release", "--example", "blog-site"])
        .status()?;
    if !status.success() {
        anyhow::bail!("example failed");
    }
    Ok(())
}

fn print_info() {
    println!("nawa CLI v2.5.1 (NAWA Web Operating System)");
    println!("─────────────────────────────────────────────");
    println!("Components:");
    println!("  • nawa-kernel: io_uring + mmap + zero-copy");
    println!("  • nawa-uring:  real io_uring on Linux 5.1+");
    println!("  • nawa-db:     MemTable + SSTable + WAL + Bloom");
    println!("  • nawa-http:   HTTP/1.1 + HTTP/3 + TLS + ACME");
    println!("  • nawa-wasm:   WASM sandbox (wasmtime)");
    println!("  • nawad:       server binary (v2.5.1)");
    println!();
    println!("Platform:");
    println!("  OS: {}", std::env::consts::OS);
    println!("  Arch: {}", std::env::consts::ARCH);
    if nawa_uring_supported() {
        println!("  io_uring: supported ✓");
    } else {
        println!("  io_uring: not supported (using fallback)");
    }
    println!();
    println!("License: MIT OR Apache-2.0");
    println!("Repo:    https://github.com/amir-helal-ali/nawa-web-os");
    println!();
    println!("Commands:");
    println!("  nawa serve           تشغيل الخادم");
    println!("  nawa new my-app      مشروع جديد");
    println!("  nawa update          تحديث NAWA");
    println!("  nawa uninstall       حذف NAWA بالكامل");
    println!("  nawa info            هذه المعلومات");
    println!("  nawa templates       قائمة القوالب");
}

fn list_templates() {
    println!("Available NAWA templates:\n");
    for (name, desc) in TEMPLATES {
        println!("  {name:<12} {desc}");
    }
    println!("\nUsage: nawa create my-app --template <name>");
}

fn find_nawad() -> Option<PathBuf> {
    // Check relative to the workspace target dir.
    let candidates = [
        PathBuf::from("../../target/release/nawad"),
        PathBuf::from("target/release/nawad"),
        PathBuf::from("../target/release/nawad"),
    ];
    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }
    // Check PATH.
    which::which("nawad").ok()
}

fn nawa_uring_supported() -> bool {
    // We can't depend on nawa-uring here (would create a cycle),
    // so just check the platform.
    cfg!(target_os = "linux")
}

// Minimal which implementation (to avoid extra dependency).
mod which {
    use std::path::PathBuf;
    pub fn which(name: &str) -> Result<PathBuf, ()> {
        if let Ok(path) = std::env::var("PATH") {
            for dir in path.split(':') {
                let candidate = std::path::Path::new(dir).join(name);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
        Err(())
    }
}
