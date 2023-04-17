use super::Identifier;

// TYPES
// ================================================================================================
pub type TraceSegment = u8;

/// [TraceAccess] is used to represent accessing one or more elements in the execution trace during
/// constraint evaluation.
///
/// - `trace_segment`: specifies how many trace commitments have preceded the specified segment.
/// - `col_idx`: specifies the index of the column within that trace segment at which the access
///   starts.
/// - `size`: refers to how many columns are being accessed.
/// - `row_offset`: specifies the offset from the current row.
///
/// For example, a single element in the "next" row of
/// the "main" trace would be specified by a trace_segment of 0, a size of 1, and a row_offset of 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceAccess {
    trace_segment: TraceSegment,
    col_idx: usize,
    size: usize,
    row_offset: usize,
}

impl TraceAccess {
    /// Creates a new [TraceAccess].
    pub fn new(
        trace_segment: TraceSegment,
        col_idx: usize,
        size: usize,
        row_offset: usize,
    ) -> Self {
        Self {
            trace_segment,
            col_idx,
            size,
            row_offset,
        }
    }

    /// Gets the trace segment of this [TraceAccess].
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Gets the column index of this [TraceAccess].
    pub fn col_idx(&self) -> usize {
        self.col_idx
    }

    /// Gets the size of this [TraceAccess].
    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets the row offset of this [TraceAccess].
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }
}

/// [TraceBinding] is used to represent one or more columns in the execution trace that are bound to
/// a name. For single columns, the size is 1. For groups, the size is the number of columns in the
/// group. The offset is the column index in the trace where the first column of the binding starts.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TraceBinding {
    binding: Identifier,
    trace_segment: TraceSegment,
    offset: usize,
    size: usize,
}

impl TraceBinding {
    /// Creates a new trace binding.
    pub fn new(binding: Identifier, trace_segment: usize, offset: usize, size: u64) -> Self {
        Self {
            binding,
            trace_segment: trace_segment as TraceSegment,
            offset,
            size: size as usize,
        }
    }

    /// Returns the name of the trace binding.
    pub fn name(&self) -> &str {
        self.binding.name()
    }

    /// Returns the trace segment of the trace binding.
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Returns the offset of the trace binding.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the size of the trace binding.
    pub fn size(&self) -> usize {
        self.size
    }
}
