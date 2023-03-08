use super::{Constant, SemanticError};

// TYPE ALIASES
// ================================================================================================

pub type PublicInput = (String, usize);
pub type PeriodicColumn = Vec<u64>;

// DECLARATIONS
// ================================================================================================

/// TODO: docs
#[derive(Default, Debug)]
pub(super) struct Declarations {
    /// A vector of constants declared in the AirScript module.
    constants: Vec<Constant>,

    /// A map of the Air's periodic columns using the index of the column within the declared
    /// periodic columns as the key and the vector of periodic values as the value
    periodic_columns: Vec<PeriodicColumn>,

    /// A vector of public inputs with each value as a tuple of input identifier and it's array
    /// size.
    public_inputs: Vec<PublicInput>,

    /// Number of random values. For array initialized in `rand: [n]` form it will be `n`, and for
    /// `rand: [a, b[n], c, ...]` it will be length of the flattened array.
    num_random_values: u16,

    /// The widths of each segment of the trace, in order such that the index is the trace segment
    /// and the value is the number of columns in this segment.
    trace_segment_widths: Vec<u16>,
}

impl Declarations {
    // --- ACCESSORS ------------------------------------------------------------------------------

    pub(super) fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub(super) fn periodic_columns(&self) -> &[PeriodicColumn] {
        &self.periodic_columns
    }

    pub(super) fn public_inputs(&self) -> &[PublicInput] {
        &self.public_inputs
    }

    /// Gets the number of trace segments that were specified for this AIR.
    pub(super) fn num_trace_segments(&self) -> usize {
        self.trace_segment_widths.len() + 1
    }

    /// Returns a slice containing the widths of all trace segments.
    pub(super) fn trace_segment_widths(&self) -> &[u16] {
        &self.trace_segment_widths
    }

    /// Returns the width of the requested trace segment.
    ///
    /// # Errors
    /// - Returns an error if the specified trace segment does not exist.
    pub(super) fn trace_segment_width(&self, trace_segment: usize) -> Result<u16, SemanticError> {
        if trace_segment > self.num_trace_segments() {
            return Err(SemanticError::trace_segment_access_out_of_bounds(
                trace_segment,
                self.num_trace_segments(),
            ));
        }

        Ok(self.trace_segment_widths[trace_segment])
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    pub(super) fn add_constant(&mut self, constant: Constant) {
        self.constants.push(constant)
    }

    pub(super) fn add_periodic_column(&mut self, periodic_column: PeriodicColumn) {
        self.periodic_columns.push(periodic_column)
    }

    pub(super) fn add_public_input(&mut self, public_input: PublicInput) {
        self.public_inputs.push(public_input)
    }

    pub(super) fn set_num_random_values(&mut self, num_random_values: u16) {
        self.num_random_values = num_random_values;
    }

    pub(super) fn set_trace_segment_width(&mut self, trace_segment: usize, width: u16) {
        if trace_segment >= self.trace_segment_widths.len() {
            self.trace_segment_widths.resize(trace_segment + 1, 0);
        }
        self.trace_segment_widths[trace_segment] = width;
    }
}
