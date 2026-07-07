import './app.css';
import App from './App.svelte';

// Mount the Svelte app into the #svelte element.
// NAWA's renderer creates this element in the SPA shell.
const target = document.getElementById('svelte') ?? document.body;
const app = new App({ target });

console.log('[NAWA-Svelte] App hydrated. Bootstrap:', window.__NAWA__);

export default app;
