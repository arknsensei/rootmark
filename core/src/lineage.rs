//! Lineage matcher v2 (SPEC §5).

/// Similarity-assignment threshold (SPEC §5.3 step 4). Set by T6 tuning.
pub const THETA: f64 = 0.60;
/// Split/merge threshold (SPEC §5.3 steps 2–3). Set by T6 tuning.
pub const THETA_SPLIT: f64 = 0.70;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchParams {
    pub theta: f64,
    pub theta_split: f64,
}

impl Default for MatchParams {
    fn default() -> Self {
        MatchParams {
            theta: THETA,
            theta_split: THETA_SPLIT,
        }
    }
}
