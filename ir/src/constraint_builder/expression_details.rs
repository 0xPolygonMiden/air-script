use super::{ConstraintDomain, NodeIndex, TraceSegment};

/// A struct containing the node index that is the root of the expression, the trace segment to
/// which the expression is applied, and the constraint domain against which any constraint
/// containing this expression must be applied.
/// TODO: get rid of need to make this public
#[derive(Debug, Copy, Clone)]
pub(crate) struct ExprDetails {
    root_idx: NodeIndex,
    trace_segment: TraceSegment,
    domain: ConstraintDomain,
}

impl ExprDetails {
    // TODO: get rid of need to make these methods public
    pub(crate) fn new(
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

    pub(crate) fn root_idx(&self) -> NodeIndex {
        self.root_idx
    }

    pub(crate) fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    pub(crate) fn domain(&self) -> ConstraintDomain {
        self.domain
    }
}
