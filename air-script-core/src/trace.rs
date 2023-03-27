use super::{Identifier, Range};

// TYPES
// ================================================================================================
pub type TraceSegment = u8;

/// A named group of columns of the specified size and trace segment.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ColumnGroup {
    name: Identifier,
    trace_segment: TraceSegment,
    size: u64,
}

impl ColumnGroup {
    /// Creates a new column group.
    pub fn new(name: Identifier, trace_segment: TraceSegment, size: u64) -> Self {
        Self {
            name,
            trace_segment,
            size,
        }
    }

    /// Returns the name of the column group.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the trace segment of the column group.
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Returns the size of the column group.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the name, trace segment, and size of the column group.
    pub fn into_parts(self) -> (Identifier, TraceSegment, u64) {
        (self.name, self.trace_segment, self.size)
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

/// [IndexedTraceAccess] is used to represent accessing an element in the execution trace during
/// constraint evaluation. The trace_segment specifies
/// how many trace commitments have preceded the specified segment. `col_idx` specifies the index
/// of the column within that trace segment, and `row_offset` specifies the offset from the current
/// row. For example, an element in the "next" row of the "main" trace would be specified by
/// a trace_segment of 0 and a row_offset of 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexedTraceAccess {
    trace_segment: TraceSegment,
    col_idx: usize,
    size: usize,
    row_offset: usize,
}

impl IndexedTraceAccess {
    /// Creates a new [IndexedTraceAccess].
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

    /// Gets the trace segment of this [IndexedTraceAccess].
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Gets the column index of this [IndexedTraceAccess].
    pub fn col_idx(&self) -> usize {
        self.col_idx
    }

    /// Gets the size of this [IndexedTraceAccess].
    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets the row offset of this [IndexedTraceAccess].
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }
}

/// [TraceBindingAccess] is used to indicate a column in the trace by specifying its offset within
/// a set of trace columns with the given identifier. If the identifier refers to a single column
/// then the index is always zero.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceBindingAccess {
    binding: Identifier,
    col_offset: usize,
    size: TraceBindingAccessSize,
    row_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TraceBindingAccessSize {
    Single,
    Slice(Range),
    Full,
}

impl TraceBindingAccess {
    pub fn new(
        binding: Identifier,
        col_offset: usize,
        size: TraceBindingAccessSize,
        row_offset: usize,
    ) -> Self {
        Self {
            binding,
            col_offset,
            size,
            row_offset,
        }
    }

    pub fn name(&self) -> &str {
        self.binding.name()
    }

    /// Gets the column offset of this [TraceBindingAccess].
    pub fn col_offset(&self) -> usize {
        self.col_offset
    }

    /// Gets the row offset of this [TraceBindingAccess].
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }
}
