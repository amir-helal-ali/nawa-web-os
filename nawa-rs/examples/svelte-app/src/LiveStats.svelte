<script>
  import { onMount } from 'svelte';

  let stats = {
    endpoints: 87,
    modules: 26,
    tests: 530,
    binary: '10.5MB',
    wasm: '74KB',
    wasmPlugins: 1,
    wsConnections: 0,
    dbKeys: 0,
    users: 0
  };

  let nawa = window.__NAWA__ || {};

  onMount(() => {
    // Update from bootstrap
    if (nawa.initialState) {
      stats.dbKeys = nawa.initialState.db_size || 0;
    }

    // Try to fetch live stats
    fetch('/api/health')
      .then(r => r.json())
      .then(d => {
        if (d.checks) stats.wasmPlugins = 1;
      })
      .catch(() => {});

    // Listen for WebSocket notifications
    window.addEventListener('nawa:notification', (e) => {
      if (e.detail.event === 'db_write') stats.dbKeys++;
      if (e.detail.event === 'user_registered') stats.users++;
    });
  });
</script>

<div class="live-stats">
  <div class="stats-grid">
    <div class="stat-card">
      <div class="stat-icon">🔌</div>
      <div class="stat-num">{stats.endpoints}</div>
      <div class="stat-name">Endpoints</div>
    </div>
    <div class="stat-card">
      <div class="stat-icon">📦</div>
      <div class="stat-num">{stats.modules}</div>
      <div class="stat-name">Modules</div>
    </div>
    <div class="stat-card">
      <div class="stat-icon">🧪</div>
      <div class="stat-num">{stats.tests}+</div>
      <div class="stat-name">Tests</div>
    </div>
    <div class="stat-card">
      <div class="stat-icon">💾</div>
      <div class="stat-num">{stats.binary}</div>
      <div class="stat-name">Binary</div>
    </div>
    <div class="stat-card highlight">
      <div class="stat-icon">⚛️</div>
      <div class="stat-num">{stats.wasm}</div>
      <div class="stat-name">WASM Module</div>
    </div>
    <div class="stat-card highlight">
      <div class="stat-icon">🔑</div>
      <div class="stat-num">{stats.dbKeys}</div>
      <div class="stat-name">DB Keys (live)</div>
    </div>
  </div>
</div>

<style>
  .live-stats { margin-top: 1rem; }
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
  }
  .stat-card {
    background: rgba(20, 20, 30, 0.6);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(245, 158, 11, 0.15);
    border-radius: 12px;
    padding: 1.25rem;
    text-align: center;
    transition: all 0.3s;
  }
  .stat-card:hover {
    border-color: rgba(245, 158, 11, 0.4);
    transform: translateY(-3px);
    box-shadow: 0 8px 24px rgba(245, 158, 11, 0.1);
  }
  .stat-card.highlight {
    border-color: rgba(16, 185, 129, 0.3);
    background: rgba(16, 185, 129, 0.05);
  }
  .stat-icon { font-size: 1.5rem; margin-bottom: 0.5rem; }
  .stat-num {
    font-size: 1.75rem;
    font-weight: 800;
    color: #f59e0b;
    font-family: 'JetBrains Mono', monospace;
  }
  .stat-card.highlight .stat-num { color: #10b981; }
  .stat-name {
    color: #8b8b9a;
    font-size: 0.8rem;
    margin-top: 0.25rem;
  }
</style>
