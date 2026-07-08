<section>
  <h2>NAWA Bootstrap State</h2>
  <p>App name: <code>{nawa.appName}</code></p>
  <p>WebSocket URL: <code>{nawa.wsUrl}</code></p>
  <p>Auth token present: <code>{nawa.authToken ? 'yes' : 'no'}</code></p>
  <p>Polling: <code>{nawa.polling ? 'yes (BAD)' : 'no (good — pure push)'}</code></p>
  <p>DB keys in initial state: <code>{nawa.initialState?.db_keys?.length ?? 0}</code></p>

  <h3>Live Notifications</h3>
  <ul>
    {#each notifications as notif, i}
      <li><strong>{notif.event}</strong>: {JSON.stringify(notif.data)}</li>
    {/each}
  </ul>
</section>

<script>
  // Read the NAWA bootstrap object injected by nawad.
  let nawa = window.__NAWA__ ?? {
    appName: 'unknown',
    wsUrl: '',
    authToken: null,
    polling: true,
    initialState: {}
  };

  let notifications = [];

  // Listen for live notifications via WebSocket (no polling!).
  if (nawa.wsUrl) {
    const ws = new WebSocket(nawa.wsUrl);
    ws.onmessage = (ev) => {
      try {
        const data = JSON.parse(ev.data);
        notifications = [data, ...notifications].slice(0, 10);
      } catch (e) {}
    };
  }

  // Also listen for the custom event the bootstrap script dispatches.
  window.addEventListener('nawa:notification', (e) => {
    notifications = [e.detail, ...notifications].slice(0, 10);
  });
</script>

<style>
  section {
    margin-top: 2rem;
    padding: 1.5rem;
    background: #1a1a1a;
    border: 1px solid #2a2a2a;
    border-radius: 12px;
  }
  h2, h3 {
    color: #f59e0b;
    margin-bottom: 0.75rem;
  }
  p {
    color: #888;
    line-height: 1.8;
  }
  code {
    background: #0d0c0a;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    color: #10b981;
    font-family: monospace;
  }
  ul {
    list-style: none;
    padding: 0;
  }
  li {
    padding: 0.5rem;
    background: #0d0c0a;
    border-radius: 6px;
    margin-bottom: 0.5rem;
    font-size: 0.85rem;
  }
</style>
