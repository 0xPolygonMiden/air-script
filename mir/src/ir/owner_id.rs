use std::sync::atomic::{AtomicU64, Ordering};

/// Stable identifier for parameter owners (Function/Evaluator/For).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct OwnerId(u64);

impl OwnerId {
    pub const UNKNOWN: OwnerId = OwnerId(0);

    pub fn next() -> OwnerId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        OwnerId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn is_unknown(self) -> bool {
        self.0 == 0
    }
}

impl Default for OwnerId {
    fn default() -> Self {
        OwnerId::UNKNOWN
    }
}
