"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  ArrowRight,
  Database,
  Code2,
  Server,
  CheckCircle2,
  AlertTriangle,
  FileCode2,
  Terminal,
  Zap,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Source = "node" | "django" | "rails" | "nextjs" | "go";

const SOURCES: Record<
  Source,
  {
    name: string;
    nameAr: string;
    color: string;
    difficulty: "easy" | "medium" | "hard";
    timeWeeks: number;
    steps: MigrationStep[];
  }
> = {
  node: {
    name: "Node.js + Express",
    nameAr: "نود + إكسبرس",
    color: "oklch(0.65 0.18 140)",
    difficulty: "easy",
    timeWeeks: 2,
    steps: [
      {
        title: "تحويل Routes",
        desc: "كل route Express يصبح #[get(...)] handler في Rust.",
        before: `app.get('/users/:id', async (req, res) => {
  const user = await db.users.findById(req.params.id);
  res.json(user);
});`,
        after: `#[get("/users/:id")]
pub fn get_user(Path(id): Path<u64>) -> Json<User> {
    let user = nawa_db::get(&format!("user:{}", id))
        .unwrap_or_default();
    Json(user)
}`,
        automated: true,
      },
      {
        title: "ترحيل PostgreSQL schema",
        desc: "كل table يصبح NAWA-DB document collection.",
        before: `CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255),
  email VARCHAR(255) UNIQUE
);`,
        after: `// schema.nawa
collection users {
  id: u64 @id,
  name: String,
  email: String @unique
}`,
        automated: true,
      },
      {
        title: "استبدال ORM",
        desc: "Mongoose/Sequelize → NAWA-DB native API.",
        before: `const user = await User.findById(1001);
await user.update({ name: 'new' });`,
        after: `let mut user: User = nawa_db::get("user:1001")?;
user.name = "new".into();
nawa_db::put("user:1001", &user)?;`,
        automated: false,
      },
      {
        title: "Middleware chain",
        desc: "Express middleware → tower::Layer chain.",
        before: `app.use(cors());
app.use(helmet());
app.use(auth);`,
        after: `let app = Router::new()
    .layer(CorsLayer::default())
    .layer(SecurityHeadersLayer)
    .layer(AuthLayer::new());`,
        automated: false,
      },
    ],
  },
  django: {
    name: "Django + DRF",
    nameAr: "جانغو + DRF",
    color: "oklch(0.55 0.18 50)",
    difficulty: "medium",
    timeWeeks: 4,
    steps: [
      {
        title: "تحويل Views",
        desc: "Django views → Rust handlers.",
        before: `@api_view(['GET'])
def get_user(request, id):
    user = User.objects.get(pk=id)
    return Response(UserSerializer(user).data)`,
        after: `#[get("/users/:id")]
pub fn get_user(Path(id): Path<u64>) -> Json<User> {
    let user: User = nawa_db::get(&format!("user:{}", id))
        .unwrap_or_default();
    Json(user)
}`,
        automated: true,
      },
      {
        title: "ترحيل Models",
        desc: "Django models → NAWA-DB documents.",
        before: `class User(models.Model):
    name = models.CharField(max_length=255)
    email = models.EmailField(unique=True)`,
        after: `#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}`,
        automated: true,
      },
      {
        title: "استبدال Django ORM",
        desc: "querysets → NAWA-DB scan + filter.",
        before: `User.objects.filter(
    name__icontains='ahmed',
    is_active=True
)[:10]`,
        after: `nawa_db::scan("user:")
    .filter(|u| u.name.contains("ahmed") && u.active)
    .take(10)`,
        automated: false,
      },
      {
        title: "Templates → SSR",
        desc: "Django templates → hypertext renderer.",
        before: `{% extends "base.html" %}
{% block content %}
  <h1>{{ user.name }}</h1>
{% endblock %}`,
        after: `pub fn user_page(user: User) -> Html {
    html! {
        h1 { (user.name) }
    }
}`,
        automated: false,
      },
    ],
  },
  rails: {
    name: "Ruby on Rails",
    nameAr: "روبي أون ريلز",
    color: "oklch(0.55 0.22 25)",
    difficulty: "medium",
    timeWeeks: 4,
    steps: [
      {
        title: "تحويل Controllers",
        desc: "Rails controllers → Rust handlers.",
        before: `class UsersController < ApplicationController
  def show
    @user = User.find(params[:id])
    render json: @user
  end
end`,
        after: `#[get("/users/:id")]
pub fn show(Path(id): Path<u64>) -> Json<User> {
    let user: User = nawa_db::get(&format!("user:{}", id))?;
    Json(user)
}`,
        automated: true,
      },
      {
        title: "Active Record → NAWA-DB",
        desc: "Models + migrations.",
        before: `class User < ApplicationRecord
  validates :email, uniqueness: true
  has_many :posts
end`,
        after: `#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub posts: Vec<u64>,
}`,
        automated: true,
      },
      {
        title: "Active Record queries",
        desc: "Rails queries → NAWA-DB.",
        before: `User.where(active: true)
     .includes(:posts)
     .limit(10)`,
        after: `nawa_db::scan("user:")
    .filter(|u| u.active)
    .take(10)`,
        automated: false,
      },
      {
        title: "ERB templates → SSR",
        desc: "Rails views → hypertext.",
        before: `<h1><%= @user.name %></h1>
<% @posts.each do |post| %>
  <p><%= post.title %></p>
<% end %>`,
        after: `html! {
    h1 { (user.name) }
    @for post in posts {
        p { (post.title) }
    }
}`,
        automated: false,
      },
    ],
  },
  nextjs: {
    name: "Next.js + Vercel",
    nameAr: "نكست جي إس",
    color: "oklch(0.65 0.18 280)",
    difficulty: "easy",
    timeWeeks: 2,
    steps: [
      {
        title: "Pages → Handlers",
        desc: "Next.js API routes → Rust handlers.",
        before: `export default function handler(req, res) {
  const user = await db.user.findUnique({
    where: { id: req.query.id }
  });
  res.status(200).json(user);
}`,
        after: `#[get("/api/users/:id")]
pub fn handler(Path(id): Path<u64>) -> Json<User> {
    let user: User = nawa_db::get(&format!("user:{}", id))?;
    Json(user)
}`,
        automated: true,
      },
      {
        title: "getServerSideProps → SSR",
        desc: "Data fetching → server-side render.",
        before: `export async function getServerSideProps() {
  const posts = await fetch('/api/posts');
  return { props: { posts } };
}`,
        after: `#[get("/")]
pub fn index() -> Html {
    let posts: Vec<Post> = nawa_db::scan("post:");
    html! {
        @for post in posts {
            article { h2 { (post.title) } }
        }
    }
}`,
        automated: true,
      },
      {
        title: "Prisma → NAWA-DB",
        desc: "schema.prisma → schema.nawa.",
        before: `model User {
  id    Int    @id @default(autoincrement())
  email String @unique
  posts Post[]
}`,
        after: `collection users {
  id: u64 @id @auto,
  email: String @unique,
  posts: [u64] @ref("posts")
}`,
        automated: true,
      },
      {
        title: "Vercel deployment",
        desc: "Vercel → any VPS with Docker.",
        before: `# vercel.json
{
  "version": 2,
  "builds": [{ "src": "*.js" }]
}`,
        after: `# docker-compose.yml
services:
  nawa:
    image: nawa/os:0.1.0
    ports: ["80:8080"]
    deploy:
      resources:
        limits: { memory: 512M }`,
        automated: false,
      },
    ],
  },
  go: {
    name: "Go + Gin",
    nameAr: "غو + غين",
    color: "oklch(0.65 0.18 200)",
    difficulty: "easy",
    timeWeeks: 2,
    steps: [
      {
        title: "Gin handlers → Rust",
        desc: "Gin routes → #[get(...)] handlers.",
        before: `r.GET("/users/:id", func(c *gin.Context) {
    id := c.Param("id")
    user := db.Find(id)
    c.JSON(200, user)
})`,
        after: `#[get("/users/:id")]
pub fn get_user(Path(id): Path<u64>) -> Json<User> {
    let user: User = nawa_db::get(&format!("user:{}", id))?;
    Json(user)
}`,
        automated: true,
      },
      {
        title: "GORM → NAWA-DB",
        desc: "Go ORM → native Rust API.",
        before: `db.Where("active = ?", true).
   Find(&users)`,
        after: `let users: Vec<User> = nawa_db::scan("user:")
    .filter(|u| u.active)
    .collect();`,
        automated: false,
      },
      {
        title: " Goroutines → tokio",
        desc: "Go concurrency → Rust async.",
        before: `go func() {
    process(data)
}()`,
        after: `tokio::spawn(async {
    process(data).await
});`,
        automated: false,
      },
      {
        title: "Dockerfile",
        desc: "Go Dockerfile → NAWA multi-stage.",
        before: `FROM golang:1.21 AS build
COPY . .
RUN go build -o app

FROM alpine
COPY --from=build /app /app
CMD ["/app"]`,
        after: `FROM rust:1.83-alpine AS builder
COPY . .
RUN cargo build --release

FROM alpine:3.20
COPY --from=builder /target/release/nawad /nawad
CMD ["/nawad"]`,
        automated: true,
      },
    ],
  },
};

type MigrationStep = {
  title: string;
  desc: string;
  before: string;
  after: string;
  automated: boolean;
};

const DIFFICULTY_CONFIG = {
  easy: { color: "text-accent", bg: "bg-accent/15", label: "سهل", time: "أسبوع-أسبوعين" },
  medium: { color: "text-yellow-500", bg: "bg-yellow-500/15", label: "متوسط", time: "3-4 أسابيع" },
  hard: { color: "text-destructive", bg: "bg-destructive/15", label: "صعب", time: "5-8 أسابيع" },
};

export function MigrationGuide() {
  const [source, setSource] = useState<Source>("node");
  const [activeStep, setActiveStep] = useState(0);
  const src = SOURCES[source];
  const step = src.steps[activeStep];
  const diff = DIFFICULTY_CONFIG[src.difficulty];

  return (
    <section id="migration" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="دليل الهجرة"
          eyebrowEn="Migration Guide"
          title="انتقل من أي stack إلى NAWA"
          titleEn="Migrate from any stack to NAWA"
          desc="أدوات هجرة شبه آمية لكل stack شهير. اختر مصدرك الحالي وستحصل على دليل خطوة بخطوة مع مقارنات كود قبل/بعد."
          descEn="Semi-automated migration tools for every popular stack. Pick your current stack to get a step-by-step guide with before/after code comparisons."
        />

        {/* Source selector */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex flex-wrap gap-2"
        >
          {(Object.keys(SOURCES) as Source[]).map((key) => {
            const s = SOURCES[key];
            return (
              <button
                key={key}
                onClick={() => {
                  setSource(key);
                  setActiveStep(0);
                }}
                className={`group px-4 py-3 rounded-xl border text-right transition-all ${
                  source === key
                    ? "border-primary bg-primary/10"
                    : "border-border/60 bg-card/40 hover:border-border"
                }`}
              >
                <div className="flex items-center gap-2">
                  <span
                    className="w-2 h-2 rounded-full"
                    style={{ background: s.color }}
                  />
                  <div>
                    <div className={`text-sm font-medium ${source === key ? "text-primary" : "text-foreground"}`}>
                      {s.nameAr}
                    </div>
                    <div className="text-[10px] text-muted-foreground mono">{s.name}</div>
                  </div>
                </div>
              </button>
            );
          })}
        </motion.div>

        {/* Migration summary */}
        <motion.div
          key={source}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
          className="mt-6 grid sm:grid-cols-3 gap-3"
        >
          <div className={`p-4 rounded-xl border border-border/60 ${diff.bg}`}>
            <div className="flex items-center gap-2 mb-1">
              <AlertTriangle className={`w-4 h-4 ${diff.color}`} />
              <span className="text-[10px] text-muted-foreground uppercase mono">difficulty</span>
            </div>
            <div className={`text-lg font-bold ${diff.color} ar`}>{diff.label}</div>
          </div>
          <div className="p-4 rounded-xl border border-border/60 bg-card/60">
            <div className="flex items-center gap-2 mb-1">
              <Zap className="w-4 h-4 text-primary" />
              <span className="text-[10px] text-muted-foreground uppercase mono">est. time</span>
            </div>
            <div className="text-lg font-bold text-primary ar">{src.timeWeeks} أسابيع</div>
          </div>
          <div className="p-4 rounded-xl border border-border/60 bg-card/60">
            <div className="flex items-center gap-2 mb-1">
              <CheckCircle2 className="w-4 h-4 text-accent" />
              <span className="text-[10px] text-muted-foreground uppercase mono">automation</span>
            </div>
            <div className="text-lg font-bold text-accent">
              {src.steps.filter((s) => s.automated).length}/{src.steps.length}
            </div>
          </div>
        </motion.div>

        {/* Steps navigation */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8 flex flex-wrap gap-2"
        >
          {src.steps.map((s, i) => (
            <button
              key={i}
              onClick={() => setActiveStep(i)}
              className={`px-3 py-1.5 rounded-lg text-xs border transition-all flex items-center gap-2 ${
                activeStep === i
                  ? "border-primary bg-primary/10 text-primary"
                  : "border-border/60 bg-card/40 text-muted-foreground hover:text-foreground"
              }`}
            >
              <span className="mono w-4 h-4 rounded-full bg-muted grid place-items-center text-[10px]">
                {i + 1}
              </span>
              <span className="ar">{s.title}</span>
              {s.automated && (
                <Badge variant="outline" className="text-[9px] mono border-accent/40 text-accent">
                  auto
                </Badge>
              )}
            </button>
          ))}
        </motion.div>

        {/* Active step comparison */}
        <AnimatePresence mode="wait">
          <motion.div
            key={`${source}-${activeStep}`}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.4 }}
            className="mt-6"
          >
            {/* Step header */}
            <div className="mb-4 p-4 rounded-xl border border-border/60 bg-card/60">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h4 className="text-base font-semibold ar">{step.title}</h4>
                  <p className="text-xs text-muted-foreground mt-1 ar">{step.desc}</p>
                </div>
                {step.automated ? (
                  <Badge className="bg-accent/15 border border-accent/40 text-accent mono text-[10px]">
                    <CheckCircle2 className="w-2.5 h-2.5 ml-1" />
                    أداة آلية
                  </Badge>
                ) : (
                  <Badge variant="outline" className="border-yellow-500/40 text-yellow-500 mono text-[10px]">
                    <AlertTriangle className="w-2.5 h-2.5 ml-1" />
                    يدوي
                  </Badge>
                )}
              </div>
            </div>

            {/* Code comparison */}
            <div className="grid lg:grid-cols-2 gap-4">
              {/* Before */}
              <div className="rounded-2xl border border-destructive/30 overflow-hidden bg-[#0d0c0a]">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-destructive/10">
                  <div className="flex items-center gap-2">
                    <FileCode2 className="w-4 h-4 text-destructive" />
                    <span className="text-xs font-medium mono">before · {src.name}</span>
                  </div>
                  <span className="text-[10px] text-muted-foreground mono">step {activeStep + 1}a</span>
                </div>
                <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono">
                  <code className="text-foreground/80 whitespace-pre">
                    {step.before.split("\n").map((line, i) => (
                      <div key={i}>{line || " "}</div>
                    ))}
                  </code>
                </pre>
              </div>

              {/* After */}
              <div className="rounded-2xl border border-primary/40 overflow-hidden bg-[#0d0c0a]">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-primary/10">
                  <div className="flex items-center gap-2">
                    <FileCode2 className="w-4 h-4 text-primary" />
                    <span className="text-xs font-medium mono">after · NAWA</span>
                  </div>
                  <span className="text-[10px] text-muted-foreground mono">step {activeStep + 1}b</span>
                </div>
                <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono">
                  <code className="text-foreground/90 whitespace-pre">
                    {step.after.split("\n").map((line, i) => (
                      <div key={i} className="hover:bg-primary/5 -mx-4 px-4">
                        {line || " "}
                      </div>
                    ))}
                  </code>
                </pre>
              </div>
            </div>

            {/* Arrow indicator (desktop) */}
            <div className="hidden lg:flex absolute -mt-32 left-1/2 -translate-x-1/2 z-10 pointer-events-none">
              <div className="w-10 h-10 rounded-full bg-primary grid place-items-center glow-amber">
                <ArrowRight className="w-5 h-5 text-primary-foreground rotate-180" />
              </div>
            </div>
          </motion.div>
        </AnimatePresence>

        {/* Migration tools */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-12 p-6 rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 to-accent/5"
        >
          <div className="flex items-center gap-2 mb-3">
            <Terminal className="w-4 h-4 text-primary" />
            <h4 className="text-sm font-semibold ar">أدوات الهجرة المدمجة</h4>
          </div>
          <p className="text-xs text-muted-foreground mb-4 ar">
            NAWA CLI يأتي مع أدوات هجرة آلية. جرّبها في الـ CLI Simulator بالأعلى:
          </p>
          <div className="grid sm:grid-cols-2 gap-2">
            {[
              `nawa migrate from-node ./old-app`,
              `nawa migrate from-django ./project`,
              `nawa migrate from-rails ./rails-app`,
              `nawa migrate from-nextjs ./next-app`,
              `nawa migrate from-go ./go-app`,
              `nawa migrate --dry-run ./old-app`,
            ].map((cmd) => (
              <code
                key={cmd}
                className="px-3 py-2 rounded-lg bg-card/60 border border-border/40 text-xs mono text-primary cursor-pointer hover:bg-card"
                onClick={() => navigator.clipboard?.writeText(cmd)}
              >
                $ {cmd}
              </code>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  );
}
