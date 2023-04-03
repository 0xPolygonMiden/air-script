use super::TraceSegment;

/// TODO: docs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceParameterAccess {
    name: String,
    trace_segment: TraceSegment,
    idx: usize,
    row_offset: usize,
}

impl TraceParameterAccess {
    pub fn new(name: String, trace_segment: TraceSegment, idx: usize, row_offset: usize) -> Self {
        Self {
            name,
            trace_segment,
            idx,
            row_offset,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Gets the row offset of this [TraceAccessArgument].
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }
}
