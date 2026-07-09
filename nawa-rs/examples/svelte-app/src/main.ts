import './app.css';
import App from './App.svelte';

const target = document.getElementById('svelte') ?? document.body;
const app = new App({ target });

console.log('[NAWA-SvelteKit] App hydrated. Bootstrap:', window.__NAWA__);
console.log('[NAWA-SvelteKit] WebSocket URL:', window.__NAWA__?.wsUrl);
console.log('[NAWA-SvelteKit] Polling disabled — pure push via WebSocket.');

export default app;
