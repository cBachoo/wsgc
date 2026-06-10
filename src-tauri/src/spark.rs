//! White-skill spark generation math.
//!
//! This module is plain Rust with no Tauri dependency, so it can be unit-tested
//! on its own. `lib.rs` wraps `calculate` in a `#[tauri::command]` and the Svelte
//! frontend calls that command via `invoke`.
//!
//! Each model maps (category, lineage_count) -> probability that a skill
//! generates as a white spark on the offspring.
//!   - category      : white (single circle), double (double circle), gold
//!   - lineage_count : how many ancestors carry that spark (0-6; max lineage
//!                     is 2 parents + 4 grandparents)

use serde::{Deserialize, Serialize};

/// The kind of spark. `serde(rename_all = "lowercase")` means JSON
/// `"white"` <-> `Category::White`, matching the strings the frontend sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    White,
    Double,
    Gold,
}

impl Category {
    /// Base generation rate before any lineage bonus.
    fn base(self) -> f64 {
        match self {
            Category::White => 0.20,
            Category::Double => 0.25,
            Category::Gold => 0.40,
        }
    }
}

/// Which probability model to use. The frontend sends `"exponential"` etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Exponential,
    Piecewise,
    Community,
}

impl Model {
    /// Probability that one spark of `cat` with `n` lineage copies generates
    /// onto the offspring. Clamped to at most 1.0.
    pub fn probability(self, cat: Category, n: u32) -> f64 {
        let p = match self {
            // aoneko_pochi (2024): base_rate * 1.1^lineage_count.
            Model::Exponential => cat.base() * 1.10_f64.powi(n as i32),

            // Empirical piecewise-at-2: +first per copy for copies 1-2,
            // +rest for every copy beyond that.
            Model::Piecewise => {
                let (first, rest) = match cat {
                    Category::White => (0.02, 0.0275),
                    Category::Double => (0.025, 0.034375),
                    Category::Gold => (0.04, 0.055),
                };
                let a = n.min(2) as f64;
                let b = n.saturating_sub(2) as f64;
                cat.base() + first * a + rest * b
            }

            // Original community hypothesis: linear boost per copy.
            Model::Community => {
                let boost = match cat {
                    Category::White | Category::Double => 0.025,
                    Category::Gold => 0.05,
                };
                cat.base() + boost * n as f64
            }
        };
        p.min(1.0)
    }
}

/// One tracked spark, as sent from the frontend.
#[derive(Debug, Clone, Deserialize)]
pub struct Spark {
    pub name: String,
    pub cat: Category,
    pub count: u32,
}

/// A tracked spark plus its computed generation probability, sent back to the UI.
#[derive(Debug, Serialize)]
pub struct SparkResult {
    pub name: String,
    pub cat: Category,
    pub count: u32,
    pub probability: f64,
}

/// The full result of a calculation. `rename_all = "camelCase"` turns
/// `at_least` into `atLeast` so it reads naturally in TypeScript.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcResult {
    /// Per-spark breakdown, in the same order as the input.
    pub sparks: Vec<SparkResult>,
    /// Expected number of white sparks generated (sum of probabilities).
    pub expected: f64,
    /// `dist[k]` = probability that exactly `k` sparks generate.
    pub dist: Vec<f64>,
    /// `at_least[k]` = probability that `k` or more sparks generate.
    pub at_least: Vec<f64>,
}

/// Poisson-binomial distribution: given independent success probabilities,
/// return a vec where index `k` holds P(exactly `k` successes).
///
/// We fold the trials in one at a time. After processing some trials, `dist`
/// is their distribution; adding trial `p` either fails (keeps `k`) or
/// succeeds (shifts to `k + 1`).
pub fn poisson_binomial(probs: &[f64]) -> Vec<f64> {
    let mut dist = vec![1.0];
    for &p in probs {
        let mut next = vec![0.0; dist.len() + 1];
        for (k, &v) in dist.iter().enumerate() {
            next[k] += v * (1.0 - p);
            next[k + 1] += v * p;
        }
        dist = next;
    }
    dist
}

/// Run the full calculation for a set of sparks under one model.
pub fn calculate(sparks: &[Spark], model: Model) -> CalcResult {
    let probs: Vec<f64> = sparks
        .iter()
        .map(|s| model.probability(s.cat, s.count))
        .collect();

    let expected = probs.iter().sum();
    let dist = poisson_binomial(&probs);

    // at_least[k] = sum of dist[k..], built by accumulating from the top down.
    let n = probs.len();
    let mut at_least = vec![0.0; n + 1];
    let mut acc = 0.0;
    for k in (0..=n).rev() {
        acc += dist[k];
        at_least[k] = acc;
    }

    let sparks = sparks
        .iter()
        .zip(&probs)
        .map(|(s, &probability)| SparkResult {
            name: s.name.clone(),
            cat: s.cat,
            count: s.count,
            probability,
        })
        .collect();

    CalcResult {
        sparks,
        expected,
        dist,
        at_least,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compare two floats with a small tolerance.
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn base_rates_match_python() {
        // At n = 0 every model returns the base rate.
        assert!(approx(Model::Exponential.probability(Category::White, 0), 0.20));
        assert!(approx(Model::Piecewise.probability(Category::Double, 0), 0.25));
        assert!(approx(Model::Community.probability(Category::Gold, 0), 0.40));
    }

    #[test]
    fn exponential_grows_by_ten_percent() {
        assert!(approx(
            Model::Exponential.probability(Category::White, 2),
            0.20 * 1.10 * 1.10
        ));
    }

    #[test]
    fn piecewise_switches_at_two() {
        // White: 0.20 + 0.02*2 + 0.0275*1 = 0.2675 at n = 3.
        assert!(approx(Model::Piecewise.probability(Category::White, 3), 0.2675));
    }

    #[test]
    fn probability_is_clamped_to_one() {
        // Gold under exponential blows past 1.0 well before n = 6.
        assert!(Model::Exponential.probability(Category::Gold, 6) <= 1.0);
    }

    #[test]
    fn poisson_binomial_sums_to_one() {
        let dist = poisson_binomial(&[0.2, 0.5, 0.9]);
        assert_eq!(dist.len(), 4);
        assert!(approx(dist.iter().sum(), 1.0));
    }

    #[test]
    fn expected_equals_sum_of_probs() {
        let sparks = vec![
            Spark { name: "a".into(), cat: Category::White, count: 1 },
            Spark { name: "b".into(), cat: Category::Gold, count: 0 },
        ];
        let result = calculate(&sparks, Model::Community);
        let manual: f64 = result.sparks.iter().map(|s| s.probability).sum();
        assert!(approx(result.expected, manual));
        // at_least[0] is "0 or more" -> always 1.0.
        assert!(approx(result.at_least[0], 1.0));
    }
}
