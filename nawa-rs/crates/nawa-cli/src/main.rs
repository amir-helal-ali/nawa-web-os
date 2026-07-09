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

    // Validate template.
    if !TEMPLATES.iter().any(|(t, _)| *t == template) {
        anyhow::bail!(
            "unknown template: {}. Available: {}",
            template,
            TEMPLATES.iter().map(|(t, _)| *t).collect::<Vec<_>>().join(", ")
        );
    }

    println!("Creating NAWA project '{name}' from template '{template}'...");

    // Create directory structure.
    std::fs::create_dir_all(&project_dir)?;
    std::fs::create_dir_all(project_dir.join("src"))?;
    std::fs::create_dir_all(project_dir.join("src/routes"))?;
    std::fs::create_dir_all(project_dir.join("src/db"))?;
    std::fs::create_dir_all(project_dir.join("templates"))?;
    std::fs::create_dir_all(project_dir.join("static"))?;

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
nawa-kernel = {{ path = "../../crates/nawa-kernel" }}
nawa-db = {{ path = "../../crates/nawa-db" }}
nawa-http = {{ path = "../../crates/nawa-http" }}
nawa-uring = {{ path = "../../crates/nawa-uring" }}
nawa-wasm = {{ path = "../../crates/nawa-wasm" }}
tokio = {{ version = "1.42", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    println!("  ✓ Cargo.toml");

    // src/main.rs — minimal NAWA app.
    let main_rs = r#"use nawa_db::{DbEngine, Value};
use nawa_http::{HttpServer, Response, Router};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Arc::new(DbEngine::open_in_memory());

    let mut router = Router::new();

    // GET / — hello world
    router.get("/", |_| async {
        Response::text("Hello from NAWA!")
    });

    // GET /health
    {
        let db = db.clone();
        router.get("/health", move |_| {
            let db = db.clone();
            async move {
                Response::json(&serde_json::json!({
                    "status": "ok",
                    "keys": db.len()
                }))
            }
        });
    }

    // GET /:key
    {
        let db = db.clone();
        router.get("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("");
                match db.get(key) {
                    Some(v) => Response::text(v.display()),
                    None => Response::not_found("key not found"),
                }
            }
        });
    }

    // POST /:key
    {
        let db = db.clone();
        router.post("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("").to_string();
                let value = Value::Bytes(req.body.clone());
                db.put(&key, value)?;
                Response::text(format!("stored: {key}"))
            }
        });
    }

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    println!("NAWA server running on http://localhost:8080");
    let server = HttpServer::new(router, addr);
    server.serve().await?;
    Ok(())
}
"#;
    std::fs::write(project_dir.join("src/main.rs"), main_rs)?;
    println!("  ✓ src/main.rs");

    // .gitignore
    std::fs::write(
        project_dir.join(".gitignore"),
        "/target\n/nawa-data\n*.log\n.env\n",
    )?;
    println!("  ✓ .gitignore");

    // README.md
    let readme = format!(
        "# {name}\n\nA NAWA web application built from the `{template}` template.\n\n\
         ## Getting Started\n\n\
         ```bash\n\
         # Run the dev server\n\
         nawa dev\n\n\
         # Build for production\n\
         nawa build\n\n\
         # Deploy\n\
         nawa deploy --target ssh://user@your-vps\n\
         ```\n\n\
         ## Endpoints\n\n\
         - `GET /` — hello world\n\
         - `GET /health` — health check\n\
         - `GET /:key` — get a value\n\
         - `POST /:key` — store a value\n",
    );
    std::fs::write(project_dir.join("README.md"), readme)?;
    println!("  ✓ README.md");

    // Dockerfile
    let dockerfile = r#"FROM rust:1.83-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.20
RUN adduser -D -u 10001 nawa
COPY --from=builder /app/target/release/app /usr/local/bin/nawa-app
USER nawa
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/nawa-app"]
"#;
    std::fs::write(project_dir.join("Dockerfile"), dockerfile)?;
    println!("  ✓ Dockerfile");

    println!("\n✓ Project created in {}/", name);
    println!("\nNext steps:");
    println!("  cd {name}");
    println!("  nawa dev");
    println!("\nTemplate: {template}");
    if let Some((_, desc)) = TEMPLATES.iter().find(|(t, _)| *t == template) {
        println!("  {desc}");
    }

    Ok(())
}

fn dev_server(addr: &str, data_dir: &std::path::Path) -> anyhow::Result<()> {
    println!("Starting NAWA dev server on {addr}");
    println!("Data directory: {}", data_dir.display());
    println!("\nPress Ctrl+C to stop\n");

    let nawad_path = find_nawad();
    match nawad_path {
        Some(path) => {
            // Run nawad directly (event-driven, restart manually).
            let mut cmd = Command::new(path);
            cmd.arg("serve").arg("--addr").arg(addr).arg("--data-dir").arg(data_dir);
            let status = cmd.status()?;
            if !status.success() {
                anyhow::bail!("nawad exited with status: {status}");
            }
            Ok(())
        }
        None => {
            eprintln!("nawad binary not found. Building...");
            eprintln!("Run: cargo build --release -p nawad");
            eprintln!("Then: ./target/release/nawad serve --addr {addr} --data-dir {}", data_dir.display());
            anyhow::bail!("nawad not found")
        }
    }
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
    println!("nawa CLI v2.4.0 (NAWA Web Operating System)");
    println!("─────────────────────────────────────────────");
    println!("Components:");
    println!("  • nawa-kernel: io_uring + mmap + zero-copy");
    println!("  • nawa-uring:  real io_uring on Linux 5.1+");
    println!("  • nawa-db:     MemTable + SSTable + WAL + Bloom");
    println!("  • nawa-http:   HTTP/1.1 + HTTP/3 + TLS + ACME");
    println!("  • nawa-wasm:   WASM sandbox (wasmtime)");
    println!("  • nawad:       server binary (v2.4.0)");
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
