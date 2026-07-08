// post-build.js — generates _nawa/manifest.json after Vite build.
// Run with: node post-build.js
import fs from 'node:fs';
import path from 'node:path';

const nawaDir = path.resolve('_nawa');
const assetsDir = path.join(nawaDir, 'assets');
const pagesDir = path.join(nawaDir, 'pages');

// Ensure directories exist.
fs.mkdirSync(assetsDir, { recursive: true });
fs.mkdirSync(pagesDir, { recursive: true });

// Find the main JS bundle (Vite outputs to assets/app.js per our config).
const mainJs = fs.existsSync(path.join(assetsDir, 'app.js')) ? 'app.js' : null;

// Find the global CSS (Vite outputs to assets/app.css or similar).
let globalCss = null;
if (fs.existsSync(assetsDir)) {
  const cssFile = fs.readdirSync(assetsDir).find(f => f.endsWith('.css'));
  globalCss = cssFile ?? null;
}

// Find favicon if present.
let favicon = null;
if (fs.existsSync(assetsDir)) {
  const fav = fs.readdirSync(assetsDir).find(f => f.startsWith('favicon'));
  favicon = fav ?? null;
}

// Write a SPA fallback HTML that loads the app.
const spaFallback = `<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>NAWA + SvelteKit</title>
${globalCss ? `<link rel="stylesheet" href="/_nawa/assets/${globalCss}">` : ''}
</head>
<body>
<div id="svelte"></div>
${mainJs ? `<script type="module" src="/_nawa/assets/${mainJs}"></script>` : ''}
</body>
</html>`;
fs.writeFileSync(path.join(pagesDir, 'spa.html'), spaFallback);

// Also write an index.html that NAWA will serve as the "/" route.
fs.writeFileSync(path.join(pagesDir, 'index.html'), spaFallback);

// Build the manifest.
const manifest = {
  version: 1,
  app_name: 'NAWA SvelteKit App (Vite)',
  built_at: new Date().toISOString(),
  sveltekit_version: '5.x (Vite build)',
  routes: [
    {
      pattern: '/',
      methods: ['GET'],
      prerendered_html: 'index.html',
      hydration_js: mainJs,
      requires_auth: false,
      admin_only: false,
      meta: {
        title: 'NAWA + SvelteKit (Vite build)',
        description: 'Real SvelteKit app compiled by Vite, served by NAWA — no Node.js at runtime'
      },
      is_endpoint: false,
      ssr_wasm: null,
      layout: null
    },
    {
      pattern: '/dashboard',
      methods: ['GET'],
      prerendered_html: null,
      hydration_js: mainJs,
      requires_auth: true,
      admin_only: false,
      meta: { title: 'Dashboard — NAWA SvelteKit' },
      is_endpoint: false,
      ssr_wasm: null,
      layout: null
    },
    {
      pattern: '/users/[id]',
      methods: ['GET'],
      prerendered_html: null,
      hydration_js: mainJs,
      requires_auth: true,
      admin_only: false,
      meta: { title: 'User Profile — NAWA SvelteKit' },
      is_endpoint: false,
      ssr_wasm: null,
      layout: null
    },
    {
      pattern: '/blog/[...slug]',
      methods: ['GET'],
      prerendered_html: null,
      hydration_js: mainJs,
      requires_auth: false,
      admin_only: false,
      meta: { title: 'Blog — NAWA SvelteKit' },
      is_endpoint: false,
      ssr_wasm: null,
      layout: null
    },
    {
      pattern: '/admin',
      methods: ['GET'],
      prerendered_html: null,
      hydration_js: mainJs,
      requires_auth: true,
      admin_only: true,
      meta: { title: 'Admin — NAWA SvelteKit' },
      is_endpoint: false,
      ssr_wasm: null,
      layout: null
    }
  ],
  global_css: globalCss,
  main_js: mainJs,
  favicon: favicon,
  spa_fallback: 'spa.html',
  default_meta: {
    title: 'NAWA SvelteKit App',
    description: 'Built with Vite + Svelte 5, served by NAWA Rust binary'
  }
};

fs.writeFileSync(
  path.join(nawaDir, 'manifest.json'),
  JSON.stringify(manifest, null, 2)
);

console.log(`\n✓ _nawa/manifest.json generated — ${manifest.routes.length} routes`);
console.log(`  Main JS: ${mainJs ?? 'none'}`);
console.log(`  Global CSS: ${globalCss ?? 'none'}`);
console.log(`  SPA fallback: spa.html`);
