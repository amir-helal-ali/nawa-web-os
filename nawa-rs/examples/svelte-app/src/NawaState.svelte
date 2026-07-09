<section class="nawa-state">
  <h3>🌐 حالة NAWA</h3>
  <div class="state-grid">
    <div class="state-item">
      <span class="state-label">اسم التطبيق</span>
      <span class="state-value">{nawa.appName || 'غير معروف'}</span>
    </div>
    <div class="state-item">
      <span class="state-label">WebSocket</span>
      <span class="state-value ws-status {wsConnected ? 'connected' : 'disconnected'}">
        {wsConnected ? '🟢 متصل' : '🔴 غير متصل'}
      </span>
    </div>
    <div class="state-item">
      <span class="state-label">Auth Token</span>
      <span class="state-value">{nawa.authToken ? '✓ موجود' : '✗ غير موجود'}</span>
    </div>
    <div class="state-item">
      <span class="state-label">Polling</span>
      <span class="state-value danger">{nawa.polling ? '⚠ مفعّل' : '✓ معطّل (push فقط)'}</span>
    </div>
  </div>

  {#if notifications.length > 0}
    <div class="notifications">
      <h4>📢 إشعارات لحظية</h4>
      {#each notifications as notif}
        <div class="notif-item">
          <span class="notif-event">{notif.event}</span>
          <span class="notif-data">{JSON.stringify(notif.data)}</span>
        </div>
      {/each}
    </div>
  {/if}
</section>

<script>
  import { onMount } from 'svelte';

  let nawa = window.__NAWA__ || {};
  let wsConnected = false;
  let notifications = [];

  onMount(() => {
    if (nawa.wsUrl) {
      const ws = new WebSocket(nawa.wsUrl);
      ws.onopen = () => { wsConnected = true; };
      ws.onclose = () => { wsConnected = false; };
      ws.onmessage = (ev) => {
        try {
          const data = JSON.parse(ev.data);
          notifications = [data, ...notifications].slice(0, 5);
        } catch (e) {}
      };
    }

    window.addEventListener('nawa:notification', (e) => {
      notifications = [e.detail, ...notifications].slice(0, 5);
    });
  });
</script>

<style>
  .nawa-state {
    margin-top: 2rem;
    padding: 1.5rem;
    background: rgba(20, 20, 30, 0.6);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(245, 158, 11, 0.15);
    border-radius: 16px;
  }
  .nawa-state h3 {
    color: #f59e0b;
    margin-bottom: 1rem;
    font-size: 1.1rem;
  }
  .state-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 0.75rem;
  }
  .state-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
  }
  .state-label { color: #8b8b9a; font-size: 0.8rem; }
  .state-value { color: #e8e8ef; font-family: monospace; font-size: 0.85rem; }
  .state-value.connected { color: #10b981; }
  .state-value.disconnected { color: #ef4444; }
  .state-value.danger { color: #ef4444; }

  .notifications { margin-top: 1.5rem; }
  .notifications h4 { color: #f59e0b; margin-bottom: 0.5rem; font-size: 0.95rem; }
  .notif-item {
    display: flex;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 6px;
    margin-bottom: 0.4rem;
    font-size: 0.85rem;
    animation: slide-in 0.3s ease;
  }
  @keyframes slide-in {
    from { opacity: 0; transform: translateY(-10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .notif-event { color: #f59e0b; font-weight: 600; min-width: 120px; }
  .notif-data { color: #8b8b9a; flex: 1; word-break: break-all; }
</style>
