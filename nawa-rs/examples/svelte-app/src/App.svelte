<main>
  <h1>🦀 NAWA + SvelteKit (real Vite build)</h1>
  <p>This page was compiled by Vite + Svelte 5, then served by NAWA — no Node.js at runtime.</p>
  <Counter />
  <NawaState />
</main>

<script>
  import Counter from './Counter.svelte';
  import NawaState from './NawaState.svelte';
</script>

<style>
  main {
    font-family: 'Noto Sans Arabic', system-ui, sans-serif;
    max-width: 800px;
    margin: 2rem auto;
    padding: 0 1rem;
    color: #e0e0e0;
    background: #0d0c0a;
    min-height: 100vh;
  }
  h1 {
    color: #f59e0b;
  }
  p {
    color: #888;
    line-height: 1.8;
  }
</style>
