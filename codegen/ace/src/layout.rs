use std::{collections::BTreeMap, ops::Range};

use air_ir::{Air, Identifier, PublicInputAccess, PublicInputTableAccess, TraceAccess};

use crate::circuit::Node;

/// For each set of inputs read from the transcript, we treat them as extension field elements
/// and pad them with zeros to the next multiple of 4. They can then be unhashed to a double-word
/// aligned region in memory.
const ELEMENT_ALIGNMENT: usize = 1;
const WORD_ALIGNMENT: usize = 2;
const DOUBLE_WORD_ALIGNMENT: usize = 4;

const NUM_QUOTIENT_PARTS: usize = 8;

/// Describes the layout of inputs given to an ACE circuit.
/// Each set of variables is aligned to the next multiple of 4, ensuring they can be efficiently
/// unhashed from the transcript and that each input region is aligned to `HASH_ALIGNMENT`.
///
/// We assume the following about the underlying `Air` from which the layout is constructed
/// - The proof always contains a `main` and `aux` segment, even when the latter is unused,
/// - The maximal degree of an [`Air`] is `9`, such that the quotient can be decomposed in 8 chunks.
///   TODO(Issue: #391): Derive the degree generically.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Layout {
    /// Region for each set of public inputs, sorted by `Identifier`.
    /// The arrays of inputs are laid out contiguously.
    /// The last array is padded to ensure this entire region is double-word aligned.
    pub public_inputs: BTreeMap<Identifier, InputRegion>,
    /// Region containing the random-reduced public input tables for bus boundary constraints.
    /// Each variable is word-aligned (interleaving with unused variables),
    /// and the region is double-word aligned.
    pub reduced_tables_region: InputRegion,
    /// Index of a specific reduced table within the [`reduced_tables_region`].
    pub reduced_tables: BTreeMap<PublicInputTableAccess, usize>,
    /// Index of the random challenge α used to randomize the multiset/logUp argument
    /// in the *aux* trace.
    pub random_alpha: usize,
    /// Index of the random challenge β used to randomly reduce/fingerprint bus messages.
    pub random_beta: usize,
    /// Regions containing the evaluations of each segment, ordered by
    /// `trace[row_offset][segment]`.
    ///
    /// # Detail:
    /// Note that we make the following assumptions which do not affect the evaluation of the
    /// circuit but facilitate the implementation of the overall MASM verifier. In particular,
    /// these properties facilitate the computation of the DEEP composition polynomial and
    /// can lead to more efficient unhashing from the transcript into memory.
    /// - The [`Air`] from which the layout is derived can only contain a *main* and *aux* trace
    ///   (the latter can be empty).
    /// - We treat the *quotient* as the third trace, handling it in the same way as the witness
    ///   traces. This requires the prover to provide the out-of-domain evaluations in the *next*
    ///   row of each quotient part. These are unused by the circuit.
    /// - The rows must be ordered as follows: ```ignore main_curr, aux_curr, quotient_curr,
    ///   main_next, aux_next, quotient_next. ```
    /// - Each segment is double-word aligned to facilitate the Merkle-tree openings during the FRI
    ///   query phase. In practice, the traces are padded with empty columns.
    ///
    /// # TODO(Issue #391):
    /// The degree of the quotient is fixed to 8 matching the degree of the VM constraints, but
    /// the actual degree can be derived from the [`Air`].
    pub trace_segments: [[InputRegion; 3]; 2],
    /// Region containing the [`StarkVar`] variables.
    pub stark_vars: InputRegion,
    /// Total number of inputs, padded to the next word-multiple.
    pub num_inputs: usize,
}

impl Layout {
    /// Returns a new [`Layout`] from a description of an [`Air`].
    /// Each region is aligned according to the requirements of the MASM verifier.
    pub fn new(air: &Air) -> Self {
        let offset = &mut 0;

        // Returns an `InputRegion` of a given width and increments the offset
        // to satisfy the alignment.
        fn next_region(current_offset: &mut usize, width: usize, alignment: usize) -> InputRegion {
            let offset = *current_offset;
            *current_offset += width.next_multiple_of(alignment);
            InputRegion { offset, width }
        }

        fn align(offset: &mut usize, alignment: usize) {
            *offset += offset.next_multiple_of(alignment);
        }

        // The arrays of all public inputs are stored contiguously.
        let public_inputs: BTreeMap<_, _> = air
            .public_inputs
            .iter()
            .map(|(ident, pi)| (*ident, next_region(offset, pi.size(), ELEMENT_ALIGNMENT)))
            .collect();

        // Ensure the entire region containing the public inputs is double-word aligned
        // since it is hashed as one contiguous array.
        align(offset, DOUBLE_WORD_ALIGNMENT);

        // List of all reduced public input table accesses in canonical order.
        let reduced_table_accesses = air.reduced_public_input_table_accesses();

        // Region containing all reduced public input table values.
        // For MASM efficiency, we store one reduced table per word.
        // Each variable therefore occupies two "variable slots".
        let reduced_tables_region =
            next_region(offset, 2 * reduced_table_accesses.len(), DOUBLE_WORD_ALIGNMENT);

        // Mapping of each access to its index within `reduced_tables_region`
        // The index is doubled to match the "one variable per word" requirement.
        let reduced_tables: BTreeMap<_, _> = reduced_table_accesses
            .into_iter()
            .enumerate()
            .map(|(index, access)| (access, 2 * index))
            .collect();

        // Random challenges α, β.
        let random_alpha = *offset;
        let random_beta = *offset + 1;
        *offset += 2;
        // The next region must be word-aligned to facilitate hashing.
        align(offset, WORD_ALIGNMENT);

        // TODO(Issue: #391): Use the following to derive the degree generically, and maybe add it
        // to `Air`
        // let degree = [0, 1]
        //     .into_iter()
        //     .flat_map(|segment| air.integrity_constraint_degrees(segment.into()))
        //     .map(|deg| deg.base())
        //     .max()
        //     .unwrap();
        //
        // let num_quotient_elements = (degree - 1)
        //     .next_power_of_two()
        //     .next_multiple_of(HASH_ALIGNMENT);
        // let quotient_degree = Self::quotient_parts(air);
        let num_quotient_parts = NUM_QUOTIENT_PARTS;

        // For better uniformity, the proof will include the evaluations of the quotient
        // at the shifted point. Even if these are not used, they facilitate uniform evaluation
        // of the DEEP composition polynomial.
        let segment_widths = [
            // The Air always contains a main segment
            air.trace_segment_widths[0] as usize,
            // If there is no aux segment, we set it to 0.
            air.trace_segment_widths.get(1).copied().unwrap_or(0) as usize,
            // Quotient is stored as a segment
            num_quotient_parts,
        ];

        // Each segment must be double-word aligned to facilitate the opening of rows
        // during the FRI query phase.
        // At the moment, we do so by padding each trace with zero-valued columns.
        let trace_segments = [0, 1].map(|_row_offset| {
            segment_widths.map(|width| next_region(offset, width, DOUBLE_WORD_ALIGNMENT))
        });

        let stark_vars = next_region(offset, StarkVar::num_vars(), WORD_ALIGNMENT);

        // Ensure the entire input region is word aligned
        align(offset, WORD_ALIGNMENT);

        Self {
            public_inputs,
            reduced_tables_region,
            reduced_tables,
            random_alpha,
            random_beta,
            trace_segments,
            stark_vars,
            num_inputs: *offset,
        }
    }

    /// Input node associated with a public input variable.
    pub fn public_input_node(&self, public_input: &PublicInputAccess) -> Option<Node> {
        self.public_inputs
            .get(&public_input.name)
            .and_then(|region| region.as_node(public_input.index))
    }

    /// Input node associated with a reduced public input table variable.
    pub fn reduced_table_node(&self, table_access: &PublicInputTableAccess) -> Option<Node> {
        self.reduced_tables
            .get(table_access)
            .and_then(|index| self.reduced_tables_region.as_node(*index))
    }

    /// Input node associated with a trace variable.
    pub fn trace_access_node(&self, trace_access: &TraceAccess) -> Option<Node> {
        let TraceAccess { segment, column, row_offset } = *trace_access;
        // We should only be able to access the main and aux segments.
        if segment > 1 {
            return None;
        };
        let segments_in_row = self.trace_segments.get(row_offset)?;
        let segment_region = segments_in_row.get(segment)?;
        segment_region.as_node(column)
    }

    /// Input node associated with the variable for the random challenge α.
    pub fn random_alpha_node(&self) -> Node {
        Node::Input(self.random_alpha)
    }

    /// Input node associated with the variable for the random challenge β.
    pub fn random_beta_node(&self) -> Node {
        Node::Input(self.random_beta)
    }

    /// Input nodes associated with the quotient polynomial coefficients.
    pub fn quotient_nodes(&self) -> Vec<Node> {
        self.trace_segments[0][2].iter_nodes().collect()
    }

    /// Input node associated with an auxiliary STARK challenge/variable.
    pub fn stark_node(&self, stark_var: StarkVar) -> Node {
        self.stark_vars.as_node(stark_var.into()).unwrap()
    }
}

/// An [`InputRegion`] is a section of indices within the overall list of inputs to the
/// [`AceCircuit`](crate::AceCircuit).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct InputRegion {
    pub offset: usize,
    pub width: usize,
}

impl InputRegion {
    /// Returns the index within the overall input section of a variable in this region,
    /// as long as it contains `index`.
    pub fn index(&self, index: usize) -> Option<usize> {
        (index < self.width).then(|| self.offset + index)
    }

    /// Returns an input [`Node`] for the input at `index` in this region if it is within bounds.
    pub fn as_node(&self, index: usize) -> Option<Node> {
        self.index(index).map(Node::Input)
    }

    /// Returns the range for all indices of inputs in this region.
    pub fn range(&self) -> Range<usize> {
        self.offset..(self.offset + self.width)
    }

    /// Returns an iterator of all input [`Node`]s in this region.
    pub fn iter_nodes(&self) -> impl Iterator<Item = Node> + use<'_> {
        self.range().map(Node::Input)
    }
}

/// List of STARK variables and challenges, derived from the public parameters and proof transcript.
#[derive(Copy, Clone, Debug)]
pub enum StarkVar {
    /// The variable g⁻² corresponding to the penultimate point in the subgroup over which the
    /// trace is interpolated.
    GenPenultimate = 0,
    /// The variable g⁻¹ corresponding to the last point in the subgroup over which the trace is
    /// interpolated.
    GenLast = 1,
    /// The variable α used as for random linear-combination of constraints.
    Alpha = 2,
    /// The variable z at which the constraints evaluation check is performed.
    Z = 3,
    /// The variable zⁿ, where `n = trace_len`
    ZPowN = 4,
    /// The variable `zᵐᵃˣ`, where `max` is equal to `trace_len / max_cycle_len`. Details can be
    /// found in [`crate::builder::CircuitBuilder::periodic_column`]
    ZMaxCycle = 5,
}

impl StarkVar {
    pub const fn num_vars() -> usize {
        6
    }
}

impl TryFrom<usize> for StarkVar {
    type Error = usize;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::GenPenultimate),
            1 => Ok(Self::GenLast),
            2 => Ok(Self::Alpha),
            3 => Ok(Self::Z),
            4 => Ok(Self::ZPowN),
            5 => Ok(Self::ZMaxCycle),
            _ => Err(value),
        }
    }
}

impl From<StarkVar> for usize {
    fn from(value: StarkVar) -> usize {
        value as usize
    }
}
