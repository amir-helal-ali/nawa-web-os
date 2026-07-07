# Creating a Project

## Quick Start

```bash
# Create a blog project
nawa create my-blog --template blog

# Enter the directory
cd my-blog

# Start the dev server
nawa dev
```

## Available Templates

| Template | Use Case |
|----------|----------|
| `blog` | Personal blog, CMS, online magazine |
| `saas` | Multi-tenant SaaS with billing |
| `shop` | E-commerce store |
| `realtime` | Chat app, live dashboard |
| `booking` | Appointment booking |
| `portfolio` | Personal portfolio site |

## Generated Structure

```
my-blog/
├── Cargo.toml          # dependencies
├── Dockerfile          # production deployment
├── README.md           # project docs
├── .gitignore
└── src/
    ├── main.rs         # entry point + 4 routes
    ├── routes/         # add your handlers here
    ├── db/             # DB schema files
    └── templates/      # HTML templates (for SSR)
```

## Customizing

Edit `src/main.rs` to add routes:

```rust
router.get("/about", |_| async {
    Response::text("About page")
});

router.get("/api/users", |_| async {
    Response::json(&serde_json::json!({"users": []}))
});
```
