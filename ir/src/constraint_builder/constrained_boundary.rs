use super::{ConstraintDomain, TraceSegment};
use std::fmt::Display;

/// [ConstrainedBoundary] represents the location within the trace where a boundary constraint is
/// applied. It identifies the trace segment, the trace column index, and the [ConstraintDomain].
/// The [ConstraintDomain] is assumed to be a valid boundary, either FirstRow or LastRow.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct ConstrainedBoundary {
    trace_segment: TraceSegment,
    col_idx: usize,
    domain: ConstraintDomain,
}

impl ConstrainedBoundary {
    pub fn new(trace_segment: TraceSegment, col_idx: usize, domain: ConstraintDomain) -> Self {
        debug_assert!(domain.is_boundary());
        Self {
            trace_segment,
            col_idx,
            domain,
        }
    }
}

impl Display for ConstrainedBoundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} of column {} in segment {}",
            self.domain, self.col_idx, self.trace_segment
        )
    }
}
