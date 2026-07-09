<script>
  import { onMount } from 'svelte';
  import Counter from './Counter.svelte';
  import NawaState from './NawaState.svelte';
  import LiveStats from './LiveStats.svelte';
  import QuantumViz from './QuantumViz.svelte';

  let loaded = false;
  let mouseX = 0;
  let mouseY = 0;
  let currentView = 'home'; // home | admin | api | aion
  let user = null;
  let adminData = { users: [], metrics: {}, audit: [] };
  let aionData = { entities: 0, relationships: 0, photon: '', sitemap: '' };

  const nawa = window.__NAWA__ || {};

  onMount(() => {
    loaded = true;
    window.addEventListener('mousemove', (e) => {
      mouseX = e.clientX;
      mouseY = e.clientY;
    });

    // Check current user from bootstrap
    if (nawa.initialState && nawa.initialState.auth && nawa.initialState.auth.logged_in) {
      // Try to fetch the user profile
      fetch('/profile', { headers: { 'Accept': 'application/json' } })
        .then(r => r.ok ? r.text() : null)
        .then(() => {
          // We can't easily parse profile HTML; just mark logged in
          user = { loggedIn: true };
        })
        .catch(() => {});
    }

    // Live notifications — pure push, no polling
    window.addEventListener('nawa:notification', (e) => {
      const n = e.detail;
      if (n.event === 'user_registered' && currentView === 'admin') loadAdmin();
    });
  });

  async function loadAdmin() {
    try {
      const [usersRes, metricsRes, auditRes] = await Promise.all([
        fetch('/api/admin/users').then(r => r.json()),
        fetch('/metrics').then(r => r.json()),
        fetch('/api/admin/audit?limit=20').then(r => r.json()),
      ]);
      adminData = { users: usersRes.users || [], metrics: metricsRes || {}, audit: auditRes.events || [] };
    } catch (e) {
      adminData = { users: [], metrics: { error: 'غير مصرح' }, audit: [] };
    }
  }

  async function loadAion() {
    try {
      const [photonRes, sitemapRes, kgRes] = await Promise.all([
        fetch('/__photon__').then(r => r.json()),
        fetch('/sitemap.xml').then(r => r.text()),
        fetch('/api/aion/health').then(r => r.json()),
      ]);
      aionData = {
        entities: kgRes.knowledge_graph?.entities || 0,
        relationships: kgRes.knowledge_graph?.relationships || 0,
        photon: JSON.stringify(photonRes, null, 2).slice(0, 600),
        sitemap: sitemapRes.slice(0, 400),
      };
    } catch (e) {
      aionData = { entities: 0, relationships: 0, photon: '', sitemap: '' };
    }
  }

  function switchView(view) {
    currentView = view;
    if (view === 'admin') loadAdmin();
    if (view === 'aion') loadAion();
  }

  function logout() {
    window.location.href = '/logout';
  }
</script>

<div class="app" class:loaded>
  <!-- Animated background grid -->
  <div class="grid-bg" style="--mx: {mouseX}px; --my: {mouseY}px;"></div>

  <!-- Top navigation -->
  <nav class="nav">
    <div class="nav-brand">
      <span class="logo">🦀</span>
      <span class="brand-text">NAWA</span>
      <span class="version-badge">v2.4.0</span>
    </div>
    <div class="nav-links">
      <button class="nav-link" class:active={currentView === 'home'} on:click={() => switchView('home')}>الرئيسية</button>
      <button class="nav-link" class:active={currentView === 'aion'} on:click={() => switchView('aion')}>AION</button>
      <button class="nav-link" class:active={currentView === 'admin'} on:click={() => switchView('admin')}>الإدارة</button>
      <button class="nav-link" class:active={currentView === 'api'} on:click={() => switchView('api')}>API</button>
      <a href="/docs" target="_blank" class="nav-link">التوثيق</a>
    </div>
    <div class="nav-actions">
      {#if user && user.loggedIn}
        <a href="/profile" class="user-chip">👤 حسابي</a>
        <button class="btn-logout" on:click={logout}>خروج</button>
      {:else}
        <a href="/login" class="btn-login">دخول</a>
        <a href="/register" class="btn-register">ابدأ الآن →</a>
      {/if}
      <button class="theme-toggle" on:click={() => window.nawaToggleTheme?.()}>🌙</button>
    </div>
  </nav>

  <!-- ═══ HOME VIEW ═══ -->
  {#if currentView === 'home'}
    <!-- Hero section -->
    <section class="hero">
      <div class="hero-content">
        <div class="hero-badge">
          <span class="pulse-dot"></span>
          Quantum-Powered Web OS · v2.4.0
        </div>
        <h1 class="hero-title">
          <span class="gradient-text">نظام تشغيل الويب</span>
          <span class="gradient-text-alt">الثوري</span>
        </h1>
        <p class="hero-subtitle">
          Binary واحد خالص · بدون Node.js · ميكانيكا كمية · صفر نسخ · 87 endpoint · 26 module
        </p>
        <div class="hero-actions">
          <a href="/register" class="btn btn-primary">ابدأ الآن →</a>
          <button class="btn btn-secondary" on:click={() => switchView('aion')}>🌐 AION Engine</button>
        </div>
      </div>

      <!-- Floating stats card -->
      <div class="hero-card">
        <div class="card-glow"></div>
        <div class="stat-row"><span class="stat-label">Endpoints</span><span class="stat-value">87</span></div>
        <div class="stat-row"><span class="stat-label">Modules</span><span class="stat-value">26</span></div>
        <div class="stat-row"><span class="stat-label">Tests</span><span class="stat-value">530+</span></div>
        <div class="stat-row"><span class="stat-label">Binary</span><span class="stat-value">10.5MB</span></div>
        <div class="stat-row"><span class="stat-label">Node.js</span><span class="stat-value danger">غير مطلوب</span></div>
        <div class="stat-row"><span class="stat-label">Polling</span><span class="stat-value danger">صفر</span></div>
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
            <span class="tag">Superposition</span><span class="tag">Entanglement</span><span class="tag">Tunneling</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 100ms">
          <div class="feature-icon">🌐</div>
          <h3>AION SEO Engine</h3>
          <p>Knowledge Graph تلقائي + 9 صيغ استجابة + Photon Protocol + Self-Healing</p>
          <div class="feature-tags">
            <span class="tag">Knowledge Graph</span><span class="tag">Photon</span><span class="tag">QEC</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 200ms">
          <div class="feature-icon">⚡</div>
          <h3>WASM SSR</h3>
          <p>Rust → WASM → HTML بدون Node.js — 74KB module فقط</p>
          <div class="feature-tags">
            <span class="tag">wasmtime</span><span class="tag">Zero-copy</span><span class="tag">Sandbox</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 300ms">
          <div class="feature-icon">🔌</div>
          <h3>WebSocket Pub/Sub</h3>
          <p>إشعارات لحظية بـ 6 قنوات — صفر polling، 100% push</p>
          <div class="feature-tags">
            <span class="tag">Real-time</span><span class="tag">6 Channels</span><span class="tag">Event Bus</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 400ms">
          <div class="feature-icon">🛡️</div>
          <h3>أمان عالي</h3>
          <p>11 security headers + CSRF + Audit log + Rate limiting + Session management</p>
          <div class="feature-tags">
            <span class="tag">CSP</span><span class="tag">HSTS</span><span class="tag">RBAC</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 500ms">
          <div class="feature-icon">🎨</div>
          <h3>تصميم احترافي</h3>
          <p>3 ثيمات (Dark/Light/Auto) + Glassmorphism + Animations + RTL</p>
          <div class="feature-tags">
            <span class="tag">3 Themes</span><span class="tag">Glassmorphism</span><span class="tag">RTL</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 600ms">
          <div class="feature-icon">🗄️</div>
          <h3>NAWA-DB مدمج</h3>
          <p>LSM-Tree + WAL + Bloom Filter — 4.3M reads/sec، 713K writes/sec</p>
          <div class="feature-tags">
            <span class="tag">LSM</span><span class="tag">WAL</span><span class="tag">Bloom</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 700ms">
          <div class="feature-icon">📡</div>
          <h3>HTTP/3 + QUIC</h3>
          <p>دعم HTTP/3 عبر QUIC اختياري مع TLS — مستقبلي وجاهز للويب القادم</p>
          <div class="feature-tags">
            <span class="tag">QUIC</span><span class="tag">TLS</span><span class="tag">HTTP/3</span>
          </div>
        </div>

        <div class="feature-card" style="--delay: 800ms">
          <div class="feature-icon">📦</div>
          <h3>Plugin System</h3>
          <p>حمّل WASM plugins في الـ sandbox مع fuel limits — آمن تماماً</p>
          <div class="feature-tags">
            <span class="tag">WASM</span><span class="tag">Sandbox</span><span class="tag">Fuel</span>
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
  {/if}

  <!-- ═══ AION VIEW ═══ -->
  {#if currentView === 'aion'}
    <section class="view-section">
      <h2 class="section-title">🌐 AION SEO Engine</h2>
      <p class="view-subtitle">Adaptive Intelligent Ontological Network — محرك SEO ثوري</p>

      <div class="aion-grid">
        <div class="aion-card">
          <h3>📊 Knowledge Graph</h3>
          <div class="kg-stats">
            <div class="kg-stat"><span class="kg-num">{aionData.entities}</span><span class="kg-label">Entities</span></div>
            <div class="kg-stat"><span class="kg-num">{aionData.relationships}</span><span class="kg-label">Relationships</span></div>
          </div>
          <button class="btn-sm" on:click={loadAion}>↻ تحديث</button>
        </div>

        <div class="aion-card">
          <h3>⚡ Photon Protocol</h3>
          <p class="card-desc">.endpoint واحد يعيد الـ Knowledge Graph كاملاً للـ crawlers</p>
          <pre class="code-block">{aionData.photon || 'اضغط تحديث...'}</pre>
          <a href="/__photon__" target="_blank" class="btn-sm">فتح /__photon__</a>
        </div>

        <div class="aion-card">
          <h3>🗺️ Sitemap تلقائي</h3>
          <p class="card-desc">sitemap.xml يُولّد ديناميكياً من الـ Knowledge Graph</p>
          <pre class="code-block">{aionData.sitemap || 'اضغط تحديث...'}</pre>
          <a href="/sitemap.xml" target="_blank" class="btn-sm">فتح /sitemap.xml</a>
        </div>

        <div class="aion-card">
          <h3>🤖 AI Crawler Support</h3>
          <p class="card-desc">9 صيغ استجابة: HTML, JSON, JSON-LD, ActivityPub, RSS, Atom, Microdata, RDFa, Photon</p>
          <div class="feature-tags">
            <span class="tag">HTML</span><span class="tag">JSON-LD</span><span class="tag">ActivityPub</span><span class="tag">RSS</span><span class="tag">Atom</span><span class="tag">Photon</span>
          </div>
          <a href="/robots.txt" target="_blank" class="btn-sm">robots.txt</a>
        </div>
      </div>
    </section>
  {/if}

  <!-- ═══ ADMIN VIEW ═══ -->
  {#if currentView === 'admin'}
    <section class="view-section">
      <h2 class="section-title">🛡️ لوحة الإدارة</h2>
      <p class="view-subtitle">إدارة المستخدمين، المراقبة، وسجل التدقيق — يتطلب صلاحية admin</p>

      {#if !user || !user.loggedIn}
        <div class="empty-state">
          <div class="empty-icon">🔒</div>
          <h3>تسجيل الدخول مطلوب</h3>
          <p>الوصول للوحة الإدارة يتطلب تسجيل الدخول بصلاحية admin</p>
          <a href="/login" class="btn btn-primary">تسجيل الدخول</a>
        </div>
      {:else}
        <div class="admin-grid">
          <!-- Users panel -->
          <div class="admin-card">
            <div class="admin-header">
              <h3>👥 المستخدمون</h3>
              <button class="btn-sm" on:click={loadAdmin}>↻</button>
            </div>
            {#if adminData.users.length === 0}
              <p class="muted">لا يوجد مستخدمون (تحقق من صلاحية admin)</p>
            {:else}
              <div class="users-list">
                {#each adminData.users as u}
                  <div class="user-row">
                    <span class="user-avatar">{u.username?.[0]?.toUpperCase() || 'U'}</span>
                    <div class="user-info">
                      <div class="user-name">{u.username}</div>
                      <div class="user-email">{u.email}</div>
                    </div>
                    <span class="badge" class:admin={u.role === 'admin'}>{u.role}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Metrics panel -->
          <div class="admin-card">
            <div class="admin-header">
              <h3>📈 المراقبة</h3>
              <a href="/metrics" target="_blank" class="btn-sm">JSON</a>
            </div>
            {#if adminData.metrics.error}
              <p class="muted">⚠ {adminData.metrics.error}</p>
            {:else}
              <div class="metrics-list">
                {#each Object.entries(adminData.metrics) as [k, v]}
                  <div class="metric-row">
                    <span class="metric-key">{k}</span>
                    <span class="metric-val">{typeof v === 'object' ? JSON.stringify(v) : v}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Audit panel -->
          <div class="admin-card admin-card-wide">
            <div class="admin-header">
              <h3>📜 سجل التدقيق</h3>
              <span class="badge">{adminData.audit.length}</span>
            </div>
            {#if adminData.audit.length === 0}
              <p class="muted">لا توجد أحداث في سجل التدقيق</p>
            {:else}
              <div class="audit-list">
                {#each adminData.audit as ev}
                  <div class="audit-row">
                    <span class="audit-time">{ev.timestamp || ''}</span>
                    <span class="audit-action">{ev.action || ev.event || ''}</span>
                    <span class="audit-user">{ev.username || ev.user_id || ''}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Quick actions -->
        <div class="quick-actions">
          <h3>⚡ إجراءات سريعة</h3>
          <div class="actions-row">
            <a href="/api/admin/users" target="_blank" class="action-btn">GET /api/admin/users</a>
            <a href="/metrics" target="_blank" class="action-btn">GET /metrics</a>
            <a href="/api/admin/audit" target="_blank" class="action-btn">GET /api/admin/audit</a>
            <a href="/api/health" target="_blank" class="action-btn">GET /api/health</a>
            <a href="/__photon__" target="_blank" class="action-btn">GET /__photon__</a>
            <a href="/system" target="_blank" class="action-btn">GET /system</a>
          </div>
        </div>
      {/if}
    </section>
  {/if}

  <!-- ═══ API VIEW ═══ -->
  {#if currentView === 'api'}
    <section class="view-section">
      <h2 class="section-title">🔌 API Reference</h2>
      <p class="view-subtitle">87 endpoint موزّعة عبر 26 module — كلها في binary واحد</p>

      <div class="api-grid">
        <div class="api-card">
          <h3>🔐 المصادقة</h3>
          <ul class="api-list">
            <li><span class="m-get">GET</span> /register</li>
            <li><span class="m-post">POST</span> /register</li>
            <li><span class="m-get">GET</span> /login</li>
            <li><span class="m-post">POST</span> /login</li>
            <li><span class="m-get">GET</span> /logout</li>
            <li><span class="m-get">GET</span> /profile</li>
            <li><span class="m-post">POST</span> /profile</li>
          </ul>
        </div>

        <div class="api-card">
          <h3>🌐 AION SEO</h3>
          <ul class="api-list">
            <li><span class="m-get">GET</span> /__photon__</li>
            <li><span class="m-get">GET</span> /sitemap.xml</li>
            <li><span class="m-get">GET</span> /robots.txt</li>
            <li><span class="m-get">GET</span> /api/aion/health</li>
            <li><span class="m-get">GET</span> /api/aion/kg</li>
            <li><span class="m-post">POST</span> /api/aion/heal</li>
          </ul>
        </div>

        <div class="api-card">
          <h3>⚛️ الكوانتم</h3>
          <ul class="api-list">
            <li><span class="m-get">GET</span> /api/quantum/state</li>
            <li><span class="m-post">POST</span> /api/quantum/measure</li>
            <li><span class="m-post">POST</span> /api/quantum/superpose</li>
            <li><span class="m-post">POST</span> /api/quantum/entangle</li>
            <li><span class="m-post">POST</span> /api/quantum/tunnel</li>
            <li><span class="m-get">GET</span> /api/quantum/qec</li>
          </ul>
        </div>

        <div class="api-card">
          <h3>🗄️ البيانات</h3>
          <ul class="api-list">
            <li><span class="m-get">GET</span> /api/data/:key</li>
            <li><span class="m-post">POST</span> /api/data/:key</li>
            <li><span class="m-del">DEL</span> /api/data/:key</li>
            <li><span class="m-get">GET</span> /api/data?prefix=</li>
            <li><span class="m-get">GET</span> /api/data/export</li>
          </ul>
        </div>

        <div class="api-card">
          <h3>🔌 Realtime</h3>
          <ul class="api-list">
            <li><span class="m-ws">WS</span> ws://host:port+1</li>
            <li><span class="m-post">POST</span> /api/publish</li>
            <li><span class="m-get">GET</span> /api/channels</li>
            <li><span class="m-get">GET</span> /api/connections</li>
          </ul>
        </div>

        <div class="api-card">
          <h3>🛡️ الإدارة</h3>
          <ul class="api-list">
            <li><span class="m-get">GET</span> /api/admin/users</li>
            <li><span class="m-get">GET</span> /api/admin/audit</li>
            <li><span class="m-get">GET</span> /metrics</li>
            <li><span class="m-get">GET</span> /system</li>
            <li><span class="m-get">GET</span> /api/health</li>
          </ul>
        </div>

        <div class="api-card">
          <h3>📦 WASM SSR</h3>
          <ul class="api-list">
            <li><span class="m-post">POST</span> /api/wasm-ssr</li>
            <li><span class="m-get">GET</span> /api/plugins</li>
            <li><span class="m-post">POST</span> /api/plugins/:name/reload</li>
          </ul>
        </div>

        <div class="api-card">
          <h3>⚙️ النظام</h3>
          <ul class="api-list">
            <li><span class="m-get">GET</span> /</li>
            <li><span class="m-get">GET</span> /docs</li>
            <li><span class="m-get">GET</span> /openapi.json</li>
            <li><span class="m-get">GET</span> /api/feature-flags</li>
            <li><span class="m-get">GET</span> /api/scheduler/jobs</li>
          </ul>
        </div>
      </div>
    </section>
  {/if}

  <!-- Footer -->
  <footer class="footer">
    <div class="footer-content">
      <p>🦀 NAWA Web Operating System v2.4.0 — Revolutionary</p>
      <div class="footer-links">
        <a href="https://github.com/amir-helal-ali/nawa-web-os">GitHub</a>
        <a href="/docs">Docs</a>
        <a href="/openapi.json">OpenAPI</a>
        <a href="/__photon__">Photon</a>
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

  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 2rem;
    position: sticky;
    top: 0;
    z-index: 100;
    background: rgba(10, 10, 15, 0.85);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
    gap: 1rem;
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
  .nav-links { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
  .nav-link {
    color: var(--muted);
    text-decoration: none;
    font-size: 0.9rem;
    transition: color 0.2s;
    background: none;
    border: none;
    cursor: pointer;
    font-family: inherit;
    padding: 0.4rem 0.8rem;
    border-radius: 8px;
  }
  .nav-link:hover { color: var(--primary); background: rgba(245,158,11,0.05); }
  .nav-link.active { color: var(--primary); background: rgba(245,158,11,0.1); }
  .nav-actions { display: flex; gap: 0.5rem; align-items: center; }
  .btn-login {
    padding: 0.4rem 1rem;
    border-radius: 8px;
    background: rgba(255,255,255,0.05);
    color: var(--text);
    text-decoration: none;
    font-size: 0.85rem;
    border: 1px solid var(--border);
    transition: all 0.2s;
  }
  .btn-login:hover { border-color: var(--primary); color: var(--primary); }
  .btn-register {
    padding: 0.4rem 1rem;
    border-radius: 8px;
    background: linear-gradient(135deg, var(--primary), #fbbf24);
    color: #0a0a0f;
    text-decoration: none;
    font-size: 0.85rem;
    font-weight: 600;
    transition: all 0.2s;
  }
  .btn-register:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(245,158,11,0.4); }
  .user-chip {
    padding: 0.4rem 0.8rem;
    border-radius: 8px;
    background: rgba(16, 185, 129, 0.1);
    border: 1px solid rgba(16, 185, 129, 0.3);
    color: var(--accent);
    text-decoration: none;
    font-size: 0.85rem;
  }
  .btn-logout {
    padding: 0.4rem 0.8rem;
    border-radius: 8px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: var(--danger);
    font-size: 0.85rem;
    cursor: pointer;
    font-family: inherit;
  }
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
    font-size: 1.15rem;
    color: var(--muted);
    margin-bottom: 2rem;
    line-height: 1.7;
  }
  .hero-actions { display: flex; gap: 1rem; flex-wrap: wrap; }
  .btn {
    padding: 0.8rem 2rem;
    border-radius: 12px;
    text-decoration: none;
    font-weight: 600;
    font-size: 1rem;
    transition: all 0.3s;
    cursor: pointer;
    border: none;
    font-family: inherit;
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

  .section-title {
    text-align: center;
    font-size: 2rem;
    font-weight: 800;
    margin-bottom: 1rem;
    background: linear-gradient(135deg, var(--text), var(--muted));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .features, .quantum-section, .stats-section, .view-section {
    padding: 4rem 2rem;
    max-width: 1200px;
    margin: 0 auto;
    position: relative;
    z-index: 1;
  }
  .view-subtitle {
    text-align: center;
    color: var(--muted);
    margin-bottom: 2rem;
    font-size: 1rem;
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
  .feature-card h3 { color: var(--primary); font-size: 1.2rem; margin-bottom: 0.5rem; }
  .feature-card p { color: var(--muted); font-size: 0.9rem; line-height: 1.6; margin-bottom: 1rem; }
  .feature-tags { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .tag {
    padding: 0.2rem 0.6rem;
    border-radius: 6px;
    background: rgba(245, 158, 11, 0.08);
    color: var(--primary);
    font-size: 0.75rem;
    font-weight: 500;
  }

  /* AION view */
  .aion-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 1.5rem;
  }
  .aion-card {
    background: var(--surface);
    backdrop-filter: blur(12px);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 1.5rem;
  }
  .aion-card h3 { color: var(--primary); margin-bottom: 1rem; font-size: 1.1rem; }
  .card-desc { color: var(--muted); font-size: 0.85rem; line-height: 1.6; margin-bottom: 1rem; }
  .kg-stats { display: flex; gap: 2rem; margin-bottom: 1rem; }
  .kg-stat { display: flex; flex-direction: column; }
  .kg-num { font-size: 2rem; font-weight: 800; color: var(--accent); font-family: monospace; }
  .kg-label { color: var(--muted); font-size: 0.8rem; }
  .code-block {
    background: rgba(0,0,0,0.4);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0.75rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    color: var(--accent);
    overflow-x: auto;
    max-height: 200px;
    overflow-y: auto;
    margin: 0.5rem 0 1rem;
    direction: ltr;
    text-align: left;
  }
  .btn-sm {
    display: inline-block;
    padding: 0.3rem 0.8rem;
    border-radius: 6px;
    background: rgba(245, 158, 11, 0.1);
    border: 1px solid var(--border);
    color: var(--primary);
    font-size: 0.8rem;
    text-decoration: none;
    cursor: pointer;
    font-family: inherit;
    transition: all 0.2s;
  }
  .btn-sm:hover { background: rgba(245, 158, 11, 0.2); }

  /* Admin view */
  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    background: var(--surface);
    border-radius: 16px;
    border: 1px dashed var(--border);
  }
  .empty-icon { font-size: 4rem; margin-bottom: 1rem; }
  .empty-state h3 { color: var(--primary); margin-bottom: 0.5rem; }
  .empty-state p { color: var(--muted); margin-bottom: 1.5rem; }
  .admin-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }
  .admin-card {
    background: var(--surface);
    backdrop-filter: blur(12px);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 1.5rem;
  }
  .admin-card-wide { grid-column: 1 / -1; }
  .admin-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  .admin-header h3 { color: var(--primary); font-size: 1.1rem; }
  .users-list, .audit-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .user-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem;
    background: rgba(0,0,0,0.2);
    border-radius: 8px;
  }
  .user-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--primary), var(--accent));
    color: #0a0a0f;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
  }
  .user-info { flex: 1; }
  .user-name { font-weight: 600; }
  .user-email { color: var(--muted); font-size: 0.8rem; }
  .badge {
    padding: 0.2rem 0.6rem;
    border-radius: 6px;
    background: rgba(139, 139, 154, 0.2);
    color: var(--muted);
    font-size: 0.7rem;
    font-weight: 600;
  }
  .badge.admin { background: rgba(245, 158, 11, 0.15); color: var(--primary); }
  .metrics-list { display: flex; flex-direction: column; gap: 0.4rem; }
  .metric-row {
    display: flex;
    justify-content: space-between;
    padding: 0.4rem 0.6rem;
    background: rgba(0,0,0,0.2);
    border-radius: 6px;
    font-family: monospace;
    font-size: 0.8rem;
  }
  .metric-key { color: var(--muted); }
  .metric-val { color: var(--accent); }
  .audit-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 1rem;
    padding: 0.4rem 0.6rem;
    background: rgba(0,0,0,0.2);
    border-radius: 6px;
    font-size: 0.8rem;
    font-family: monospace;
    direction: ltr;
    text-align: left;
  }
  .audit-time { color: var(--muted); }
  .audit-action { color: var(--primary); }
  .audit-user { color: var(--accent); }
  .muted { color: var(--muted); }
  .quick-actions { margin-top: 1.5rem; }
  .quick-actions h3 { color: var(--primary); margin-bottom: 1rem; }
  .actions-row { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  .action-btn {
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    background: rgba(0,0,0,0.3);
    border: 1px solid var(--border);
    color: var(--text);
    text-decoration: none;
    font-family: monospace;
    font-size: 0.75rem;
    transition: all 0.2s;
  }
  .action-btn:hover { border-color: var(--primary); color: var(--primary); }

  /* API view */
  .api-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 1.5rem;
  }
  .api-card {
    background: var(--surface);
    backdrop-filter: blur(12px);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 1.25rem;
  }
  .api-card h3 { color: var(--primary); margin-bottom: 1rem; font-size: 1rem; }
  .api-list { list-style: none; display: flex; flex-direction: column; gap: 0.4rem; }
  .api-list li {
    padding: 0.4rem 0.6rem;
    background: rgba(0,0,0,0.2);
    border-radius: 6px;
    font-family: monospace;
    font-size: 0.75rem;
    direction: ltr;
    text-align: left;
  }
  .m-get { color: var(--accent); font-weight: 700; }
  .m-post { color: var(--primary); font-weight: 700; }
  .m-del { color: var(--danger); font-weight: 700; }
  .m-ws { color: #8b5cf6; font-weight: 700; }

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

  @media (max-width: 768px) {
    .nav { padding: 1rem; }
    .nav-links { gap: 0.25rem; }
    .nav-link { padding: 0.3rem 0.5rem; font-size: 0.8rem; }
    .hero { padding: 2rem 1rem; flex-direction: column; text-align: center; }
    .hero-actions { justify-content: center; }
    .hero-card { width: 100%; }
    .features, .quantum-section, .stats-section, .view-section { padding: 2rem 1rem; }
  }
</style>
