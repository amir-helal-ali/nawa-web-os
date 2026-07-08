// CommonJS shim for environments that don't support ESM.
'use strict';
const fs = require('node:fs');
const path = require('node:path');

module.exports = function adapter(options = {}) {
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
            const nawaDir = path.resolve(outDir);
            const pagesDir = path.join(nawaDir, pages);
            const assetsDir = path.join(nawaDir, assets);
            builder.rimraf(nawaDir);
            builder.mkdirp(pagesDir);
            builder.mkdirp(assetsDir);
            if (fallback) {
                await fs.promises.writeFile(
                    path.join(pagesDir, fallback),
                    builder.generateFallback({ title: 'NAWA App' })
                );
            }
            const routes = [];
            if (prerender) {
                for (const [route, _] of builder.prerendered.entries()) {
                    const fileName = routeToFileName(route);
                    await fs.promises.writeFile(
                        path.join(pagesDir, fileName),
                        builder.readPrerenderedFile(route)
                    );
                    routes.push({ pattern: route, methods: ['GET'], prerendered_html: fileName, hydration_js: null, requires_auth: false, admin_only: false, meta: {}, is_endpoint: false, ssr_wasm: null, layout: null });
                }
            }
            builder.writeClient(assetsDir);
            const manifest = {
                version: 1, app_name: 'NAWA App', built_at: new Date().toISOString(),
                sveltekit_version: '2.x', routes,
                global_css: null, main_js: 'app.js', favicon: null,
                spa_fallback: fallback, default_meta: {},
            };
            await fs.promises.writeFile(path.join(nawaDir, 'manifest.json'), JSON.stringify(manifest, null, 2));
            console.log(`\n✓ adapter-nawa: ${routes.length} routes → ${nawaDir}\n`);
        },
    };
};

function routeToFileName(route) {
    if (route === '/' || route === '') return 'index.html';
    return route.replace(/^\//, '').replace(/\//g, '_') + '.html';
}
