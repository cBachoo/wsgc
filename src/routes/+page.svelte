<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Calculator from "$lib/Calculator.svelte";
  import type { Calc, ModelKey, Spark } from "$lib/spark";

  // Native decorations are disabled (tauri.conf.json), so we drive the window
  // ourselves. These map to the core:window:* permissions in capabilities.
  const appWindow = getCurrentWindow();

  const STORE_KEY = "wsgc.state.v2";
  const STORE_KEY_V1 = "wsgc.state.v1";
  const THEME_KEY = "wsgc.theme";

  type Theme = "light" | "dark";
  type Workspace = { calcs: Calc[]; nextId: number };

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

  function freshCalc(id: number, name: string): Calc {
    return { id, name, sparks: [], modelKey: "exponential" };
  }

  function loadWorkspace(): Workspace {
    try {
      const raw = localStorage.getItem(STORE_KEY);
      if (raw) return JSON.parse(raw);

      // Migrate the old single-calculator state into one calc.
      const v1 = localStorage.getItem(STORE_KEY_V1);
      if (v1) {
        const { sparks, modelKey } = JSON.parse(v1) as {
          sparks: Spark[];
          modelKey: ModelKey;
        };
        return { calcs: [{ id: 1, name: "Calc 1", sparks, modelKey }], nextId: 2 };
      }
    } catch {
      /* corrupt or unavailable — fall through to a fresh workspace */
    }
    return { calcs: [freshCalc(1, "Calc 1")], nextId: 2 };
  }

  const initial = loadWorkspace();

  // --- reactive state --------------------------------------------------------
  let calcs = $state<Calc[]>(initial.calcs);
  let nextId = $state(initial.nextId);
  let theme = $state<Theme>(loadTheme());

  // Persist the whole workspace so calcs survive a restart.
  $effect(() => {
    const snapshot: Workspace = { calcs: $state.snapshot(calcs), nextId };
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

  function addCalc() {
    calcs.push(freshCalc(nextId, `Calc ${calcs.length + 1}`));
    nextId += 1;
  }

  function removeCalc(id: number) {
    calcs = calcs.filter((c) => c.id !== id);
  }

  // Window controls.
  const minimize = () => appWindow.minimize();
  const toggleMaximize = () => appWindow.toggleMaximize();
  const closeWindow = () => appWindow.close();
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
    <button type="button" class="btn-add" onclick={addCalc}>+ New Calcparison</button>
    <p class="tagline"><small>calcparison is slang for calculator comparison btw</small></p>
  </header>

  <div class="calcs">
    {#each calcs as c (c.id)}
      <Calculator calc={c} removable={calcs.length > 1} onRemove={removeCalc} />
    {/each}
  </div>
</main>
