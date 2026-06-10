<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { cubicOut } from "svelte/easing";
  import {
    CATEGORIES,
    MODELS,
    pct,
    type Calc,
    type CalcResult,
    type Category,
    type Spark,
  } from "./spark";

  // `calc` is a reactive proxy owned by the parent's $state array, so mutating
  // calc.sparks / calc.modelKey / calc.name flows straight back up (and gets
  // persisted by the parent). `removable` hides the remove button on the last
  // remaining calc.
  let {
    calc,
    removable = false,
    onRemove,
  }: {
    calc: Calc;
    removable?: boolean;
    onRemove?: (id: number) => void;
  } = $props();

  let result = $state<CalcResult | null>(null);

  // input row fields (local to this calculator)
  let nameInput = $state("");
  let catInput = $state<Category>("white");
  let countInput = $state(1);

  // Recompute on the Rust side whenever this calc's sparks or model change.
  $effect(() => {
    const payload = { sparks: $state.snapshot(calc.sparks), model: calc.modelKey };
    if (calc.sparks.length === 0) {
      result = null;
      return;
    }
    invoke<CalcResult>("calc_sparks", payload).then((r) => (result = r));
  });

  function addSpark() {
    const name = nameInput.trim();
    if (!name) return;
    calc.sparks.push({ name, cat: catInput, count: countInput });
    nameInput = "";
  }

  function removeSpark(idx: number) {
    calc.sparks.splice(idx, 1);
  }

  function clearAll() {
    calc.sparks = [];
  }

  function bump(s: Spark, delta: number) {
    s.count = Math.max(0, Math.min(6, s.count + delta));
  }

  // Most likely outcome (the mode of the distribution) for the callout.
  const mode = $derived.by(() => {
    if (!result) return null;
    let best = 0;
    for (let k = 1; k < result.dist.length; k++) {
      if (result.dist[k] > result.dist[best]) best = k;
    }
    return { k: best, p: result.dist[best] };
  });

  const maxDist = $derived(result ? Math.max(...result.dist) : 1);

  // Fade + collapse height together so a removed row's space disappears in
  // sync with the fade, and a new row grows + fades in.
  function collapseFade(node: HTMLElement, { duration = 200 } = {}) {
    const s = getComputedStyle(node);
    const height = parseFloat(s.height);
    const pt = parseFloat(s.paddingTop);
    const pb = parseFloat(s.paddingBottom);
    return {
      duration,
      easing: cubicOut,
      css: (t: number) =>
        `overflow: hidden;` +
        `opacity: ${t};` +
        `height: ${t * height}px;` +
        `padding-top: ${t * pt}px;` +
        `padding-bottom: ${t * pb}px;`,
    };
  }
</script>

<section class="calc-panel">
  <div class="calc-head">
    <input
      class="calc-name"
      bind:value={calc.name}
      aria-label="Calculator name"
      placeholder="Calculator"
    />
    {#if removable}
      <button
        type="button"
        class="calc-remove"
        onclick={() => onRemove?.(calc.id)}
        aria-label="Remove this calculator"
        title="Remove calculator"
      >×</button>
    {/if}
  </div>

  <!-- add a spark -->
  <section class="card">
    <form class="add-form" onsubmit={(e) => { e.preventDefault(); addSpark(); }}>
      <div class="field grow">
        <span class="field-label">Skill</span>
        <input
          type="text"
          bind:value={nameInput}
          placeholder="e.g. Professor of Curvature"
          autocomplete="off"
        />
      </div>

      <div class="field">
        <span class="field-label">Type</span>
        <div class="segmented" role="radiogroup" aria-label="Spark type">
          {#each CATEGORIES as c}
            <button
              type="button"
              class="seg seg-{c.key}"
              class:active={catInput === c.key}
              aria-pressed={catInput === c.key}
              onclick={() => (catInput = c.key)}
            >
              {c.short}
            </button>
          {/each}
        </div>
      </div>

      <div class="field">
        <span class="field-label">In lineage</span>
        <div class="stepper">
          <button type="button" aria-label="decrease" onclick={() => (countInput = Math.max(0, countInput - 1))}>−</button>
          <input type="number" min="0" max="6" bind:value={countInput} />
          <button type="button" aria-label="increase" onclick={() => (countInput = Math.min(6, countInput + 1))}>+</button>
        </div>
      </div>

      <button type="submit" class="btn-primary">Add spark</button>
    </form>
  </section>

  <!-- model selector -->
  <section class="card">
    <h2 class="card-title">Probability model</h2>
    <div class="models">
      {#each MODELS as m}
        <button
          type="button"
          class="model-card"
          class:active={calc.modelKey === m.key}
          aria-pressed={calc.modelKey === m.key}
          onclick={() => (calc.modelKey = m.key)}
        >
          <span class="model-name">{m.label}</span>
          <span class="model-desc">{m.desc}</span>
        </button>
      {/each}
    </div>
  </section>

  <!-- tracked sparks -->
  <section class="card">
    <div class="card-head">
      <h2 class="card-title">Tracked sparks</h2>
      {#if calc.sparks.length > 0}
        <button type="button" class="btn-ghost" onclick={clearAll}>Clear all</button>
      {/if}
    </div>

    {#if calc.sparks.length === 0}
      <div class="empty">
        <p>No sparks tracked yet.</p>
        <p class="empty-hint">Add a skill above to start calculating.</p>
      </div>
    {:else}
      <ul class="spark-list">
        {#each calc.sparks as s, i (s)}
          <li class="spark-row" transition:collapseFade={{ duration: 200 }}>
            <div class="spark-id">
              <span class="dot dot-{s.cat}"></span>
              <span class="spark-name">{s.name}</span>
            </div>

            <select class="type-select badge-{s.cat}" bind:value={s.cat} aria-label="type">
              {#each CATEGORIES as c}
                <option value={c.key}>{c.label}</option>
              {/each}
            </select>

            <div class="stepper sm">
              <button type="button" aria-label="fewer copies" onclick={() => bump(s, -1)}>−</button>
              <span class="count">{s.count}</span>
              <button type="button" aria-label="more copies" onclick={() => bump(s, 1)}>+</button>
            </div>

            <div class="prob">
              <div class="prob-bar">
                <div
                  class="prob-fill fill-{s.cat}"
                  style="width: {(result?.sparks[i]?.probability ?? 0) * 100}%"
                ></div>
              </div>
              <span class="prob-val">{result?.sparks[i] ? pct(result.sparks[i].probability) : "…"}</span>
            </div>

            <button
              type="button"
              class="remove"
              aria-label="remove {s.name}"
              onclick={() => removeSpark(i)}
            >×</button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- results -->
  {#if result}
    <section class="card results">
      <div class="summary">
        <div class="stat">
          <span class="stat-label">Expected white sparks</span>
          <span class="stat-value">{result.expected.toFixed(2)}</span>
          <span class="stat-sub">of {calc.sparks.length} tracked</span>
        </div>
        {#if mode}
          <div class="stat">
            <span class="stat-label">Most likely outcome</span>
            <span class="stat-value">{mode.k}</span>
            <span class="stat-sub">{pct(mode.p)} chance</span>
          </div>
        {/if}
      </div>

      <h2 class="card-title">How many will actually generate</h2>
      <div class="dist">
        {#each result.dist as d, k}
          <div class="dist-row" class:peak={mode?.k === k}>
            <span class="dist-k">{k}</span>
            <div class="dist-track">
              <div class="dist-fill" style="width: {(d / maxDist) * 100}%"></div>
            </div>
            <span class="dist-exact">{pct(d)}</span>
            <span class="dist-least">≥ {pct(result.atLeast[k])}</span>
          </div>
        {/each}
      </div>
      <p class="legend">
        <span class="legend-item"><b>exactly</b> this many</span>
        <span class="legend-item"><b>≥</b> this many or more</span>
      </p>
    </section>
  {/if}
</section>
