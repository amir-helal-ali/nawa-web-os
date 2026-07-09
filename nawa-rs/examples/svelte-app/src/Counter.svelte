<button class="counter-btn">
  <span class="counter-label">النقرات</span>
  <span class="counter-value">{count}</span>
</button>

<script>
  let count = 0;
</script>

<style>
  .counter-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.875rem 2rem;
    border: 1px solid rgba(245, 158, 11, 0.3);
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(245, 158, 11, 0.1), rgba(251, 191, 36, 0.05));
    color: #e8e8ef;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s;
    font-family: inherit;
    margin: 1rem 0;
  }
  .counter-btn:hover {
    border-color: #f59e0b;
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(245, 158, 11, 0.2);
  }
  .counter-btn:active { transform: translateY(0); }
  .counter-label { color: #8b8b9a; font-size: 0.9rem; }
  .counter-value {
    color: #f59e0b;
    font-family: 'JetBrains Mono', monospace;
    font-weight: 800;
    font-size: 1.3rem;
    min-width: 30px;
    text-align: center;
  }
</style>
