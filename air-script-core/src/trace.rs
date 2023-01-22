use super::Identifier;

// TYPES
// ================================================================================================
pub type TraceSegment = u8;

/// [IndexedTraceAccess] is used to represent accessing an element in the execution trace during
/// constraint evaluation. The trace_segment specifies
/// how many trace commitments have preceded the specified segment. `col_idx` specifies the index
/// of the column within that trace segment, and `row_offset` specifies the offset from the current
/// row. For example, an element in the "next" row of the "main" trace would be specified by
/// a trace_segment of 0 and a row_offset of 1.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct IndexedTraceAccess {
    trace_segment: TraceSegment,
    col_idx: usize,
    row_offset: usize,
}

impl IndexedTraceAccess {
    pub fn new(trace_segment: TraceSegment, col_idx: usize, row_offset: usize) -> Self {
        Self {
            trace_segment,
            col_idx,
            row_offset,
        }
    }

    /// Gets the trace segment of this [IndexedTraceAccess].
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Gets the column index of this [IndexedTraceAccess].
    pub fn col_idx(&self) -> usize {
        self.col_idx
    }

    /// Gets the row offset of this [IndexedTraceAccess].
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }
}

/// [NamedTraceAccess] is used to indicate a column in the trace by specifying its index within a
/// set of trace columns with the given identifier. If the identifier refers to a single column
/// then the index is always zero.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NamedTraceAccess {
    name: Identifier,
    idx: usize,
    row_offset: usize,
}

impl NamedTraceAccess {
    pub fn new(name: Identifier, idx: usize, row_offset: usize) -> Self {
        Self {
            name,
            idx,
            row_offset,
        }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Gets the row offset of this [NamedTraceAccess].
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }
}
