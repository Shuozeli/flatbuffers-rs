use rand::rngs::StdRng;
use rand::Rng;
use std::collections::VecDeque;

/// Abstraction over random decisions. Every random choice in the generator
/// goes through this trait, enabling deterministic testing via `ScriptedChooser`.
pub trait Chooser {
    /// Binary decision with given probability (replaces `gen_bool`).
    fn flip(&mut self, probability: f64) -> bool;
    /// Pick unsigned integer in `[min, max]` inclusive (replaces `gen_range`).
    fn pick(&mut self, min: usize, max: usize) -> usize;
    /// Pick signed integer in `[min, max]` inclusive (replaces `gen_range` for defaults).
    fn pick_i64(&mut self, min: i64, max: i64) -> i64;
    /// Pick from weighted categories, returns index (replaces manual bucket logic).
    fn pick_weighted(&mut self, weights: &[u32]) -> usize;
}

/// Real implementation backed by a seeded `StdRng`.
pub struct RngChooser(pub StdRng);

impl Chooser for RngChooser {
    fn flip(&mut self, probability: f64) -> bool {
        self.0.gen_bool(probability)
    }

    fn pick(&mut self, min: usize, max: usize) -> usize {
        if min == max {
            return min;
        }
        self.0.gen_range(min..=max)
    }

    fn pick_i64(&mut self, min: i64, max: i64) -> i64 {
        if min == max {
            return min;
        }
        self.0.gen_range(min..=max)
    }

    fn pick_weighted(&mut self, weights: &[u32]) -> usize {
        let total: u32 = weights.iter().sum();
        assert!(total > 0, "pick_weighted called with all-zero weights");
        let roll = self.0.gen_range(0..total);
        let mut cursor = 0u32;
        for (i, &w) in weights.iter().enumerate() {
            cursor += w;
            if roll < cursor {
                return i;
            }
        }
        weights.len() - 1
    }
}

/// Test implementation that returns predetermined values from queues.
/// Each queue is consumed independently based on which method is called.
///
/// When a queue is exhausted:
/// - If a default is set (via `with_defaults()` or individual `with_default_*` methods),
///   the default value is returned (clamped to valid range).
/// - Otherwise, the call panics with a descriptive message.
///
/// This allows tests to script only the decisions they care about and let
/// everything else use safe defaults.
#[derive(Default)]
pub struct ScriptedChooser {
    pub flips: VecDeque<bool>,
    pub picks: VecDeque<usize>,
    pub picks_i64: VecDeque<i64>,
    pub weighted_picks: VecDeque<usize>,
    default_flip: Option<bool>,
    default_pick: Option<usize>,
    default_pick_i64: Option<i64>,
    default_weighted: Option<usize>,
}

impl ScriptedChooser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable safe defaults for all queues: `false` for flips, `0` for picks/weighted.
    /// When a queue runs out, the default is returned instead of panicking.
    pub fn with_defaults(mut self) -> Self {
        self.default_flip = Some(false);
        self.default_pick = Some(0);
        self.default_pick_i64 = Some(0);
        self.default_weighted = Some(0);
        self
    }

    pub fn with_default_flip(mut self, v: bool) -> Self {
        self.default_flip = Some(v);
        self
    }

    pub fn with_default_pick(mut self, v: usize) -> Self {
        self.default_pick = Some(v);
        self
    }

    pub fn with_default_pick_i64(mut self, v: i64) -> Self {
        self.default_pick_i64 = Some(v);
        self
    }

    pub fn with_default_weighted(mut self, v: usize) -> Self {
        self.default_weighted = Some(v);
        self
    }

    pub fn with_flips(mut self, values: impl IntoIterator<Item = bool>) -> Self {
        self.flips.extend(values);
        self
    }

    pub fn with_picks(mut self, values: impl IntoIterator<Item = usize>) -> Self {
        self.picks.extend(values);
        self
    }

    pub fn with_picks_i64(mut self, values: impl IntoIterator<Item = i64>) -> Self {
        self.picks_i64.extend(values);
        self
    }

    pub fn with_weighted(mut self, values: impl IntoIterator<Item = usize>) -> Self {
        self.weighted_picks.extend(values);
        self
    }
}

impl Chooser for ScriptedChooser {
    fn flip(&mut self, _probability: f64) -> bool {
        self.flips
            .pop_front()
            .or(self.default_flip)
            .expect("ScriptedChooser: flips queue exhausted (no default set)")
    }

    fn pick(&mut self, min: usize, max: usize) -> usize {
        let v = self
            .picks
            .pop_front()
            .or(self.default_pick)
            .expect("ScriptedChooser: picks queue exhausted (no default set)");
        v.clamp(min, max)
    }

    fn pick_i64(&mut self, min: i64, max: i64) -> i64 {
        let v = self
            .picks_i64
            .pop_front()
            .or(self.default_pick_i64)
            .expect("ScriptedChooser: picks_i64 queue exhausted (no default set)");
        v.clamp(min, max)
    }

    fn pick_weighted(&mut self, _weights: &[u32]) -> usize {
        self.weighted_picks
            .pop_front()
            .or(self.default_weighted)
            .expect("ScriptedChooser: weighted_picks queue exhausted (no default set)")
    }
}
