<script>
  import { onMount } from 'svelte';
  import Counter from './Counter.svelte';
  import NawaState from './NawaState.svelte';
  import LiveStats from './LiveStats.svelte';
  import QuantumViz from './QuantumViz.svelte';

  let loaded = false;
  let mouseX = 0;
  let mouseY = 0;

  onMount(() => {
    loaded = true;
    window.addEventListener('mousemove', (e) => {
      mouseX = e.clientX;
      mouseY = e.clientY;
    });
  });
</script>

<div class="app" class:loaded>
  <!-- Animated background grid -->
  <div class="grid-bg" style="--mx: {mouseX}px; --my: {mouseY}px;"></div>

  <!-- Top navigation -->
  <nav class="nav">
    <div class="nav-brand">
      <span class="logo">🦀</span>
      <span class="brand-text">NAWA</span>
      <span class="version-badge">v1.9.0</span>
    </div>
    <div class="nav-links">
      <a href="#features">الميزات</a>
      <a href="#quantum">الكوانتم</a>
      <a href="#stats">الإحصائيات</a>
      <a href="/docs" target="_blank">API Docs</a>
    </div>
    <button class="theme-toggle" onclick="window.nawaToggleTheme?.()">🌙</button>
  </nav>

  <!-- Hero section -->
  <section class="hero">
    <div class="hero-content">
      <div class="hero-badge">
        <span class="pulse-dot"></span>
        Quantum-Powered Web OS
      </div>
      <h1 class="hero-title">
        <span class="gradient-text">نظام تشغيل الويب</span>
        <span class="gradient-text-alt">الثوري</span>
      </h1>
      <p class="hero-subtitle">
        Binary واحد خالص · بدون Node.js · ميكانيكا كمية · صفر نسخ
      </p>
      <div class="hero-actions">
        <a href="/register" class="btn btn-primary">
          ابدأ الآن →
        </a>
        <a href="/docs" class="btn btn-secondary">
          📚 التوثيق
        </a>
      </div>
    </div>

    <!-- Floating stats card -->
    <div class="hero-card">
      <div class="card-glow"></div>
      <div class="stat-row">
        <span class="stat-label">Endpoints</span>
        <span class="stat-value">77</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Tests</span>
        <span class="stat-value">450+</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Binary</span>
        <span class="stat-value">10MB</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Node.js</span>
        <span class="stat-value danger">غير مطلوب</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Polling</span>
        <span class="stat-value danger">صفر</span>
      </div>
    </div>
  </section>

  <!-- Features grid -->
  <section id="features" class="features">
    <h2 class="section-title">الميزات الثورية</h2>
    <div class="features-grid">
      <div class="feature-card" style="--delay: 0ms">
        <div class="feature-icon">⚛️</div>
        <h3>الميكانيكا الكمية</h3>
        <p>تراكب، تشابك، نفق كمي، تصحيح أخطاء كمي — كلها مدمجة في النواة</p>
        <div class="feature-tags">
          <span class="tag">Superposition</span>
          <span class="tag">Entanglement</span>
          <span class="tag">Tunneling</span>
        </div>
      </div>

      <div class="feature-card" style="--delay: 100ms">
        <div class="feature-icon">🌐</div>
        <h3>AION SEO Engine</h3>
        <p>Knowledge Graph تلقائي + 9 صيغ استجابة + Photon Protocol + Self-Healing</p>
        <div class="feature-tags">
          <span class="tag">Knowledge Graph</span>
          <span class="tag">Photon</span>
          <span class="tag">QEC</span>
        </div>
      </div>

      <div class="feature-card" style="--delay: 200ms">
        <div class="feature-icon">⚡</div>
        <h3>WASM SSR</h3>
        <p>Rust → WASM → HTML بدون Node.js — 74KB module فقط</p>
        <div class="feature-tags">
          <span class="tag">wasmtime</span>
          <span class="tag">Zero-copy</span>
          <span class="tag">Sandbox</span>
        </div>
      </div>

      <div class="feature-card" style="--delay: 300ms">
        <div class="feature-icon">🔌</div>
        <h3>WebSocket Pub/Sub</h3>
        <p>إشعارات لحظية بـ 6 قنوات — صفر polling، 100% push</p>
        <div class="feature-tags">
          <span class="tag">Real-time</span>
          <span class="tag">6 Channels</span>
          <span class="tag">Event Bus</span>
        </div>
      </div>

      <div class="feature-card" style="--delay: 400ms">
        <div class="feature-icon">🛡️</div>
        <h3>أمان عالي</h3>
        <p>11 security headers + CSRF + Audit log + Rate limiting + Session management</p>
        <div class="feature-tags">
          <span class="tag">CSP</span>
          <span class="tag">HSTS</span>
          <span class="tag">RBAC</span>
        </div>
      </div>

      <div class="feature-card" style="--delay: 500ms">
        <div class="feature-icon">🎨</div>
        <h3>تصميم احترافي</h3>
        <p>3 ثيمات (Dark/Light/Auto) + Glassmorphism + Animations + RTL</p>
        <div class="feature-tags">
          <span class="tag">3 Themes</span>
          <span class="tag">Glassmorphism</span>
          <span class="tag">RTL</span>
        </div>
      </div>
    </div>
  </section>

  <!-- Quantum visualization -->
  <section id="quantum" class="quantum-section">
    <h2 class="section-title">المحرك الكمي</h2>
    <QuantumViz />
  </section>

  <!-- Live stats -->
  <section id="stats" class="stats-section">
    <h2 class="section-title">إحصائيات لحظية</h2>
    <LiveStats />
    <NawaState />
    <Counter />
  </section>

  <!-- Footer -->
  <footer class="footer">
    <div class="footer-content">
      <p>🦀 NAWA Web Operating System — Revolutionary</p>
      <div class="footer-links">
        <a href="https://github.com/amir-helal-ali/nawa-web-os">GitHub</a>
        <a href="/docs">Docs</a>
        <a href="/api">API</a>
      </div>
    </div>
  </footer>
</div>

<style>
  :root {
    --bg: #0a0a0f;
    --surface: rgba(20, 20, 30, 0.6);
    --border: rgba(245, 158, 11, 0.15);
    --primary: #f59e0b;
    --accent: #10b981;
    --text: #e8e8ef;
    --muted: #8b8b9a;
    --danger: #ef4444;
    --font: 'Noto Sans Arabic', system-ui, sans-serif;
  }

  * { margin: 0; padding: 0; box-sizing: border-box; }

  .app {
    font-family: var(--font);
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
    overflow-x: hidden;
    opacity: 0;
    transition: opacity 0.5s;
  }
  .app.loaded { opacity: 1; }

  /* Animated grid background */
  .grid-bg {
    position: fixed;
    inset: 0;
    background-image:
      linear-gradient(rgba(245, 158, 11, 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(245, 158, 11, 0.03) 1px, transparent 1px);
    background-size: 50px 50px;
    pointer-events: none;
    z-index: 0;
    mask-image: radial-gradient(circle at var(--mx) var(--my), rgba(0,0,0,0.5), transparent 300px);
    -webkit-mask-image: radial-gradient(circle at var(--mx) var(--my), rgba(0,0,0,0.5), transparent 300px);
  }

  /* Navigation */
  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 2rem;
    position: sticky;
    top: 0;
    z-index: 100;
    background: rgba(10, 10, 15, 0.8);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--border);
  }
  .nav-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.5rem;
    font-weight: 800;
  }
  .logo { font-size: 1.8rem; }
  .brand-text {
    background: linear-gradient(135deg, var(--primary), var(--accent));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .version-badge {
    font-size: 0.65rem;
    padding: 0.15rem 0.5rem;
    border-radius: 10px;
    background: rgba(245, 158, 11, 0.15);
    color: var(--primary);
    font-weight: 600;
  }
  .nav-links { display: flex; gap: 1.5rem; }
  .nav-links a {
    color: var(--muted);
    text-decoration: none;
    font-size: 0.9rem;
    transition: color 0.2s;
  }
  .nav-links a:hover { color: var(--primary); }
  .theme-toggle {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border);
    border-radius: 50%;
    width: 36px;
    height: 36px;
    cursor: pointer;
    font-size: 1.1rem;
    color: var(--muted);
    transition: all 0.2s;
  }
  .theme-toggle:hover { border-color: var(--primary); transform: scale(1.1); }

  /* Hero */
  .hero {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4rem 2rem;
    max-width: 1200px;
    margin: 0 auto;
    position: relative;
    z-index: 1;
    gap: 3rem;
    flex-wrap: wrap;
  }
  .hero-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 1rem;
    border-radius: 20px;
    background: rgba(16, 185, 129, 0.1);
    border: 1px solid rgba(16, 185, 129, 0.3);
    color: var(--accent);
    font-size: 0.85rem;
    font-weight: 500;
    margin-bottom: 1.5rem;
  }
  .pulse-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 2s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.4); }
    50% { opacity: 0.5; box-shadow: 0 0 0 8px rgba(16, 185, 129, 0); }
  }
  .hero-title {
    font-size: clamp(2.5rem, 6vw, 4.5rem);
    font-weight: 900;
    line-height: 1.1;
    letter-spacing: -0.03em;
    margin-bottom: 1rem;
  }
  .gradient-text {
    display: block;
    background: linear-gradient(135deg, var(--primary), #fbbf24);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .gradient-text-alt {
    display: block;
    background: linear-gradient(135deg, var(--accent), #34d399);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .hero-subtitle {
    font-size: 1.2rem;
    color: var(--muted);
    margin-bottom: 2rem;
  }
  .hero-actions { display: flex; gap: 1rem; }
  .btn {
    padding: 0.8rem 2rem;
    border-radius: 12px;
    text-decoration: none;
    font-weight: 600;
    font-size: 1rem;
    transition: all 0.3s;
    cursor: pointer;
    border: none;
  }
  .btn-primary {
    background: linear-gradient(135deg, var(--primary), #fbbf24);
    color: #0a0a0f;
    box-shadow: 0 4px 20px rgba(245, 158, 11, 0.3);
  }
  .btn-primary:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 30px rgba(245, 158, 11, 0.5);
  }
  .btn-secondary {
    background: rgba(255, 255, 255, 0.05);
    color: var(--text);
    border: 1px solid var(--border);
  }
  .btn-secondary:hover { border-color: var(--primary); color: var(--primary); }

  /* Hero card */
  .hero-card {
    position: relative;
    background: var(--surface);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 2rem;
    min-width: 280px;
    box-shadow: 0 8px 40px rgba(0, 0, 0, 0.4);
  }
  .card-glow {
    position: absolute;
    inset: -2px;
    border-radius: 22px;
    background: linear-gradient(135deg, var(--primary), var(--accent));
    opacity: 0.15;
    z-index: -1;
    filter: blur(20px);
  }
  .stat-row {
    display: flex;
    justify-content: space-between;
    padding: 0.6rem 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
  .stat-row:last-child { border-bottom: none; }
  .stat-label { color: var(--muted); font-size: 0.9rem; }
  .stat-value { font-weight: 700; color: var(--primary); font-family: monospace; }
  .stat-value.danger { color: var(--danger); }

  /* Section */
  .section-title {
    text-align: center;
    font-size: 2rem;
    font-weight: 800;
    margin-bottom: 2rem;
    background: linear-gradient(135deg, var(--text), var(--muted));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  /* Features */
  .features {
    padding: 4rem 2rem;
    max-width: 1200px;
    margin: 0 auto;
    position: relative;
    z-index: 1;
  }
  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }
  .feature-card {
    background: var(--surface);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 1.75rem;
    transition: all 0.3s;
    opacity: 0;
    transform: translateY(20px);
    animation: card-in 0.6s forwards;
    animation-delay: var(--delay);
  }
  @keyframes card-in {
    to { opacity: 1; transform: translateY(0); }
  }
  .feature-card:hover {
    border-color: var(--primary);
    transform: translateY(-4px);
    box-shadow: 0 12px 40px rgba(245, 158, 11, 0.15);
  }
  .feature-icon { font-size: 2.5rem; margin-bottom: 0.75rem; }
  .feature-card h3 {
    color: var(--primary);
    font-size: 1.2rem;
    margin-bottom: 0.5rem;
  }
  .feature-card p {
    color: var(--muted);
    font-size: 0.9rem;
    line-height: 1.6;
    margin-bottom: 1rem;
  }
  .feature-tags { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .tag {
    padding: 0.2rem 0.6rem;
    border-radius: 6px;
    background: rgba(245, 158, 11, 0.08);
    color: var(--primary);
    font-size: 0.75rem;
    font-weight: 500;
  }

  /* Quantum & Stats sections */
  .quantum-section, .stats-section {
    padding: 4rem 2rem;
    max-width: 1200px;
    margin: 0 auto;
    position: relative;
    z-index: 1;
  }

  /* Footer */
  .footer {
    text-align: center;
    padding: 3rem 2rem;
    border-top: 1px solid var(--border);
    margin-top: 4rem;
    position: relative;
    z-index: 1;
  }
  .footer-content p { color: var(--muted); margin-bottom: 0.5rem; }
  .footer-links { display: flex; justify-content: center; gap: 1.5rem; }
  .footer-links a { color: var(--primary); text-decoration: none; font-size: 0.9rem; }

  /* Responsive */
  @media (max-width: 768px) {
    .nav { padding: 1rem; flex-wrap: wrap; gap: 0.5rem; }
    .nav-links { gap: 0.75rem; font-size: 0.85rem; }
    .hero { padding: 2rem 1rem; flex-direction: column; text-align: center; }
    .hero-actions { justify-content: center; }
    .hero-card { width: 100%; }
  }
</style>
