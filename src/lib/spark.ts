// Shared types and constants for the calculator UI.
// These types mirror the Rust structs in src-tauri/src/spark.rs.

export type Category = "white" | "double" | "gold";
export type ModelKey = "exponential" | "piecewise" | "community";
export type Spark = { name: string; cat: Category; count: number };

export type SparkResult = Spark & { probability: number };
export type CalcResult = {
  sparks: SparkResult[];
  expected: number;
  dist: number[];
  atLeast: number[];
};

// One calculator instance in the workspace.
export type Calc = {
  id: number;
  name: string;
  sparks: Spark[];
  modelKey: ModelKey;
};

export const CATEGORIES: { key: Category; label: string; short: string }[] = [
  { key: "white", label: "White (single)", short: "White" },
  { key: "double", label: "Double circle", short: "Double" },
  { key: "gold", label: "Gold", short: "Gold" },
];

export const MODELS: { key: ModelKey; label: string; desc: string }[] = [
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

export const pct = (x: number) => `${(x * 100).toFixed(1)}%`;
