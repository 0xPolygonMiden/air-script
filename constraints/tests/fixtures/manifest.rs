//! Tag ID bases and counts for constraint groups.

/// Base ID for the system constraint group.
pub const TAG_SYSTEM_BASE: usize = 0;
/// Number of system constraints in this group.
pub const TAG_SYSTEM_COUNT: usize = 13;

/// Base ID for the range constraint group.
pub const TAG_RANGE_BASE: usize = TAG_SYSTEM_BASE + TAG_SYSTEM_COUNT;
/// Number of range constraints in this group.
pub const TAG_RANGE_COUNT: usize = 3;

/// Base ID for the stack general constraint group.
pub const TAG_STACK_GENERAL_BASE: usize = TAG_RANGE_BASE + TAG_RANGE_COUNT;
/// Number of stack general constraints in this group.
pub const TAG_STACK_GENERAL_COUNT: usize = 16;

/// Base ID for the range bus constraint.
///
/// The range bus tag is intentionally placed after the main-trace constraints.
pub const TAG_RANGE_BUS_BASE: usize = TAG_STACK_GENERAL_BASE + TAG_STACK_GENERAL_COUNT;
/// Number of range bus constraints in this group.
pub const TAG_RANGE_BUS_COUNT: usize = 1;

/// Base ID for the stack overflow constraint group.
pub const TAG_STACK_OVERFLOW_BASE: usize = TAG_RANGE_BUS_BASE + TAG_RANGE_BUS_COUNT;
/// Number of stack overflow constraints in this group.
pub const TAG_STACK_OVERFLOW_COUNT: usize = 9;

/// Highest constraint ID (zero-based) for the current group set.
pub const CURRENT_MAX_ID: usize = TAG_STACK_OVERFLOW_BASE + TAG_STACK_OVERFLOW_COUNT - 1;
/// Total tagged constraints in the current group set.
pub const TOTAL_TAGS: usize = CURRENT_MAX_ID + 1;
