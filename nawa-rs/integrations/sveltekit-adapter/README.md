# adapter-nawa

**SvelteKit adapter for NAWA Web Operating System**

Embeds a SvelteKit application inside a NAWA Rust binary.
**No Node.js required at runtime** — SvelteKit is compiled at build time, and the
output is served directly by the NAWA daemon (`nawad`).

## Why?

NAWA is a Rust-only web OS. It ships as a single binary, runs on 512 MB RAM,
and has zero external runtime dependencies. SvelteKit normally needs Node.js in
production — `adapter-nawa` eliminates that requirement by:

1. **Pre-rendering** all static pages at build time (HTML + CSS + JS bundles).
2. **Generating a `manifest.json`** describing every route.
3. Letting `nawad` serve the pre-rendered output via its built-in HTTP server.
4. Hydrating pages client-side with Svelte's normal hydration flow.

## Install

```bash
npm install --save-dev adapter-nawa
```

## Usage

In `svelte.config.js`:

```js
import adapter from 'adapter-nawa';

export default {
  kit: {
    adapter: adapter({
      outDir: '_nawa',        // default: '_nawa'
      prerender: true,         // default: true — pre-render all static pages
      fallback: 'spa.html',   // default: 'spa.html' — SPA fallback page
      pages: 'pages',          // default: 'pages'
      assets: 'assets',        // default: 'assets'
    })
  }
};
```

Then build:

```bash
npm run build
# → produces _nawa/ directory containing manifest.json, pages/, assets/
```

Copy `_nawa/` next to your `nawad` binary and start the server:

```bash
nawad serve --svelte-dir ./_nawa
# nawad reads _nawa/manifest.json, serves routes, hydrates on the client.
```

## Output structure

```
_nawa/
├── manifest.json          # Route table + metadata (parsed by nawa-svelte crate)
├── pages/
│   ├── index.html         # Pre-rendered HTML for "/"
│   ├── about.html         # Pre-rendered HTML for "/about"
│   └── spa.html           # SPA fallback (no route matched)
├── assets/
│   ├── app.js             # Main entry bundle
│   ├── app.css            # Global CSS
│   ├── index.js           # Hydration chunk for "/"
│   └── users.[id].js      # Hydration chunk for "/users/[id]"
└── ssr/                   # (future) WASM modules for server-side rendering
```

## How it works

```
┌────────────── Build-time (Node.js) ──────────────┐
│  SvelteKit → adapter-nawa → _nawa/ directory    │
└────────────────────────┬───────────────────────────┘
                         ↓ embed in binary / deploy
┌────────────── Runtime (Rust binary) ────────────┐
│  nawad:                                          │
│  • reads _nawa/manifest.json                     │
│  • matches incoming request → route              │
│  • serves pre-rendered HTML (zero-copy)          │
│  • injects window.__NAWA__ bootstrap             │
│  • Svelte hydrates client-side                   │
│  • WebSocket pushes live updates                 │
└──────────────────────────────────────────────────┘
```

## NAWA bootstrap

Every page rendered by NAWA gets a `window.__NAWA__` object injected:

```js
window.__NAWA__ = {
  appName: 'MyApp',
  wsUrl: 'ws://localhost:8081',
  authToken: '<jwt>',
  csrfToken: '<hex>',
  route: { pattern: '/users/[id]', params: { id: '42' }, query: {} },
  user: { username: 'admin', role: 'admin' },
  initialState: { /* DB state */ },
  transport: 'websocket-push',
  polling: false,
};
```

In your Svelte components:

```svelte
<script>
  // Access NAWA-provided state.
  let { user, initialState } = window.__NAWA__;

  // Listen for live updates via WebSocket.
  window.addEventListener('nawa:notification', (e) => {
    console.log('Live update:', e.detail);
  });
</script>
```

## License

MIT OR Apache-2.0
