import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Vite config for building a NAWA-embedded SvelteKit-style app.
// Output goes to _nawa/ so nawad can serve it directly (no Node.js at runtime).
export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: '_nawa',
    emptyOutDir: true,
    // Generate a manifest so NAWA knows which routes exist.
    // We use a custom plugin below to write _nawa/manifest.json.
    rollupOptions: {
      input: {
        main: './src/main.ts',
      },
      output: {
        entryFileNames: 'assets/app.js',
        assetFileNames: 'assets/[name][extname]',
      },
    },
  },
});
