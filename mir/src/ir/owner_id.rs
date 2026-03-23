//! Stable identifiers for parameter owners.
//!
//! The aim of these ids is to preserve parameter identity across inlining/unrolling where owners
//! are duplicated. The approach is simple: allocate monotonically increasing ids from an atomic
//! counter. The tradeoff is that ids are process-local and not stable across runs, which is fine
//! for in-memory identity tracking.

use std::sync::atomic::{AtomicU64, Ordering};

/// Stable identifier for parameter owners (Function/Evaluator/For).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct OwnerId(u64);

impl OwnerId {
    /// Sentinel for an unknown owner.
    pub const UNKNOWN: OwnerId = OwnerId(0);

    /// Allocate a new unique owner id.
    pub fn next() -> OwnerId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        OwnerId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns true if this id is the sentinel `UNKNOWN`.
    pub fn is_unknown(self) -> bool {
        self.0 == 0
    }
}

impl Default for OwnerId {
    fn default() -> Self {
        OwnerId::UNKNOWN
    }
}
