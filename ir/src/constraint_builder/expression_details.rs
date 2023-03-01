use super::{ConstraintDomain, NodeIndex, TraceSegment};

/// A struct containing the node index that is the root of the expression, the trace segment to
/// which the expression is applied, and the constraint domain against which any constraint
/// containing this expression must be applied.
#[derive(Debug, Copy, Clone)]
pub(super) struct ExprDetails {
    root_idx: NodeIndex,
    trace_segment: TraceSegment,
    domain: ConstraintDomain,
}

impl ExprDetails {
    pub(super) fn new(
        root_idx: NodeIndex,
        trace_segment: TraceSegment,
        domain: ConstraintDomain,
    ) -> Self {
        Self {
            root_idx,
            trace_segment,
            domain,
        }
    }

    pub(super) fn root_idx(&self) -> NodeIndex {
        self.root_idx
    }

    pub(super) fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    pub(super) fn domain(&self) -> ConstraintDomain {
        self.domain
    }
}
