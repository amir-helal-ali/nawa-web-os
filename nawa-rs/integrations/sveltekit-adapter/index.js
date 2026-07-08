/**
 * adapter-nawa — SvelteKit adapter for NAWA Web Operating System
 *
 * Compiles a SvelteKit app into a `_nawa/` directory that can be embedded
 * in a NAWA Rust binary. No Node.js required at runtime.
 *
 * ## Output structure
 *
 * _nawa/
 * ├── manifest.json          # Route table + metadata
 * ├── pages/
 * │   ├── index.html         # Pre-rendered HTML for "/"
 * │   ├── about.html         # Pre-rendered HTML for "/about"
 * │   └── spa.html           # SPA fallback (when no prerender matches)
 * ├── assets/
 * │   ├── app.js             # Main JS bundle (entry point)
 * │   ├── app.css            # Global CSS
 * │   ├── index.js           # Hydration chunk for "/"
 * │   └── users.[id].js      # Hydration chunk for "/users/[id]"
 * └── ssr/                   # (future) WASM modules for server-side rendering
 *
 * ## Usage in svelte.config.js
 *
 * ```js
 * import adapter from 'adapter-nawa';
 * export default {
 *   kit: {
 *     adapter: adapter({
 *       outDir: '_nawa',          // default: '_nawa'
 *       prerender: true,           // default: true
 *       fallback: 'spa.html',     // default: 'spa.html'
 *       pages: 'pages',            // default: 'pages'
 *       assets: 'assets',          // default: 'assets'
 *     })
 *   }
 * };
 * ```
 */

import fs from 'node:fs';
import path from 'node:path';

/**
 * @typedef {Object} AdapterOptions
 * @property {string} [outDir='_nawa'] — Output directory (relative to project root).
 * @property {boolean} [prerender=true] — Whether to pre-render pages at build time.
 * @property {string} [fallback='spa.html'] — SPA fallback page name.
 * @property {string} [pages='pages'] — Subdirectory for HTML pages.
 * @property {string} [assets='assets'] — Subdirectory for JS/CSS/assets.
 */

/** @type {import('@sveltejs/kit').Adapter} */
export default function adapter(options = {}) {
    const {
        outDir = '_nawa',
        prerender = true,
        fallback = 'spa.html',
        pages = 'pages',
        assets = 'assets',
    } = options;

    return {
        name: 'adapter-nawa',
        async adapt(builder) {
            const root = builder.config.kit?.outDir || '.';
            const nawaDir = path.resolve(outDir);
            const pagesDir = path.join(nawaDir, pages);
            const assetsDir = path.join(nawaDir, assets);

            // Clean & create directories.
            builder.rimraf(nawaDir);
            builder.mkdirp(pagesDir);
            builder.mkdirp(assetsDir);

            // 1. Write fallback SPA page (if configured).
            if (fallback) {
                const fallbackContents = builder.generateFallback({
                    title: builder.config.kit?.app?.name || 'NAWA App'
                });
                await fs.promises.writeFile(
                    path.join(pagesDir, fallback),
                    fallbackContents
                );
            }

            // 2. Pre-render pages.
            const routes = [];
            if (prerender) {
                const entries = [...builder.prerendered.entries()];
                for (const [route, _] of entries) {
                    // Each prerendered entry is written to its own HTML file.
                    // The file name is derived from the route path.
                    const fileName = routeToFileName(route);
                    const html = builder.readPrerenderedFile(route);
                    await fs.promises.writeFile(
                        path.join(pagesDir, fileName),
                        html
                    );
                    routes.push({
                        pattern: route,
                        methods: ['GET'],
                        prerendered_html: fileName,
                        hydration_js: null,
                        requires_auth: false,
                        admin_only: false,
                        meta: {},
                        is_endpoint: false,
                        ssr_wasm: null,
                        layout: null,
                    });
                }
            }

            // 3. Write client-side assets (JS, CSS).
            builder.writeClient(assetsDir);

            // 4. Write the main entry chunk.
            const mainJsName = builder.getClientEntry?.() || 'app.js';
            const globalCss = findGlobalCss(assetsDir);

            // 5. Collect dynamic routes (no prerender) for SPA hydration.
            for (const route of builder.routes || []) {
                if (!routes.find(r => r.pattern === route.id)) {
                    routes.push({
                        pattern: route.id,
                        methods: route.methods || ['GET'],
                        prerendered_html: null,
                        hydration_js: route.hydrationChunk || null,
                        requires_auth: route.requiresAuth || false,
                        admin_only: route.adminOnly || false,
                        meta: route.meta || {},
                        is_endpoint: route.type === 'endpoint',
                        ssr_wasm: null,
                        layout: route.layout || null,
                    });
                }
            }

            // 6. Write manifest.json — the bridge to NAWA Rust runtime.
            const manifest = {
                version: 1,
                app_name: builder.config.kit?.app?.name || 'NAWA App',
                built_at: new Date().toISOString(),
                sveltekit_version: builder.config.kit?.version || '2.x',
                routes,
                global_css: globalCss,
                main_js: mainJsName,
                favicon: findFavicon(assetsDir),
                spa_fallback: fallback,
                default_meta: {
                    title: builder.config.kit?.app?.name || 'NAWA App',
                    description: 'Built with NAWA + SvelteKit',
                },
            };

            await fs.promises.writeFile(
                path.join(nawaDir, 'manifest.json'),
                JSON.stringify(manifest, null, 2)
            );

            console.log(`\n✓ adapter-nawa: built ${routes.length} routes → ${nawaDir}\n`);
            console.log(`  Pages:   ${pagesDir}`);
            console.log(`  Assets:  ${assetsDir}`);
            console.log(`  Manifest:${path.join(nawaDir, 'manifest.json')}\n`);
        },
    };
}

/**
 * Convert a SvelteKit route path to a safe file name.
 * "/" → "index.html"
 * "/about" → "about.html"
 * "/users/[id]" → "users/[id].html" (preserved as a pattern marker)
 */
function routeToFileName(route) {
    if (route === '/' || route === '') return 'index.html';
    const safe = route.replace(/^\//, '').replace(/\//g, '_');
    return `${safe}.html`;
}

/** Find the global CSS file in the assets directory. */
function findGlobalCss(assetsDir) {
    try {
        const files = fs.readdirSync(assetsDir);
        const css = files.find(f => f.endsWith('.css') && /^app\.|^entry\./.test(f));
        return css || null;
    } catch {
        return null;
    }
}

/** Find the favicon in the assets directory. */
function findFavicon(assetsDir) {
    try {
        const files = fs.readdirSync(assetsDir);
        return files.find(f => f === 'favicon.ico' || f === 'favicon.png') || null;
    } catch {
        return null;
    }
}
