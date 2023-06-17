use crate::constants::{AUX_TRACE, MAIN_TRACE};
use crate::visitor::{
    walk_boundary_constraints, walk_constant_bindings, walk_integrity_constraint_degrees,
    walk_integrity_constraints, walk_periodic_columns, walk_public_inputs, AirVisitor,
};
use ir::{
    constraints::{ConstraintRoot, Operation},
    AccessType, AirIR, ConstantBinding, ConstantValueExpr, Identifier, IntegrityConstraintDegree,
    NodeIndex, PeriodicColumn, PublicInput, TraceAccess, Value,
};
use std::collections::btree_map::{BTreeMap, Entry};
use std::mem::{replace, take};
use std::ops::{Bound, RangeBounds};

use processor::math::{Felt, FieldElement, StarkField};
use winter_prover::math::fft;

use crate::config::CodegenConfig;
use crate::writer::Writer;

pub struct CodeGenerator<'ast> {
    /// Miden Assembly writer.
    ///
    /// Track indentation level, and performs basic validations for generated instructions and
    /// closing of blocks.
    writer: Writer,

    /// Counts how many periodic columns have been visited so far.
    ///
    /// Periodic columns are visited in order, and the counter is the same as the columns ID.
    periodic_column: u32,

    /// A list of the periodic lengths in decreasing order.
    ///
    /// The index in this vector corresponds to the offset of the pre-computed z value.
    periods: Vec<usize>,

    /// Counts how many composition coefficients have been used so far, used to compute the correct
    /// offset in memory.
    composition_coefficient_count: u32,

    /// Counts how many integrity constraint roots have been visited so far, used for
    /// emitting documentation.
    integrity_contraints: usize,

    /// Counts how many boundary constraint roots have been visited so far, used for
    /// emitting documentation.
    boundary_contraints: usize,

    /// Maps the public input to their start offset.
    public_input_to_offset: BTreeMap<String, usize>,

    /// Map of the constants found while visitint the [AirIR].
    ///
    /// These values are later used to emit immediate values.
    constants: BTreeMap<&'ast Identifier, &'ast ConstantValueExpr>,

    /// The [AirIR] to visit.
    ir: &'ast AirIR,

    /// Configuration for the codegen.
    config: CodegenConfig,
}

/// Given a periodic column group position, returns a memory offset.
///
/// Periodic columns are grouped based on their length, this is done so that only a single z value
/// needs to be cached per group. The grouping is based on unique lengths, sorted from highest to
/// lowest. Given a periodic group, this function will return a memory offset, which can be used to
/// load the correspoding z value.
fn periodic_group_to_memory_offset(group: u32) -> u32 {
    // Each memory address contains a single quadraic element, this makes the code to store/load the
    // data more efficient, since it is easier to push/pop the high values of a word. So below we
    // have to multiply the group by 2, to account for the zero padding, and add 1, to account for
    // the data being at the low and not high part of the word.
    group * 2 + 1
}

/// Loads the `element` from a memory range starting at `base_addr`.
///
/// This function is used to load a qudratic element from memory, and discard the other value. Even
/// values are store in higher half of the word, while odd values are stored in the lower half.
fn load_quadratic_element(
    writer: &mut Writer,
    base_addr: u32,
    element: u32,
) -> Result<(), CodegenError> {
    let target_word: u32 = element / 2;
    let address = base_addr + target_word;

    // Load data from memory
    writer.padw();
    writer.mem_loadw(address);

    // Discard the other value
    match element % 2 {
        0 => {
            writer.movdn(3);
            writer.movdn(3);
            writer.drop();
            writer.drop();
        }
        1 => {
            writer.drop();
            writer.drop();
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Precomputes the exemption points for a given power.
///
/// The current version of the generated code has hardcoded 2 exemption points, this means the
/// points can be precomputed during compilation time to save a few cycles during runtime.
fn points_for_power(power: u32) -> (u64, u64) {
    let g = Felt::get_root_of_unity(power);
    let trace_len = 2u64.pow(power);
    let one = g.exp(trace_len - 1).as_int();
    let two = g.exp(trace_len - 2).as_int();
    (one, two)
}

/// Generate code to push the exemptions point to the top of the stack.
///
/// This procedure handles two power using conditional drops, instead of control flow with if
/// statements, since the former is slightly faster for the small number of instructions used. The
/// emitted code assumes the trace_length is at the top of the stack, and afer executing it will
/// leave leave the stack as follows:
///
/// Stack: [g^{trace_len-2}, g^{trace_len-1}, ...]
fn exemption_points(writer: &mut Writer, small_power: u32) {
    let (lone, ltwo) = points_for_power(small_power);
    let (hone, htwo) = points_for_power(small_power + 1);

    writer.push(2u64.pow(small_power));
    writer.u32checked_and();
    writer.neq(0);
    writer.comment(format!(
        "Test if trace length is a power of 2^{}",
        small_power
    ));

    writer.push(hone);
    writer.push(lone);
    writer.dup(2);
    writer.cdrop();

    writer.push(htwo);
    writer.push(ltwo);
    writer.movup(3);
    writer.cdrop();
}

/// Helper function to emit efficient code to bisect the trace length value.
///
/// The callbacks `yes` and `no` are used to emit the code for each branch.
fn bisect_trace_len<L, R>(writer: &mut Writer, range: impl RangeBounds<u32>, yes: L, no: R)
where
    L: FnOnce(&mut Writer),
    R: FnOnce(&mut Writer),
{
    let mask = match (range.end_bound(), range.start_bound()) {
        (Bound::Included(&start), Bound::Included(&end)) => {
            let high_mask = 2u64.pow(start + 1) - 1;
            let low_mask = 2u64.pow(end) - 1;
            high_mask ^ low_mask
        }
        _ => panic!("Only inclusive ranges are supported"),
    };

    writer.dup(0);
    writer.push(mask);
    writer.u32checked_and();
    writer.neq(0);
    writer.comment(format!(
        "{:?}..{:?}",
        range.start_bound(),
        range.end_bound()
    ));

    writer.r#if();
    yes(writer);
    writer.r#else();
    no(writer);
    writer.r#end();
}

/// Assumes a quadratic element is at the top of the stack and square it `n` times.
fn quadratic_element_square(writer: &mut Writer, n: u32) {
    for _ in 0..n {
        writer.dup(1);
        writer.dup(1);
        writer.ext2mul();
    }
}

impl<'ast> CodeGenerator<'ast> {
    pub fn new(ir: &'ast AirIR, config: CodegenConfig) -> CodeGenerator<'ast> {
        // remove duplicates and sort period lengths in descending order, since larger periods will
        // give smaller number of cycles (which means a smaller number of exponentiations)
        let mut periods: Vec<usize> = ir.periodic_columns().iter().map(|e| e.len()).collect();
        periods.sort();
        periods.dedup();
        periods.reverse();

        // map from public input name to start offset, used to compute the memory location of the
        // public inputs
        let public_input_to_offset = ir
            .public_inputs()
            .iter()
            .scan(0, |public_input_count, input| {
                let start_offset = *public_input_count;
                *public_input_count += input.1;
                Some((input.0.clone(), start_offset))
            })
            .collect();

        // create a map for constants lookups
        let constants = ir
            .constants()
            .iter()
            .map(|e| (e.name(), e.value()))
            .collect();

        CodeGenerator {
            writer: Writer::new(),
            periodic_column: 0,
            periods,
            composition_coefficient_count: 0,
            integrity_contraints: 0,
            boundary_contraints: 0,
            public_input_to_offset,
            constants,
            ir,
            config,
        }
    }

    /// Emits the Miden Assembly code  after visiting the [AirIR].
    pub fn generate(mut self) -> Result<String, CodegenError> {
        self.visit_air()?;
        Ok(self.writer.into_code())
    }

    /// Emits code for the procedure `cache_z_exp`.
    ///
    /// The procedure computes and caches the necessary exponentiation of `z`. These values are
    /// later on used to evaluate each periodic column polynomial and the constraint divisor.
    ///
    /// This procedure exists because the VM doesn't have native instructions for exponentiation of
    /// quadratic extension elements, and this is an expensive operation.
    ///
    /// The generated code is optimized to perform the fewest number of exponentiations, this is
    /// achieved by observing that periodic columns and trace length are both powers-of-two, since
    /// the exponent is defined as `exp = trace_len / periodic_column_len`, all exponents are
    /// themselves powers-of-two. This allows the results to be computed from smallest to largest,
    /// re-using the intermediary values.
    fn gen_cache_z_exp(&mut self) -> Result<(), CodegenError> {
        // NOTE:
        // - the trace length is a power-of-two.
        //   Ref: https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/random_coin.masm#L82-L87
        // - the periodic columns are powers-of-two.
        //   Ref: https://github.com/0xPolygonMiden/air-script/blob/next/ir/src/symbol_table/mod.rs#L305-L309
        // - the trace length is always greater-than-or-equal the periodic column length.
        //   Ref: https://github.com/facebook/winterfell/blob/main/air/src/air/mod.rs#L322-L326

        self.writer.proc("cache_z_exp");

        self.load_z();
        self.writer.header("=> [z_1, z_0, ...]");

        // The loop below needs to mutable borrow the codegen, so take the field for the iteration
        // (must reset value after the loop).
        let periods = take(&mut self.periods);

        // Emit code to precompute the exponentiation of z for the periodic columns.
        let mut previous_period_size: Option<u64> = None;
        for (idx, period) in periods.iter().enumerate() {
            assert!(
                period.is_power_of_two(),
                "The length of a periodic column must be a power-of-two"
            );

            match previous_period_size {
                None => {
                    self.writer.header(format!(
                        "Number of exponentiations for a period of length {}",
                        period
                    ));

                    // This procedure caches the result of `z.exp(trace_len / period_size)`. Note
                    // that `trace_len = 2^x` and `period_len = 2^y`, so the result of the division
                    // is the same as `2^(x - y)`, the code below computes `x-y` because both
                    // values are in log2 form.
                    //
                    // The result is the number of times that `z` needs to be squared. The
                    // instructions below result in a negative value, as `add.1` is optimized in
                    // the VM (IOW, counting up is faster than counting down).
                    self.load_log2_trace_len();
                    self.writer.neg();
                    self.writer.add(period.ilog2().into());
                }
                Some(prev) => {
                    self.writer.header(format!(
                        "Number of exponentiations to bring the exp from previous {} to {}",
                        prev, *period,
                    ));

                    // The previous iteration computed `log2(trace_len) - log2(prev_period_size)`,
                    // this iteration will compute `log2(trace_len) - log2(period_size)`. The goal
                    // is to reuse the previous value as a cache, so only compute the difference of
                    // the two values which is just `log2(prev_period_size) - log2(period_size)`.
                    let prev = Felt::new(prev);
                    let new = Felt::new(period.ilog2().into());
                    let diff = prev - new; // this is a negative value
                    self.writer.push(diff.as_int());
                }
            }
            self.writer.header("=> [count, z_1, z_0, ...]");

            self.writer.header("Exponentiate z");

            // The trace length and the period may have the same size, so it is necessary to perform
            // the test before entering the loop.
            self.writer.dup(0);
            self.writer.neq(0);

            self.writer.r#while();
            self.writer.movdn(2);
            self.writer.dup(1);
            self.writer.dup(1);
            self.writer.ext2mul();
            self.writer.header("=> [z_1^2, z_0^2, i, ...]");

            self.writer.movup(2);
            self.writer.add(1);
            self.writer.dup(0);
            self.writer.neq(0);
            self.writer.header("=> [b, i+1, z_1^2, z_0^2, ...]");
            self.writer.end();

            let idx: u32 = idx.try_into().expect("periodic column length is too large");
            let addr = self.config.z_exp_address + idx;
            self.writer.push(0);
            self.writer.mem_storew(addr);
            self.writer.comment(format!("z^exp for period {}", *period));

            self.writer.header("=> [0, 0, z_1^2, z_0^2, ...]");
            self.writer.drop();
            self.writer.drop();

            previous_period_size = Some((*period).try_into().expect("diff must fit in a u64"));
        }

        // Re-set the periods now that the loop is over
        let _ = replace(&mut self.periods, periods);

        // Emit code to precompute the exponentiation of z for the divisor.
        match previous_period_size {
            None => {
                self.writer.header("Exponentiate z trace_len times");
                self.load_log2_trace_len();
                self.writer.neg();
            }
            Some(prev) => {
                self.writer
                    .header(format!("Exponentiate z {} times, until trace_len", prev));
                let prev = Felt::new(prev);
                let neg_prev = -prev;
                self.writer.push(neg_prev.as_int());
            }
        }

        // The trace length and the period may have the same size, so it is necessary to perform
        // the test before entering the loop.
        self.writer.dup(0);
        self.writer.neq(0);

        self.writer.r#while();
        self.writer.movdn(2);
        self.writer.dup(1);
        self.writer.dup(1);
        self.writer.ext2mul();
        self.writer.header("=> [z_1^2, z_0^2, i, ...]");

        self.writer.movup(2);
        self.writer.add(1);
        self.writer.dup(0);
        self.writer.neq(0);
        self.writer.header("=> [b, i+1, z_1^2, z_0^2, ...]");
        self.writer.end();

        let idx: u32 = self
            .periods
            .len()
            .try_into()
            .expect("periodic column length is too large");
        let addr = self.config.z_exp_address + idx;
        self.writer.push(0);
        self.writer.mem_storew(addr);
        self.writer.comment("z^exp for trace_len");

        self.writer.header("=> [0, 0, z_1^2, z_0^2, ...]");
        self.writer.drop();
        self.writer.drop();

        self.writer
            .header("=> [z_1^2, z_0^2, num_of_cycles', -log2(trace_len), ...]");
        self.writer.comment("Clean stack");
        self.writer.dropw();

        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `cache_periodic_polys`.
    ///
    /// This procedure first computes the `z**exp` for each periodic column, and then evaluates
    /// each periodic polynomial using Horner's method. The results are cached to memory.
    fn gen_evaluate_periodic_polys(&mut self) -> Result<(), CodegenError> {
        self.writer.proc("cache_periodic_polys");
        walk_periodic_columns(self, self.ir)?;
        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `compute_evaluate_integrity_constraints`.
    ///
    /// This procedure evaluates each top-level integrity constraint and leaves the result on the
    /// stack. This is useful for testing the evaluation. Later on the value is aggregated.
    fn gen_compute_evaluate_integrity_constraints(&mut self) -> Result<(), CodegenError> {
        self.writer.proc("compute_evaluate_integrity_constraints");
        walk_integrity_constraints(self, self.ir, MAIN_TRACE)?;
        self.integrity_contraints = 0; // reset counter for the aux trace
        walk_integrity_constraints(self, self.ir, AUX_TRACE)?;
        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `compute_evaluate_boundary_constraints`.
    fn gen_compute_evaluate_boundary_constraints(&mut self) -> Result<(), CodegenError> {
        self.writer.proc("compute_evaluate_boundary_constraints");
        walk_boundary_constraints(self, self.ir, MAIN_TRACE)?;
        self.boundary_contraints = 0; // reset counter for the aux trace
        walk_boundary_constraints(self, self.ir, AUX_TRACE)?;
        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `get_exemptions_points`.
    ///
    /// The generated procedure contains the precomputed exeption points to be used when computing
    /// the divisors. The values are returned in the stack.
    fn gen_get_exemptions_points(&mut self) -> Result<(), CodegenError> {
        // Notes:
        // - Computing the exemption points on the fly would require 1 exponentiation to find the
        // root-of-unity from the two-adacity, followed by another exponetiation to compute the
        // two-to-last exemption point, and a multiplication to compute the last exemption point.
        // Each exponentiation is 41 cycles, giving around 83 cycles to compute the values.
        // - For the range from powers 3 to 32 there are 30 unique values, which requires 8 words
        // of data. Storing the data to memory requires pushing the 4 elements of a word to the
        // stack, the target address, the store, and cleaning the stack, resulting in 10 cycles per
        // word for a total of 80 cycles and some additional cycles to load the right value from
        // memory when needed.
        // - The code below instead uses a binary search to find the right value. And push only the
        // necessary data to memory, in 62/73 cycles.
        // - The smallest trace length is 2^3,
        // Ref: https://github.com/facebook/winterfell/blob/main/air/src/air/trace_info.rs#L34-L35
        // - The trace length is guaranteed to be a power-of-two and to fit in a u32.
        // Ref: https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/random_coin.masm#L76

        self.writer.proc("get_exemptions_points");
        self.load_trace_len();
        self.writer.header("=> [trace_len, ...]");

        let writer = &mut self.writer;
        bisect_trace_len(
            writer,
            3..=16,
            |writer: &mut Writer| {
                bisect_trace_len(
                    writer,
                    3..=10,
                    |writer: &mut Writer| {
                        bisect_trace_len(
                            writer,
                            3..=6,
                            |writer: &mut Writer| {
                                bisect_trace_len(
                                    writer,
                                    3..=4,
                                    |writer: &mut Writer| exemption_points(writer, 3),
                                    |writer: &mut Writer| exemption_points(writer, 5),
                                )
                            },
                            |writer: &mut Writer| {
                                bisect_trace_len(
                                    writer,
                                    7..=8,
                                    |writer: &mut Writer| exemption_points(writer, 7),
                                    |writer: &mut Writer| exemption_points(writer, 9),
                                )
                            },
                        )
                    },
                    |writer: &mut Writer| {
                        bisect_trace_len(
                            writer,
                            11..=14,
                            |writer: &mut Writer| {
                                bisect_trace_len(
                                    writer,
                                    11..=12,
                                    |writer: &mut Writer| exemption_points(writer, 11),
                                    |writer: &mut Writer| exemption_points(writer, 13),
                                )
                            },
                            |writer: &mut Writer| exemption_points(writer, 15),
                        )
                    },
                );
            },
            |writer: &mut Writer| {
                bisect_trace_len(
                    writer,
                    17..=24,
                    |writer: &mut Writer| {
                        bisect_trace_len(
                            writer,
                            17..=20,
                            |writer: &mut Writer| {
                                bisect_trace_len(
                                    writer,
                                    17..=18,
                                    |writer: &mut Writer| exemption_points(writer, 17),
                                    |writer: &mut Writer| exemption_points(writer, 19),
                                )
                            },
                            |writer: &mut Writer| {
                                bisect_trace_len(
                                    writer,
                                    21..=22,
                                    |writer: &mut Writer| exemption_points(writer, 21),
                                    |writer: &mut Writer| exemption_points(writer, 23),
                                )
                            },
                        )
                    },
                    |writer: &mut Writer| {
                        bisect_trace_len(
                            writer,
                            25..=28,
                            |writer: &mut Writer| {
                                bisect_trace_len(
                                    writer,
                                    25..=26,
                                    |writer: &mut Writer| exemption_points(writer, 25),
                                    |writer: &mut Writer| exemption_points(writer, 27),
                                )
                            },
                            |writer: &mut Writer| {
                                bisect_trace_len(
                                    writer,
                                    29..=30,
                                    |writer: &mut Writer| exemption_points(writer, 29),
                                    |writer: &mut Writer| {
                                        let (one, two) = points_for_power(31);
                                        writer.push(one);
                                        writer.push(two);
                                    },
                                )
                            },
                        )
                    },
                );
            },
        );

        self.writer.end(); // end proc

        Ok(())
    }

    /// Emits code for the procedure `evaluate_integrity_constraints`.
    ///
    /// Evaluates the integrity constraints for both the main and auxiliary traces.
    fn gen_evaluate_integrity_constraints(&mut self) -> Result<(), CodegenError> {
        self.writer.proc("evaluate_integrity_constraints");

        if !self.ir.periodic_columns().is_empty() {
            self.writer.exec("cache_periodic_polys");
        }

        self.writer.exec("compute_evaluate_integrity_constraints");

        self.writer
            .header("Accumulate the numerator of the constraint polynomial");

        let total_len = self.ir.integrity_constraints(MAIN_TRACE).len()
            + self.ir.integrity_constraints(AUX_TRACE).len();

        for _ in 0..total_len {
            self.writer.ext2add();
        }

        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `evaluate_boundary_constraints`.
    ///
    /// Evaluates the boundary constraints for both the main and auxiliary traces.
    fn gen_evaluate_boundary_constraints(&mut self) -> Result<(), CodegenError> {
        self.writer.proc("evaluate_boundary_constraints");
        self.writer.exec("compute_evaluate_boundary_constraints");

        self.writer
            .header("Accumulate the numerator of the constraint polynomial");

        let total_len = self.ir.boundary_constraints(MAIN_TRACE).len()
            + self.ir.boundary_constraints(AUX_TRACE).len();

        for _ in 0..total_len {
            self.writer.ext2add();
        }

        self.writer.end();

        Ok(())
    }

    /// Emits code to load the `log_2(trace_len)` onto the top of the stack.
    fn load_trace_len(&mut self) {
        self.writer.mem_load(self.config.trace_len_address);
    }

    /// Emits code to load the `log_2(trace_len)` onto the top of the stack.
    fn load_log2_trace_len(&mut self) {
        self.writer.mem_load(self.config.log2_trace_len_address);
    }

    /// Emits code to load `z` onto the top of the stack.
    fn load_z(&mut self) {
        self.writer.padw();
        self.writer.mem_loadw(self.config.z_address);
        self.writer.drop();
        self.writer.drop();
    }
}

#[derive(Debug)]
pub enum CodegenError {
    DuplicatedConstant,
    InvalidAccessType,
    UnknownConstant,
    InvalidRowOffset,
    InvalidSize,
    InvalidIndex,
    InvalidBoundaryConstraint,
    InvalidIntegrityConstraint,
}

impl<'ast> AirVisitor<'ast> for CodeGenerator<'ast> {
    type Value = ();
    type Error = CodegenError;

    fn visit_access_type(&mut self, _access: &'ast AccessType) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_boundary_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: u8,
    ) -> Result<Self::Value, Self::Error> {
        if !constraint.domain().is_boundary() {
            return Err(CodegenError::InvalidBoundaryConstraint);
        }

        let segment = if trace_segment == MAIN_TRACE {
            "main"
        } else {
            "aux"
        };

        self.writer.header(format!(
            "boundary constraint {} for {}",
            self.boundary_contraints, segment
        ));

        // Note: AirScript's boundary constraints are only defined for the first or last row.
        // Meaning they are implemented as an assertion for a single element. Visiting the
        // [NodeIndex] will emit code to compute the difference of the expected value and the
        // evaluation frame value.
        self.visit_node_index(constraint.node_index())?;

        self.writer
            .header("Multiply by the composition coefficient");

        load_quadratic_element(
            &mut self.writer,
            self.config.composition_coef_address,
            self.composition_coefficient_count,
        )?;
        self.writer.ext2mul();
        self.composition_coefficient_count += 1;

        self.boundary_contraints += 1;
        Ok(())
    }

    fn visit_air(&mut self) -> Result<Self::Value, Self::Error> {
        walk_constant_bindings(self, self.ir)?;
        walk_public_inputs(self, self.ir)?;
        walk_integrity_constraint_degrees(self, self.ir, MAIN_TRACE)?;
        walk_integrity_constraint_degrees(self, self.ir, AUX_TRACE)?;

        self.gen_get_exemptions_points()?;
        self.gen_cache_z_exp()?;
        if !self.ir.periodic_columns().is_empty() {
            self.gen_evaluate_periodic_polys()?;
        }
        self.gen_compute_evaluate_integrity_constraints()?;
        self.gen_compute_evaluate_boundary_constraints()?;
        self.gen_evaluate_integrity_constraints()?;
        self.gen_evaluate_boundary_constraints()?;

        Ok(())
    }

    fn visit_constant_binding(
        &mut self,
        _constant: &'ast ConstantBinding,
    ) -> Result<Self::Value, Self::Error> {
        Ok(())
    }

    fn visit_integrity_constraint_degree(
        &mut self,
        _constraint: IntegrityConstraintDegree,
        _trace_segment: u8,
    ) -> Result<Self::Value, Self::Error> {
        Ok(()) // TODO
    }

    fn visit_integrity_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: u8,
    ) -> Result<Self::Value, Self::Error> {
        if !constraint.domain().is_integrity() {
            return Err(CodegenError::InvalidIntegrityConstraint);
        }

        let segment = if trace_segment == MAIN_TRACE {
            "main"
        } else {
            "aux"
        };

        self.writer.header(format!(
            "integrity constraint {} for {}",
            self.integrity_contraints, segment
        ));

        self.visit_node_index(constraint.node_index())?;

        self.writer
            .header("Multiply by the composition coefficient");

        load_quadratic_element(
            &mut self.writer,
            self.config.composition_coef_address,
            self.composition_coefficient_count,
        )?;
        self.writer.ext2mul();
        self.composition_coefficient_count += 1;

        self.integrity_contraints += 1;
        Ok(())
    }

    fn visit_node_index(
        &mut self,
        node_index: &'ast NodeIndex,
    ) -> Result<Self::Value, Self::Error> {
        let op = self.ir.constraint_graph().node(node_index).op();
        self.visit_operation(op)
    }

    fn visit_operation(&mut self, op: &'ast Operation) -> Result<Self::Value, Self::Error> {
        match op {
            Operation::Value(value) => {
                self.visit_value(value)?;
            }
            Operation::Add(left, right) => {
                self.visit_node_index(left)?;
                self.visit_node_index(right)?;
                self.writer.ext2add();
            }
            Operation::Sub(left, right) => {
                self.visit_node_index(left)?;
                self.visit_node_index(right)?;
                self.writer.ext2sub();
            }
            Operation::Mul(left, right) => {
                self.visit_node_index(left)?;
                self.visit_node_index(right)?;
                self.writer.ext2mul();
            }
            Operation::Exp(left, exp) => {
                // NOTE: The VM doesn't support exponentiation of extension elements.
                //
                // Ref: https://github.com/facebook/winterfell/blob/0acb2a148e2e8445d5f6a3511fa9d852e54818dd/math/src/field/traits.rs#L124-L150

                self.visit_node_index(left)?;

                self.writer.header("push the accumulator to the stack");
                self.writer.push(1);
                self.writer.movdn(2);
                self.writer.push(0);
                self.writer.movdn(2);
                self.writer.header("=> [b1, b0, r1, r0, ...]");

                // emitted code computes exponentiation via square-and-multiply
                let mut e: usize = *exp;
                while e != 0 {
                    self.writer
                        .header(format!("square {} times", e.trailing_zeros()));
                    quadratic_element_square(&mut self.writer, e.trailing_zeros());

                    // account for the exponentiations done above
                    e = e >> e.trailing_zeros();

                    self.writer.header("multiply");
                    self.writer.dup(1);
                    self.writer.dup(1);
                    self.writer.movdn(5);
                    self.writer.movdn(5);
                    self.writer
                        .header("=> [b1, b0, r1, r0, b1, b0, ...] (4 cycles)");

                    self.writer.ext2mul();
                    self.writer.movdn(3);
                    self.writer.movdn(3);
                    self.writer.header("=> [b1, b0, r1', r0', ...] (5 cycles)");

                    // account for the multiply done above
                    assert!(
                        e & 1 == 1,
                        "this loop is only executed if the number is non-zero"
                    );
                    e ^= 1;
                }

                self.writer.header("clean stack");
                self.writer.drop();
                self.writer.drop();
                self.writer.header("=> [r1, r0, ...] (2 cycles)");
            }
        };

        Ok(())
    }

    fn visit_periodic_column(
        &mut self,
        column: &'ast PeriodicColumn,
    ) -> Result<Self::Value, Self::Error> {
        // convert the periodic column to a polynomial
        let inv_twiddles = fft::get_inv_twiddles::<Felt>(column.len());
        let mut poly: Vec<Felt> = column.iter().map(|e| Felt::new(*e)).collect();
        fft::interpolate_poly(&mut poly, &inv_twiddles);

        self.writer
            .comment(format!("periodic column {}", self.periodic_column));

        // LOAD OOD ELEMENT
        // ---------------------------------------------------------------------------------------

        // assumes that cache_z_exp has been called before, which precomputes the value of z**exp
        let group: u32 = self
            .periods
            .iter()
            .position(|&p| p == column.len())
            .expect("All periods are added in the constructor")
            .try_into()
            .expect("periods are u32");
        load_quadratic_element(
            &mut self.writer,
            self.config.z_exp_address,
            periodic_group_to_memory_offset(group),
        )?;
        self.writer.header("=> [z_exp_1, z_exp_0, ...]");

        // EVALUATE PERIODIC POLY
        // ---------------------------------------------------------------------------------------

        // convert coefficients from Montgomery form (Masm uses plain integers).
        let coef: Vec<u64> = poly.iter().map(|e| e.as_int()).collect();

        // periodic columns have at least 2 values, push the first as the accumulator
        self.writer.push(coef[0]);
        self.writer.push(0);
        self.writer.header("=> [a_1, a_0, z_exp_1, z_exp_0, ...]");

        // Evaluate the periodic polynomial at point z**exp using Horner's algorithm
        for c in coef.iter().skip(1) {
            self.writer.header("duplicate z_exp");
            self.writer.dup(3);
            self.writer.dup(3);
            self.writer
                .header("=> [z_exp_1, z_exp_0, a_1, a_0, z_exp_1, z_exp_0, ...]");

            self.writer.ext2mul();
            self.writer.push(*c);
            self.writer.push(0);
            self.writer.ext2add();
            self.writer.header("=> [a_1, a_0, z_exp_1, z_exp_0, ...]");
        }

        self.writer.header("Clean z_exp from the stack");
        self.writer.movup(3);
        self.writer.movup(3);
        self.writer.drop();
        self.writer.drop();
        self.writer.header("=> [a_1, a_0, ...]");

        self.writer.header(
            "Save the evaluation of the periodic polynomial at point z**exp, and clean stack",
        );
        let addr = self.config.periodic_values_address + self.periodic_column;
        self.writer.push(0);
        self.writer.push(0);
        self.writer.mem_storew(addr);
        self.writer.dropw();

        self.periodic_column += 1;
        Ok(())
    }

    fn visit_public_input(
        &mut self,
        _constant: &'ast PublicInput,
    ) -> Result<Self::Value, Self::Error> {
        Ok(())
    }

    fn visit_trace_access(
        &mut self,
        _trace_access: &'ast TraceAccess,
    ) -> Result<Self::Value, Self::Error> {
        Ok(()) // TODO
    }

    fn visit_value(&mut self, value: &'ast Value) -> Result<Self::Value, Self::Error> {
        match value {
            Value::BoundConstant(symbol) => match self.constants.entry(symbol.ident()) {
                Entry::Occupied(entry) => match (entry.get(), symbol.access_type()) {
                    (ConstantValueExpr::Scalar(scalar), AccessType::Default) => {
                        self.writer.push(*scalar);
                        self.writer.push(0);
                    }
                    (ConstantValueExpr::Vector(vec), AccessType::Vector(pos)) => {
                        let scalar = vec.get(*pos).ok_or(CodegenError::InvalidAccessType)?;
                        self.writer.push(*scalar);
                        self.writer.push(0);
                    }
                    (ConstantValueExpr::Matrix(mat), AccessType::Matrix(x, y)) => {
                        let scalar = mat
                            .get(*x)
                            .and_then(|v| v.get(*y))
                            .ok_or(CodegenError::InvalidAccessType)?;
                        self.writer.push(*scalar);
                        self.writer.push(0);
                    }
                    _ => return Err(CodegenError::InvalidAccessType),
                },
                Entry::Vacant(_) => return Err(CodegenError::UnknownConstant),
            },
            Value::InlineConstant(value) => {
                self.writer.push(*value);
                self.writer.push(0);
            }
            Value::TraceElement(access) => {
                // eventually larger offsets will be supported
                if access.row_offset() > 1 {
                    return Err(CodegenError::InvalidRowOffset);
                }

                // should always be one
                if access.size() != 1 {
                    return Err(CodegenError::InvalidSize);
                }

                // Compute the target address for this variable. Each memory address contains the
                // curr and next values of a single variable.
                //
                // Layout defined at: https://github.com/0xPolygonMiden/miden-vm/issues/875
                let target_word: u32 = access
                    .col_idx()
                    .try_into()
                    .map_err(|_| CodegenError::InvalidIndex)?;
                let el_pos: u32 = access
                    .row_offset()
                    .try_into()
                    .or(Err(CodegenError::InvalidIndex))?;
                let target_element = target_word * 2 + el_pos;

                let base_address = if access.trace_segment() == MAIN_TRACE {
                    self.config.ood_frame_address
                } else {
                    self.config.ood_aux_frame_address
                };

                load_quadratic_element(&mut self.writer, base_address, target_element)?;
            }
            Value::PeriodicColumn(_, length) => {
                let group: u32 = self
                    .periods
                    .iter()
                    .position(|&p| p == *length)
                    .expect("All periods are added in the constructor")
                    .try_into()
                    .expect("periods are u32");
                load_quadratic_element(
                    &mut self.writer,
                    self.config.periodic_values_address,
                    periodic_group_to_memory_offset(group),
                )?;
            }
            Value::PublicInput(name, index) => {
                let start_offset = self
                    .public_input_to_offset
                    .get(name)
                    .unwrap_or_else(|| panic!("public input {} unknown", name));

                self.writer.header(format!(
                    "Load public input {} pos {} with final offset {}",
                    name, index, start_offset,
                ));
                let index: u32 = (start_offset + *index)
                    .try_into()
                    .or(Err(CodegenError::InvalidIndex))?;
                load_quadratic_element(&mut self.writer, self.config.public_inputs_address, index)?;
            }
            Value::RandomValue(element) => {
                // Compute the target address for the random value. Each memory address contains
                // two values.
                //
                // Layout defined at: https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/random_coin.masm#L169-L172
                load_quadratic_element(
                    &mut self.writer,
                    self.config.aux_rand_address,
                    (*element).try_into().or(Err(CodegenError::InvalidIndex))?,
                )?;
            }
        };

        Ok(())
    }
}
