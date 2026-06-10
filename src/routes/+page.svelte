<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { cubicOut } from "svelte/easing";

  // Fade + collapse height together so a removed row's space disappears in
  // sync with the fade (the rows below then flow up cleanly), and a new row
  // grows + fades in. Used for both directions via `transition:`.
  function collapseFade(node: HTMLElement, { duration = 200 } = {}) {
    const s = getComputedStyle(node);
    const height = parseFloat(s.height);
    const pt = parseFloat(s.paddingTop);
    const pb = parseFloat(s.paddingBottom);
    return {
      duration,
      easing: cubicOut,
      // opacity fades everything (text + the separator border); height and
      // padding collapse the row's space. The border keeps full width so it
      // fades out instead of shrinking to a hairline.
      css: (t: number) =>
        `overflow: hidden;` +
        `opacity: ${t};` +
        `height: ${t * height}px;` +
        `padding-top: ${t * pt}px;` +
        `padding-bottom: ${t * pb}px;`,
    };
  }

  // Native decorations are disabled (tauri.conf.json), so we drive the window
  // ourselves. These map to the core:window:* permissions in capabilities.
  const appWindow = getCurrentWindow();

  // These types mirror the Rust structs in src-tauri/src/spark.rs.
  type Category = "white" | "double" | "gold";
  type ModelKey = "exponential" | "piecewise" | "community";
  type Spark = { name: string; cat: Category; count: number };

  type SparkResult = Spark & { probability: number };
  type CalcResult = {
    sparks: SparkResult[];
    expected: number;
    dist: number[];
    atLeast: number[];
  };

  const CATEGORIES: { key: Category; label: string; short: string }[] = [
    { key: "white", label: "White (single)", short: "White" },
    { key: "double", label: "Double circle", short: "Double" },
    { key: "gold", label: "Gold", short: "Gold" },
  ];
  const KEY_TO_LABEL: Record<Category, string> = {
    white: "White (single)",
    double: "Double circle",
    gold: "Gold",
  };

  const MODELS: { key: ModelKey; label: string; desc: string }[] = [
    {
      key: "exponential",
      label: "Exponential",
      desc: "base × 1.1ⁿ per lineage copy — recommended",
    },
    {
      key: "piecewise",
      label: "Piecewise-at-2",
      desc: "steeper for the first 2 copies, then linear — best empirical fit",
    },
    {
      key: "community",
      label: "Community linear",
      desc: "a flat boost per copy — reference only",
    },
  ];

  const STORE_KEY = "wsgc.state.v1";
  const THEME_KEY = "wsgc.theme";

  type Theme = "light" | "dark";

  function loadTheme(): Theme {
    try {
      const saved = localStorage.getItem(THEME_KEY);
      if (saved === "light" || saved === "dark") return saved;
      // No saved choice yet — follow the OS preference on first run.
      if (window.matchMedia("(prefers-color-scheme: dark)").matches) return "dark";
    } catch {
      /* unavailable — fall through */
    }
    return "light";
  }

  function loadState(): { sparks: Spark[]; modelKey: ModelKey } {
    try {
      const raw = localStorage.getItem(STORE_KEY);
      if (raw) return JSON.parse(raw);
    } catch {
      /* corrupt or unavailable — fall through to defaults */
    }
    return { sparks: [], modelKey: "exponential" };
  }

  const initial = loadState();

  // --- reactive state --------------------------------------------------------
  let sparks = $state<Spark[]>(initial.sparks);
  let modelKey = $state<ModelKey>(initial.modelKey);
  let result = $state<CalcResult | null>(null);
  let theme = $state<Theme>(loadTheme());

  // input row fields
  let nameInput = $state("");
  let catInput = $state<Category>("white");
  let countInput = $state(1);

  // Recompute on the Rust side whenever sparks or the chosen model change.
  // $state.snapshot strips Svelte's reactive proxy so the data serializes cleanly.
  $effect(() => {
    const payload = { sparks: $state.snapshot(sparks), model: modelKey };
    if (sparks.length === 0) {
      result = null;
      return;
    }
    invoke<CalcResult>("calc_sparks", payload).then((r) => (result = r));
  });

  // Persist to localStorage so tracked sparks survive a restart.
  $effect(() => {
    const snapshot = { sparks: $state.snapshot(sparks), modelKey };
    try {
      localStorage.setItem(STORE_KEY, JSON.stringify(snapshot));
    } catch {
      /* storage may be unavailable — ignore */
    }
  });

  // Apply the chosen theme to <html data-theme> and remember it.
  $effect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem(THEME_KEY, theme);
    } catch {
      /* storage may be unavailable — ignore */
    }
  });

  function toggleTheme() {
    theme = theme === "dark" ? "light" : "dark";
  }

  // Window controls.
  const minimize = () => appWindow.minimize();
  const toggleMaximize = () => appWindow.toggleMaximize();
  const closeWindow = () => appWindow.close();

  function addSpark() {
    const name = nameInput.trim();
    if (!name) return;
    sparks.push({ name, cat: catInput, count: countInput });
    nameInput = "";
  }

  function removeSpark(idx: number) {
    sparks.splice(idx, 1);
  }

  function clearAll() {
    sparks = [];
  }

  function bump(s: Spark, delta: number) {
    s.count = Math.max(0, Math.min(6, s.count + delta));
  }

  const pct = (x: number) => `${(x * 100).toFixed(1)}%`;

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
</script>

<!-- custom titlebar (native decorations are off) -->
<div class="titlebar" data-tauri-drag-region>
  <img class="titlebar-icon" src="/favicon.png" alt="" draggable="false" data-tauri-drag-region />
  <span class="titlebar-title" data-tauri-drag-region>WSGC</span>

  <div class="titlebar-actions">
    <button
      type="button"
      class="tb-btn"
      onclick={toggleTheme}
      aria-label={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
      title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
    >
      {#if theme === "dark"}
        <!-- sun -->
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
        </svg>
      {:else}
        <!-- moon -->
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      {/if}
    </button>

    <span class="tb-sep"></span>

    <button type="button" class="tb-btn" onclick={minimize} aria-label="Minimize" title="Minimize">
      <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true"><rect x="2" y="5.4" width="8" height="1.2" fill="currentColor" /></svg>
    </button>
    <button type="button" class="tb-btn" onclick={toggleMaximize} aria-label="Maximize" title="Maximize">
      <svg viewBox="0 0 12 12" width="11" height="11" fill="none" stroke="currentColor" stroke-width="1.2" aria-hidden="true"><rect x="2.2" y="2.2" width="7.6" height="7.6" /></svg>
    </button>
    <button type="button" class="tb-btn tb-close" onclick={closeWindow} aria-label="Close" title="Close">
      <svg viewBox="0 0 12 12" width="11" height="11" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" aria-hidden="true"><path d="M3 3l6 6M9 3l-6 6" /></svg>
    </button>
  </div>
</div>

<main class="app">
  <header class="hero">
    <h1>White Spark Generation Calc</h1>
    <p class="tagline"><small>calc is slang for calculator btw</small></p>
  </header>

  <div class="duo">
  <!-- add a spark -->
  <section class="card">
    <form class="add-form" onsubmit={(e) => { e.preventDefault(); addSpark(); }}>
      <div class="field grow">
        <label for="skill-name">Skill</label>
        <input
          id="skill-name"
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
        <label for="lineage">In lineage</label>
        <div class="stepper">
          <button
            type="button"
            aria-label="decrease"
            onclick={() => (countInput = Math.max(0, countInput - 1))}
          >−</button>
          <input
            id="lineage"
            type="number"
            min="0"
            max="6"
            bind:value={countInput}
          />
          <button
            type="button"
            aria-label="increase"
            onclick={() => (countInput = Math.min(6, countInput + 1))}
          >+</button>
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
          class:active={modelKey === m.key}
          aria-pressed={modelKey === m.key}
          onclick={() => (modelKey = m.key)}
        >
          <span class="model-name">{m.label}</span>
          <span class="model-desc">{m.desc}</span>
        </button>
      {/each}
    </div>
  </section>
  </div>

  <div class="duo">
  <!-- tracked sparks -->
  <section class="card">
    <div class="card-head">
      <h2 class="card-title">Tracked sparks</h2>
      {#if sparks.length > 0}
        <button type="button" class="btn-ghost" onclick={clearAll}>Clear all</button>
      {/if}
    </div>

    {#if sparks.length === 0}
      <div class="empty">
        <p>No sparks tracked yet.</p>
        <p class="empty-hint">Add a skill above to start calculating.</p>
      </div>
    {:else}
      <ul class="spark-list">
        {#each sparks as s, i (s)}
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
          <span class="stat-sub">of {sparks.length} tracked</span>
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
  </div>
</main>
