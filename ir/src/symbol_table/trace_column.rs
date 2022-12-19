use super::TraceSegment;

/// Describes a column in the execution trace by the trace segment to which it belongs and its
/// index within that segment.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TraceColumn {
    trace_segment: TraceSegment,
    col_idx: usize,
}

impl TraceColumn {
    /// Creates a [TraceColumn] in the specified trace segment at the specified index.
    pub(super) fn new(trace_segment: TraceSegment, col_idx: usize) -> Self {
        Self {
            trace_segment,
            col_idx,
        }
    }

    /// Gets the trace segment of this [TraceColumn].
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Gets the column index of this [TraceColumn].
    pub fn col_idx(&self) -> usize {
        self.col_idx
    }
}
