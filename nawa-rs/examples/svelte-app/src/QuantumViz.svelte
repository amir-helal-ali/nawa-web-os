<script>
  import { onMount } from 'svelte';

  let superposition = ['|0⟩', '|1⟩', '|2⟩', '|3⟩'];
  let probabilities = [0.25, 0.25, 0.25, 0.25];
  let collapsed = null;
  let isCollapsed = false;
  let tunneling = { energy: 100, best: 100, temp: 50, tunnelings: 0 };

  function measure() {
    const r = Math.random();
    let cumulative = 0;
    for (let i = 0; i < probabilities.length; i++) {
      cumulative += probabilities[i];
      if (r <= cumulative) {
        collapsed = superposition[i];
        isCollapsed = true;
        return;
      }
    }
    collapsed = superposition[superposition.length - 1];
    isCollapsed = true;
  }

  function reset() {
    isCollapsed = false;
    collapsed = null;
    probabilities = [0.25, 0.25, 0.25, 0.25];
  }

  function tunnel() {
    const newEnergy = 100 + Math.random() * 20 - 10;
    const barrier = newEnergy - tunneling.energy;
    const prob = Math.exp(-barrier / tunneling.temp);

    if (newEnergy <= tunneling.energy || Math.random() < prob) {
      tunneling.energy = newEnergy;
      if (newEnergy < tunneling.best) tunneling.best = newEnergy;
      if (newEnergy > tunneling.energy) tunneling.tunnelings++;
    }
    tunneling.temp *= 0.999;
  }

  onMount(() => {
    // Animate probabilities
    const interval = setInterval(() => {
      if (!isCollapsed) {
        probabilities = probabilities.map(() => Math.random());
        const total = probabilities.reduce((a, b) => a + b, 0);
        probabilities = probabilities.map(p => p / total);
      }
    }, 2000);

    // Animate tunneling
    const tunnelInterval = setInterval(tunnel, 500);

    return () => { clearInterval(interval); clearInterval(tunnelInterval); };
  });
</script>

<div class="quantum-viz">
  <div class="quantum-grid">
    <!-- Superposition -->
    <div class="quantum-card">
      <h3>⚛️ التراكب الكمي</h3>
      <div class="superposition-bars">
        {#each superposition as state, i}
          <div class="bar-container">
            <span class="bar-label">{state}</span>
            <div class="bar-track">
              <div
                class="bar-fill"
                style="width: {probabilities[i] * 100}%; transition: width 1s ease;"
              ></div>
            </div>
            <span class="bar-prob">{(probabilities[i] * 100).toFixed(1)}%</span>
          </div>
        {/each}
      </div>
      <div class="quantum-result">
        {#if isCollapsed}
          <div class="collapsed">
            <span class="collapse-label">انهار إلى:</span>
            <span class="collapse-value">{collapsed}</span>
          </div>
          <button class="qbtn" on:click={reset}>↻ إعادة</button>
        {:else}
          <button class="qbtn qbtn-primary" on:click={measure}>📐 قياس (Collapse)</button>
        {/if}
      </div>
    </div>

    <!-- Tunneling -->
    <div class="quantum-card">
      <h3>🏔️ النفق الكمي</h3>
      <div class="tunnel-viz">
        <div class="energy-display">
          <div class="energy-row">
            <span>الطاقة الحالية:</span>
            <span class="energy-val">{tunneling.energy.toFixed(2)}</span>
          </div>
          <div class="energy-row">
            <span>أفضل طاقة:</span>
            <span class="energy-val best">{tunneling.best.toFixed(2)}</span>
          </div>
          <div class="energy-row">
            <span>الحرارة:</span>
            <span class="energy-val temp">{tunneling.temp.toFixed(2)}</span>
          </div>
          <div class="energy-row">
            <span>مرات النفق:</span>
            <span class="energy-val">{tunneling.tunnelings}</span>
          </div>
        </div>
        <div class="barrier-viz">
          <div class="barrier" style="opacity: {tunneling.temp / 50};">
            <span>حاجز طاقة</span>
          </div>
          <div class="particle" style="bottom: {100 - tunneling.energy}%;"></div>
        </div>
      </div>
    </div>

    <!-- QEC -->
    <div class="quantum-card">
      <h3>🛡️ تصحيح الأخطاء الكمي</h3>
      <div class="qec-viz">
        <div class="qec-blocks">
          <div class="qec-block ok">✓ نسخة 1</div>
          <div class="qec-block ok">✓ نسخة 2</div>
          <div class="qec-block ok">✓ نسخة 3</div>
        </div>
        <div class="qec-info">
          <p>رمز التكرار الثلاثي مع تصويت الأغلبية</p>
          <div class="qec-status">
            <span class="status-dot ok"></span>
            <span>لا أخطاء — تصحيح نشط</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .quantum-viz { margin-top: 1rem; }
  .quantum-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;
  }
  .quantum-card {
    background: rgba(20, 20, 30, 0.6);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(245, 158, 11, 0.15);
    border-radius: 16px;
    padding: 1.5rem;
  }
  .quantum-card h3 {
    color: #f59e0b;
    margin-bottom: 1rem;
    font-size: 1.1rem;
  }
  .superposition-bars { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1rem; }
  .bar-container { display: flex; align-items: center; gap: 0.5rem; }
  .bar-label { font-family: monospace; width: 30px; color: #8b8b9a; font-size: 0.85rem; }
  .bar-track { flex: 1; height: 8px; background: rgba(255,255,255,0.05); border-radius: 4px; overflow: hidden; }
  .bar-fill { height: 100%; background: linear-gradient(90deg, #f59e0b, #fbbf24); border-radius: 4px; }
  .bar-prob { font-family: monospace; font-size: 0.8rem; color: #8b8b9a; width: 50px; text-align: left; }
  .quantum-result { text-align: center; padding-top: 0.5rem; }
  .collapsed { margin-bottom: 0.75rem; }
  .collapse-label { color: #8b8b9a; font-size: 0.85rem; }
  .collapse-value { color: #10b981; font-family: monospace; font-weight: 700; font-size: 1.2rem; margin-right: 0.5rem; }
  .qbtn {
    padding: 0.5rem 1.5rem; border-radius: 8px; border: 1px solid rgba(245,158,11,0.3);
    background: transparent; color: #e8e8ef; cursor: pointer; font-size: 0.9rem; transition: all 0.2s;
  }
  .qbtn-primary { background: linear-gradient(135deg, #f59e0b, #fbbf24); color: #0a0a0f; border: none; font-weight: 600; }
  .qbtn:hover { transform: translateY(-1px); }

  .tunnel-viz { display: flex; flex-direction: column; gap: 1rem; }
  .energy-display { display: flex; flex-direction: column; gap: 0.4rem; }
  .energy-row { display: flex; justify-content: space-between; font-size: 0.85rem; }
  .energy-row span:first-child { color: #8b8b9a; }
  .energy-val { font-family: monospace; color: #f59e0b; font-weight: 600; }
  .energy-val.best { color: #10b981; }
  .energy-val.temp { color: #ef4444; }
  .barrier-viz {
    position: relative; height: 100px; background: rgba(255,255,255,0.03);
    border-radius: 8px; overflow: hidden;
  }
  .barrier {
    position: absolute; top: 20%; left: 40%; width: 20%; height: 60%;
    background: linear-gradient(180deg, rgba(239,68,68,0.2), rgba(239,68,68,0.1));
    border-left: 2px dashed rgba(239,68,68,0.3); border-right: 2px dashed rgba(239,68,68,0.3);
    display: flex; align-items: center; justify-content: center; font-size: 0.7rem; color: #ef4444;
    transition: opacity 0.5s;
  }
  .particle {
    position: absolute; left: 10%; width: 12px; height: 12px; border-radius: 50%;
    background: #10b981; box-shadow: 0 0 10px rgba(16,185,129,0.5); transition: bottom 0.5s;
  }

  .qec-viz { display: flex; flex-direction: column; gap: 1rem; }
  .qec-blocks { display: flex; gap: 0.5rem; }
  .qec-block {
    flex: 1; padding: 0.5rem; border-radius: 8px; text-align: center; font-size: 0.8rem;
    background: rgba(16,185,129,0.1); border: 1px solid rgba(16,185,129,0.2); color: #10b981;
  }
  .qec-info p { color: #8b8b9a; font-size: 0.85rem; margin-bottom: 0.5rem; }
  .qec-status { display: flex; align-items: center; gap: 0.5rem; font-size: 0.85rem; }
  .status-dot { width: 8px; height: 8px; border-radius: 50%; }
  .status-dot.ok { background: #10b981; animation: pulse 2s infinite; }
  @keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.5; } }
</style>
