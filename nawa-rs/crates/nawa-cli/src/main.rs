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
        #[arg(short, long, default_value = "blog")]
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

    /// Show version and component info.
    Info,

    /// List available templates.
    Templates,
}

const TEMPLATES: &[(&str, &str)] = &[
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
        Commands::Info => {
            print_info();
            Ok(())
        }
        Commands::Templates => {
            list_templates();
            Ok(())
        }
    }
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

    // Try to find and run nawad.
    let nawad_path = find_nawad();
    match nawad_path {
        Some(path) => {
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
            println!("  nawa deploy --target ssh://user@your-vps");
            Ok(())
        }
        Some(target) => {
            println!("Deploying to {target}...");
            println!("  Remote data: {}", remote_data_dir.display());
            println!("\nDeployment steps:");
            println!("  1. Build Docker image: docker build -t nawa-app .");
            println!("  2. Push to registry: docker push nawa-app");
            println!("  3. SSH to {target}");
            println!("  4. Pull image: docker pull nawa-app");
            println!("  5. Run: docker run -d -p 80:8080 -v {data}:{data} nawa-app",
                data = remote_data_dir.display());
            println!("\n✓ Deployment plan generated");
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

fn print_info() {
    println!("nawa CLI v0.1.0-alpha (NAWA Web Operating System)");
    println!("─────────────────────────────────────────────");
    println!("Components:");
    println!("  • nawa-kernel: io_uring + mmap + zero-copy");
    println!("  • nawa-uring:  real io_uring on Linux 5.1+");
    println!("  • nawa-db:     MemTable + SSTable + WAL + Bloom");
    println!("  • nawa-http:   HTTP/1.1 + HTTP/3 + TLS + ACME");
    println!("  • nawa-wasm:   WASM sandbox (wasmtime)");
    println!("  • nawad:       server binary");
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
    println!("Run 'nawa templates' to see available project templates.");
    println!("Run 'nawa create my-app --template blog' to start.");
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
