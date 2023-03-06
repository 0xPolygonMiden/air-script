use super::TraceSegment;

/// Describes columns in the execution trace by the trace segment to which it belongs, it's size
/// and it's offset.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TraceColumns {
    trace_segment: TraceSegment,
    offset: usize,
    size: usize,
}

impl TraceColumns {
    /// Creates a [TraceColumns] in the specified trace segment of the specified size and offset.
    pub fn new(trace_segment: TraceSegment, offset: usize, size: usize) -> Self {
        Self {
            trace_segment,
            size,
            offset,
        }
    }

    /// Returns the trace segment of this [TraceColumns].
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Returns the offset of this [TraceColumns].
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the size of this [TraceColumns].
    pub fn size(&self) -> usize {
        self.size
    }
}
