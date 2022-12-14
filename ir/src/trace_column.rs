use super::TraceSegment;

/// Describes a column in the execution trace by the trace segment to which it belongs and its
/// index within that segment.
#[derive(Debug, Copy, Clone)]
pub struct TraceColumn {
    trace_segment: TraceSegment,
    col_idx: usize,
}

impl TraceColumn {
    /// Creates a [TraceColumn] in the specified trace segment at the specified index.
    pub fn new(trace_segment: TraceSegment, col_idx: usize) -> Self {
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

#[derive(Debug, Clone)]
pub struct TraceColumnGroup {
    trace_segment: TraceSegment,
    size: usize,
    start_col_idx: usize,
}

impl TraceColumnGroup {
    /// Creates a [TraceColumnGroup] in the specified trace segment at the specified start index.
    pub fn new(trace_segment: TraceSegment, size: usize, start_col_idx: usize) -> Self {
        Self {
            trace_segment,
            size,
            start_col_idx,
        }
    }

    /// Gets the trace segment of this [TraceColumn].
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets the start column index of this [TraceColumn].
    pub fn start_col_idx(&self) -> usize {
        self.start_col_idx
    }
}