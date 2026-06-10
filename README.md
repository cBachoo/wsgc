# White Spark Generation Calc

A small desktop tool for estimating white skill spark generation. Add the skills
you're hoping to inherit, set how many white sparks are in the lineage, and it
tells you the chance each one generates as a white spark — plus the full
distribution of how many you'll actually land on a given run.

Built with **Tauri + SvelteKit + TypeScript** on the frontend and **Rust** on
the backend. Ported from the original Tkinter prototype in
`old/white_spark_calc.py`.

## Download

Grab the latest **portable** executable from the [Releases page](https://github.com/cBachoo/wsgc/releases)
— no installer, just download and run:

- **Windows** — `wsgc-windows.exe`
- **macOS** — `wsgc-macos`
- **Linux** — `wsgc-linux`

> Windows needs the [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/),
> which ships with Windows 10/11 by default.

## What it does

For each skill you track, you provide:

- **Type** — white (single circle), double circle, or gold
- **Copies in lineage** — how many parents carry that spark (0–6)

It then shows the per-skill generation chance, the expected number of white
sparks, and the full distribution of how many will generate.

## Models

The base rates and lineage scaling come from a large community dataset. Three
models are selectable:

| Model | Formula | Notes |
|-------|---------|-------|
| **Exponential** (recommended) | `base_rate × 1.1^copies` | Fits all three categories well (p ≥ 0.84). |
| **Piecewise-at-2** | base + a flat boost for copies 1–2, larger boost after | Best empirical fit to the data. |
| **Community linear** | `base + flat % × copies` | The original community hypothesis; kept for reference. |

Base rates: white **20%**, double circle **25%**, gold **40%**.

The exponential model is credited to [aoneko_pochi (2024)](https://x.com/aoneko_pochi/status/1762370579603304731),
who first observed that a per-copy multiplicative boost fits white spark
generation far better than a linear one. These are descriptive fits, not
confirmed in-game formulas. Community Linear is the baseline expectation noted
in crazyfellow's parenting and gene guide. Piecewise-at-2 is based on Aya's
conclusions from their CM 10 – CM 12 room match data.

## Architecture

The probability math lives in Rust; the frontend is a thin UI that calls it.

- **`src-tauri/src/spark.rs`** — the probability engine in plain Rust (no Tauri
  dependency, so it's unit-testable). Holds the three models, the
  Poisson-binomial distribution, and `calculate`.
- **`src-tauri/src/lib.rs`** — exposes `calculate` as the `calc_sparks`
  Tauri command.
- **`src/routes/+page.svelte`** — the UI: calls `invoke("calc_sparks", …)`
  and renders the results.
- **`src/app.css`** — all styling (global), themed via CSS variables
  (Gruvbox light/dark).

## Develop

```sh
yarn install
yarn tauri dev      # run the desktop app with hot reload
```

## Test the Rust engine

```sh
cd src-tauri
cargo test
```

## Build locally

```sh
yarn tauri build --no-bundle    # portable exe at src-tauri/target/release/wsgc.exe
```

Bundling is disabled (`bundle.active: false` in `tauri.conf.json`), so the build
produces just the standalone executable — no installer.

## Releasing (CI builds the portable binaries)

A GitHub Actions workflow (`.github/workflows/release.yml`) builds the portable
executable on Windows, macOS and Linux.

- **Cut a release:** push a version tag, and CI attaches the binaries to a
  draft GitHub Release:

  ```sh
  git tag v0.1.0
  git push origin v0.1.0
  ```

  Then publish the draft release on GitHub.

- **Just want binaries:** trigger the workflow manually from the **Actions** tab
  (`Run workflow`). It builds the executables and uploads them as downloadable
  run artifacts without creating a release.

## App icon

Replace every icon from a single 1024×1024 PNG:

```sh
yarn tauri icon path/to/icon.png
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## License

Personal project — use freely.
