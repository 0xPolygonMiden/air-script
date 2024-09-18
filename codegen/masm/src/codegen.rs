use crate::config::CodegenConfig;
use crate::constants::{AUX_TRACE, MAIN_TRACE};
use crate::error::CodegenError;
use crate::utils::{
    boundary_group_to_procedure_name, load_quadratic_element, periodic_group_to_memory_offset,
};
use crate::visitor::{
    walk_boundary_constraints, walk_integrity_constraints, walk_periodic_columns, AirVisitor,
};
use crate::writer::Writer;
use air_ir::{
    Air, ConstraintDomain, ConstraintRoot, Identifier, NodeIndex, Operation, PeriodicColumn,
    TraceSegmentId, Value,
};
use miden_core::{Felt, StarkField};
use std::collections::btree_map::BTreeMap;
use std::mem::{replace, take};
use winter_math::fft;

#[derive(Default)]
pub struct CodeGenerator {
    config: CodegenConfig,
}
impl CodeGenerator {
    pub fn new(config: CodegenConfig) -> Self {
        Self { config }
    }
}
impl air_ir::CodeGenerator for CodeGenerator {
    type Output = String;

    fn generate(&self, ir: &Air) -> anyhow::Result<Self::Output> {
        let generator = Backend::new(ir, self.config);
        generator.generate()
    }
}

struct Backend<'ast> {
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
    /// offset in memory. This counter is shared among integrity and boundary constraints for both
    /// the main and auxiliary traces.
    composition_coefficient_count: u32,

    /// Counts how many integrity constraint roots have been visited so far, used for
    /// emitting documentation.
    integrity_contraints: usize,

    /// Counts how many boundary constraint roots have been visited so far, used for
    /// emitting documentation.
    boundary_contraints: usize,

    /// Counts the size of a given boundary constraint category. The counter is used to emit the
    /// correct number of multiplications for a given divisor.
    boundary_constraint_count: BTreeMap<(TraceSegmentId, ConstraintDomain), usize>,

    /// Maps the public input to their start offset.
    public_input_to_offset: BTreeMap<Identifier, usize>,

    /// The [Air] to visit.
    ir: &'ast Air,

    /// Configuration for the codegen.
    config: CodegenConfig,
}

impl<'ast> Backend<'ast> {
    fn new(ir: &'ast Air, config: CodegenConfig) -> Self {
        // remove duplicates and sort period lengths in descending order, since larger periods will
        // have smaller number of cycles (which means a smaller number of exponentiations)
        let mut periods: Vec<usize> = ir.periodic_columns().map(|e| e.period()).collect();
        periods.sort();
        periods.dedup();
        periods.reverse();

        // Maps the public input name to its memory offset, were the memory offset is the
        // accumulated number of inputs laid out in memory prior to our target. For example:
        //
        //  Input "a" starts at offset 0
        // |      Input "b" starts at offset 4, after the 4 values of "a"
        // v      v                   Input "c" starts at offset 20, after the values of "a" and "b"
        // [ .... | ................ | ....]
        //
        // The offset is used by the codegen to load public input values.
        let public_input_to_offset = ir
            .public_inputs()
            .scan(0, |public_input_count, input| {
                let start_offset = *public_input_count;
                *public_input_count += input.size;
                Some((input.name, start_offset))
            })
            .collect();

        // count the boundary constraints
        let mut boundary_constraint_count = BTreeMap::new();
        for segment in [MAIN_TRACE, AUX_TRACE] {
            for boundary in ir.boundary_constraints(segment) {
                boundary_constraint_count
                    .entry((segment, boundary.domain()))
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }

        Self {
            writer: Writer::new(),
            periodic_column: 0,
            periods,
            composition_coefficient_count: 0,
            integrity_contraints: 0,
            boundary_contraints: 0,
            boundary_constraint_count,
            public_input_to_offset,
            ir,
            config,
        }
    }

    /// Emits the Miden Assembly code  after visiting the [AirIR].
    fn generate(mut self) -> anyhow::Result<String> {
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

        self.writer
            .header("Procedure to efficiently compute the required exponentiations of the out-of-domain point `z` and cache them for later use.");
        self.writer.header("");
        self.writer.header("This computes the power of `z` needed to evaluate the periodic polynomials and the constraint divisors");
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [...]");

        self.writer.proc("cache_z_exp");

        self.load_z();
        self.writer.header("=> [z_1, z_0, ...]");

        // The loop below needs to mutably borrow the codegen, so take the field for the iteration
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
                        "Find number exponentiations required to get for a period of length {}",
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
                    self.writer.header(format!(
                        "=> [count, z_1, z_0, ...] where count = -log2(trace_len) + {}",
                        period.ilog2()
                    ));
                }
                Some(prev) => {
                    self.writer.header(format!(
                        "Find number of exponentiations to bring from length {} to {}",
                        prev, *period,
                    ));

                    // The previous iteration computed `log2(trace_len) - log2(prev_period_size)`,
                    // this iteration will compute `log2(trace_len) - log2(period_size)`. The goal
                    // is to reuse the previous value as a cache, so only compute the difference of
                    // the two values which is just `log2(prev_period_size) - log2(period_size)`.
                    let prev = Felt::new(prev.ilog2().into());
                    let new = Felt::new(period.ilog2().into());
                    let diff = new - prev; // this is a negative value
                    self.writer.push(diff.as_int());
                    self.writer.header(format!(
                        "=> [count, (z_1, z_0)^{}, ...] where count = {} - {}",
                        prev.as_int(),
                        new.as_int(),
                        prev.as_int(),
                    ));
                }
            }

            self.writer.header("Exponentiate z");
            self.writer.ext2_exponentiate();

            let idx: u32 = idx.try_into().expect("periodic column length is too large");
            let addr = self.config.z_exp_address + idx;
            self.writer.push(0);
            self.writer.mem_storew(addr);
            self.writer.comment(format!("z^{}", *period));

            self.writer.header(format!(
                "=> [0, 0, (z_1, z_0)^n, ...] where n = trace_len-{}",
                *period
            ));
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
                self.writer
                    .header("=> [count, z_1, z_0, ...] where count = -log2(trace_len)");
            }
            Some(prev) => {
                self.writer
                    .header(format!("Exponentiate z {} times, until trace_len", prev));
                let prev = Felt::new(prev.ilog2().into());
                let neg_prev = -prev;
                self.writer.push(neg_prev.as_int());
                self.writer.header(format!(
                    "=> [count, (z_1, z_0)^n, ...] where count=-{} , n=trace_len-{}",
                    prev.as_int(),
                    prev.as_int(),
                ));
            }
        }

        self.writer.ext2_exponentiate();

        let idx: u32 = self
            .periods
            .len()
            .try_into()
            .expect("periodic column length is too large");
        let addr = self.config.z_exp_address + idx;
        self.writer.push(0);
        self.writer.mem_storew(addr);
        self.writer.comment("z^trace_len");

        self.writer.header("=> [0, 0, (z_1, z_0)^trace_len, ...]");
        self.writer.dropw();
        self.writer.comment("Clean stack");

        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `cache_periodic_polys`.
    ///
    /// This procedure first computes the `z**exp` for each periodic column, and then evaluates
    /// each periodic polynomial using Horner's method. The results are cached to memory.
    fn gen_evaluate_periodic_polys(&mut self) -> Result<(), CodegenError> {
        self.writer
            .header("Procedure to evaluate the periodic polynomials.");
        self.writer.header("");
        self.writer
            .header("Procedure `cache_z_exp` must have been called prior to this.");
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [...]");

        self.writer.proc("cache_periodic_polys");
        walk_periodic_columns(self, self.ir)?;
        self.writer.end();

        Ok(())
    }

    fn gen_compute_integrity_constraint_divisor(&mut self) -> Result<(), CodegenError> {
        self.writer
            .header("Procedure to compute the integrity constraint divisor.");
        self.writer.header("");
        self.writer.header(
            "The divisor is defined as `(z^trace_len - 1) / ((z - g^{trace_len-2}) * (z - g^{trace_len-1}))`",
        );
        self.writer
            .header("Procedure `cache_z_exp` must have been called prior to this.");
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [divisor_1, divisor_0, ...]");

        self.writer.proc("compute_integrity_constraint_divisor");

        // `z^trace_len` is saved after all the period column points
        let group: u32 = self.periods.len().try_into().expect("periods are u32");
        load_quadratic_element(
            &mut self.writer,
            self.config.z_exp_address,
            periodic_group_to_memory_offset(group),
        )?;
        self.writer.comment("load z^trace_len");

        self.writer.header("Comments below use zt = `z^trace_len`");
        self.writer.header("=> [zt_1, zt_0, ...]");

        // Compute the numerator `z^trace_len - 1`
        self.writer.push(1);
        self.writer.push(0);
        self.writer.ext2sub();
        self.writer.header("=> [zt_1-1, zt_0-1, ...]");

        // Compute the denominator of the divisor
        self.load_z();
        self.writer.header("=> [z_1, z_0, zt_1-1, zt_0-1, ...]");

        self.writer.exec("get_exemptions_points");
        self.writer
            .header("=> [g^{trace_len-2}, g^{trace_len-1}, z_1, z_0, zt_1-1, zt_0-1, ...]");

        self.writer.dup(0);
        self.writer.mem_store(self.config.exemption_two_address);
        self.writer
            .comment("Save a copy of `g^{trace_len-2} to be used by the boundary divisor");

        // Compute `z - g^{trace_len-2}`
        self.writer.dup(3);
        self.writer.dup(3);
        self.writer.movup(3);
        self.writer.push(0);
        self.writer.ext2sub();
        self.writer
            .header("=> [e_1, e_0, g^{trace_len-1}, z_1, z_0, zt_1-1, zt_0-1, ...]");

        // Compute `z - g^{trace_len-1}`
        self.writer.movup(4);
        self.writer.movup(4);
        self.writer.movup(4);
        self.writer.push(0);
        self.writer.ext2sub();
        self.writer
            .header("=> [e_3, e_2, e_1, e_0, zt_1-1, zt_0-1, ...]");

        // Compute the denominator `(z - g^{trace_len-2}) * (z - g^{trace_len-1})`
        self.writer.ext2mul();
        self.writer
            .header("=> [denominator_1, denominator_0, zt_1-1, zt_0-1, ...]");

        // Compute the divisor `(z^trace_len - 1) / ((z - g^{trace_len-2}) * (z - g^{trace_len-1}))`
        self.writer.ext2div();
        self.writer.header("=> [divisor_1, divisor_0, ...]");
        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `compute_integrity_constraints`.
    ///
    /// This procedure evaluates each top-level integrity constraint and leaves the result on the
    /// stack. This is useful for testing the evaluation. Later on the value is aggregated.
    fn gen_compute_integrity_constraints(&mut self) -> Result<(), CodegenError> {
        let main_trace_count = self.ir.integrity_constraints(MAIN_TRACE).len();
        let aux_trace_count = self.ir.integrity_constraints(AUX_TRACE).len();

        self.writer
            .header("Procedure to evaluate numerators of all integrity constraints.");
        self.writer.header("");
        self.writer.header(format!(
            "All the {} main and {} auxiliary constraints are evaluated.",
            main_trace_count, aux_trace_count
        ));
        self.writer.header(
            "The result of each evaluation is kept on the stack, with the top of the stack",
        );
        self.writer.header(
            "containing the evaluations for the auxiliary trace (if any) followed by the main trace.",
        );
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [(r_1, r_0)*, ...]");
        self.writer.header(
            "where: (r_1, r_0) is the quadratic extension element resulting from the integrity constraint evaluation.",
        );
        self.writer.header(format!(
            "       This procedure pushes {} quadratic extension field elements to the stack",
            main_trace_count + aux_trace_count
        ));

        self.writer.proc("compute_integrity_constraints");
        walk_integrity_constraints(self, self.ir, MAIN_TRACE)?;
        self.integrity_contraints = 0; // reset counter for the aux trace
        walk_integrity_constraints(self, self.ir, AUX_TRACE)?;
        self.writer.end();

        Ok(())
    }

    /// Emits procedure to compute boundary constraints values.
    ///
    /// This will emit four procedures:
    ///
    /// - compute_boundary_constraints_main_first
    /// - compute_boundary_constraints_main_last
    /// - compute_boundary_constraints_aux_first
    /// - compute_boundary_constraints_aux_last
    ///
    /// Each procedure corresponds to a specific boundary constraint group. They are emitted
    /// separetely because each value is divided by a different divisor, and it is best to
    /// manipulate each point separetely.
    fn gen_compute_boundary_constraints(&mut self) -> Result<(), CodegenError> {
        // The boundary constraints have a natural order defined as (trace, domain, column_pos).
        // The code below iterates using that order

        if self
            .boundary_constraint_count
            .contains_key(&(MAIN_TRACE, ConstraintDomain::FirstRow))
        {
            let name = boundary_group_to_procedure_name(MAIN_TRACE, ConstraintDomain::FirstRow);
            self.writer.header(
                "Procedure to evaluate the boundary constraint numerator for the first row of the main trace",
            );
            self.writer.header("");
            self.writer.header("Input: [...]");
            self.writer.header("Output: [(r_1, r_0)*, ...]");
            self.writer.header(
                "Where: (r_1, r_0) is one quadratic extension field element for each constraint",
            );
            self.writer.proc(name);
            walk_boundary_constraints(self, self.ir, MAIN_TRACE, ConstraintDomain::FirstRow)?;
            self.writer.end();
        }

        if self
            .boundary_constraint_count
            .contains_key(&(MAIN_TRACE, ConstraintDomain::LastRow))
        {
            let name = boundary_group_to_procedure_name(MAIN_TRACE, ConstraintDomain::LastRow);
            self.writer.header(
                "Procedure to evaluate the boundary constraint numerator for the last row of the main trace",
            );
            self.writer.header("");
            self.writer.header("Input: [...]");
            self.writer.header("Output: [(r_1, r_0)*, ...]");
            self.writer.header(
                "Where: (r_1, r_0) is one quadratic extension field element for each constraint",
            );
            self.writer.proc(name);
            walk_boundary_constraints(self, self.ir, MAIN_TRACE, ConstraintDomain::LastRow)?;
            self.writer.end();
        }

        if self
            .boundary_constraint_count
            .contains_key(&(AUX_TRACE, ConstraintDomain::FirstRow))
        {
            let name = boundary_group_to_procedure_name(AUX_TRACE, ConstraintDomain::FirstRow);
            self.writer.header(
            "Procedure to evaluate the boundary constraint numerator for the first row of the auxiliary trace",
        );
            self.writer.header("");
            self.writer.header("Input: [...]");
            self.writer.header("Output: [(r_1, r_0)*, ...]");
            self.writer.header(
                "Where: (r_1, r_0) is one quadratic extension field element for each constraint",
            );
            self.writer.proc(name);
            walk_boundary_constraints(self, self.ir, AUX_TRACE, ConstraintDomain::FirstRow)?;
            self.writer.end();
        }

        if self
            .boundary_constraint_count
            .contains_key(&(AUX_TRACE, ConstraintDomain::LastRow))
        {
            let name = boundary_group_to_procedure_name(AUX_TRACE, ConstraintDomain::LastRow);
            self.writer.header(
            "Procedure to evaluate the boundary constraint numerator for the last row of the auxiliary trace",
        );
            self.writer.header("");
            self.writer.header("Input: [...]");
            self.writer.header("Output: [(r_1, r_0)*, ...]");
            self.writer.header(
                "Where: (r_1, r_0) is one quadratic extension field element for each constraint",
            );
            self.writer.proc(name);
            walk_boundary_constraints(self, self.ir, AUX_TRACE, ConstraintDomain::LastRow)?;
            self.writer.end();
        }

        Ok(())
    }

    /// Emits code for the procedure `get_exemptions_points`.
    ///
    /// Generate code to push the exemption points to the top of the stack.
    /// Stack: [g^{trace_len-2}, g^{trace_len-1}, ...]
    fn gen_get_exemptions_points(&mut self) -> Result<(), CodegenError> {
        self.writer
            .header("Procedure to compute the exemption points.");
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [g^{-2}, g^{-1}, ...]");

        self.writer.proc("get_exemptions_points");
        self.load_trace_domain_generator();
        self.writer.header("=> [g, ...]");

        self.writer.push(1);
        self.writer.swap();
        self.writer.div();
        self.writer.header("=> [g^{-1}, ...]");

        self.writer.dup(0);
        self.writer.dup(0);
        self.writer.mul();
        self.writer.header("=> [g^{-2}, g^{-1}, ...]");

        self.writer.end(); // end proc

        Ok(())
    }

    /// Emits code for the procedure `evaluate_integrity_constraints`.
    ///
    /// Evaluates the integrity constraints for both the main and auxiliary traces.
    fn gen_evaluate_integrity_constraints(&mut self) -> Result<(), CodegenError> {
        self.writer
            .header("Procedure to evaluate all integrity constraints.");
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [(r_1, r_0), ...]");
        self.writer
            .header("Where: (r_1, r_0) is the final result with the divisor applied");

        self.writer.proc("evaluate_integrity_constraints");

        if !self.ir.periodic_columns.is_empty() {
            self.writer.exec("cache_periodic_polys");
        }

        self.writer.exec("compute_integrity_constraints");

        self.writer
            .header("Numerator of the transition constraint polynomial");

        let total_len = self.ir.integrity_constraints(MAIN_TRACE).len()
            + self.ir.integrity_constraints(AUX_TRACE).len();

        for _ in 0..total_len {
            self.writer.ext2add();
        }

        self.writer
            .header("Divisor of the transition constraint polynomial");

        self.writer.exec("compute_integrity_constraint_divisor");

        self.writer.ext2div();
        self.writer.comment("divide the numerator by the divisor");

        self.writer.end();

        Ok(())
    }

    /// Emits code for the procedure `evaluate_boundary_constraints`.
    ///
    /// Evaluates the boundary constraints for both the main and auxiliary traces.
    fn gen_evaluate_boundary_constraints(&mut self) -> Result<(), CodegenError> {
        self.writer
            .header("Procedure to evaluate all boundary constraints.");
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [(r_1, r_0), ...]");
        self.writer
            .header("Where: (r_1, r_0) is the final result with the divisor applied");

        self.writer.proc("evaluate_boundary_constraints");

        let last = self.boundary_constraint_group(ConstraintDomain::LastRow);
        let first = self.boundary_constraint_group(ConstraintDomain::FirstRow);

        if last != 0 && first != 0 {
            self.writer.header("Add first and last row groups");
            self.writer.ext2add();
        }

        self.writer.end();

        Ok(())
    }

    /// Emits code to evaluate the boundary constraint for a given group determined by the domain.
    fn boundary_constraint_group(&mut self, domain: ConstraintDomain) -> usize {
        let aux_count = self
            .boundary_constraint_count
            .get(&(AUX_TRACE, domain))
            .cloned();

        let name = match domain {
            ConstraintDomain::LastRow => "last",
            ConstraintDomain::FirstRow => "first",
            _ => panic!("unexpected domain"),
        };

        if let Some(count) = aux_count {
            self.boundary_constraint_numerator(count, AUX_TRACE, domain);
            self.writer
                .header(format!("=> [(aux_{name}1, aux_{name}0), ...]"));
        }

        let main_count = self
            .boundary_constraint_count
            .get(&(MAIN_TRACE, domain))
            .cloned();

        if let Some(count) = main_count {
            self.boundary_constraint_numerator(count, MAIN_TRACE, domain);

            if aux_count.is_some() {
                self.writer.header(format!(
                    "=> [(main_{name}1, main_{name}0), (aux_{name}1, aux_{name}0), ...]"
                ));
                self.writer.ext2add();
            }

            self.writer.header(format!("=> [({name}1, {name}0), ...]"));
        }

        if aux_count.is_some() || main_count.is_some() {
            self.writer
                .header(format!("Compute the denominator for domain {:?}", domain));

            match domain {
                ConstraintDomain::FirstRow => {
                    self.load_z();
                    self.writer.push(1);
                    self.writer.push(0);
                    self.writer.ext2sub();
                }
                ConstraintDomain::LastRow => {
                    self.load_z();
                    self.writer.mem_load(self.config.exemption_two_address);
                    self.writer.push(0);
                    self.writer.ext2sub();
                }
                _ => panic!("unexpected constraint domain"),
            };

            self.writer
                .header(format!("Compute numerator/denominator for {name} row"));
            self.writer.ext2div();

            aux_count.unwrap_or(0) + main_count.unwrap_or(0)
        } else {
            0
        }
    }

    /// Emits code to evaluate the numerator portion of a boundary constraint point determined by
    /// `segment` and `domain`.
    fn boundary_constraint_numerator(
        &mut self,
        count: usize,
        segment: TraceSegmentId,
        domain: ConstraintDomain,
    ) {
        let name = boundary_group_to_procedure_name(segment, domain);
        self.writer.exec(name);

        if count > 1 {
            self.writer.header(format!(
                "Accumulate the numerator for segment {} {:?}",
                segment, domain
            ));
            for _ in 0..count {
                self.writer.ext2add();
            }
        }
    }

    /// Emits code for the procedure `evaluate_constraints`.
    ///
    /// This will compute and cache values, the transition and boundary constraints for both the main and auxiliary traces.
    fn gen_evaluate_constraints(&mut self) {
        self.writer
            .header("Procedure to evaluate the integrity and boundary constraints.");
        self.writer.header("");
        self.writer.header("Input: [...]");
        self.writer.header("Output: [(r_1, r_0), ...]");

        self.writer.export("evaluate_constraints");

        // The order of execution below is important. These are the dependencies:
        // - `z^trace_len` is computed and cached to be used by integrity contraints
        // - `g^{trace_len-2}` is computed and cached to be used by boundary constraints
        self.writer.exec("cache_z_exp");
        self.writer.exec("evaluate_integrity_constraints");
        self.writer.exec("evaluate_boundary_constraints");
        self.writer.ext2add();

        self.writer.end();
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
        self.writer.comment("load z");
    }

    /// Emits code to load `g`, the trace domain generator.
    fn load_trace_domain_generator(&mut self) {
        self.writer
            .mem_load(self.config.trace_domain_generator_address);
    }
}

impl<'ast> AirVisitor<'ast> for Backend<'ast> {
    type Value = ();
    type Error = CodegenError;

    fn visit_integrity_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: TraceSegmentId,
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

    fn visit_boundary_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: TraceSegmentId,
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

        // Note: The correctness of the load below relies on the integrity constraint being
        // iterated first _and_ the boundary constraints being iterated in natural order.
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
        self.gen_cache_z_exp()?;
        self.gen_get_exemptions_points()?;

        if !self.ir.periodic_columns.is_empty() {
            self.gen_evaluate_periodic_polys()?;
        }

        self.gen_compute_integrity_constraint_divisor()?;

        self.gen_compute_integrity_constraints()?;
        self.gen_compute_boundary_constraints()?;

        // NOTE: Order of the following two methods is important! The iteration order is used to
        // determine the composition coefficient index. The correct order is:
        // 1. Integrity constraints for the MAIN trace.
        // 2. Integrity constraints for the AUX trace.
        // 3. Boundary constraints for the MAIN trace.
        // 4. Boundary constraints for the AUX trace.
        self.gen_evaluate_integrity_constraints()?;
        self.gen_evaluate_boundary_constraints()?;

        self.gen_evaluate_constraints();

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
        };

        Ok(())
    }

    fn visit_periodic_column(
        &mut self,
        column: &'ast PeriodicColumn,
    ) -> Result<Self::Value, Self::Error> {
        // convert the periodic column to a polynomial
        let inv_twiddles = fft::get_inv_twiddles::<Felt>(column.period());
        let mut poly: Vec<Felt> = column.values.iter().map(|e| Felt::new(*e)).collect();
        fft::interpolate_poly(&mut poly, &inv_twiddles);

        self.writer
            .comment(format!("periodic column {}", self.periodic_column));

        // LOAD OOD ELEMENT
        // ---------------------------------------------------------------------------------------

        // assumes that cache_z_exp has been called before, which precomputes the value of z**exp
        let group: u32 = self
            .periods
            .iter()
            .position(|&p| p == column.period())
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

    fn visit_value(&mut self, value: &'ast Value) -> Result<Self::Value, Self::Error> {
        match value {
            Value::Constant(value) => {
                self.writer.push(*value);
                self.writer.push(0);
            }
            Value::TraceAccess(access) => {
                // eventually larger offsets will be supported
                if access.row_offset > 1 {
                    return Err(CodegenError::InvalidRowOffset);
                }

                // Compute the target address for this variable. Each memory address contains the
                // curr and next values of a single variable.
                //
                // Layout defined at: https://github.com/0xPolygonMiden/miden-vm/issues/875
                let target_word: u32 = access
                    .column
                    .try_into()
                    .map_err(|_| CodegenError::InvalidIndex)?;
                let el_pos: u32 = access
                    .row_offset
                    .try_into()
                    .or(Err(CodegenError::InvalidIndex))?;
                let target_element = target_word * 2 + el_pos;

                let base_address = if access.segment == MAIN_TRACE {
                    self.config.ood_frame_address
                } else {
                    self.config.ood_aux_frame_address
                };

                load_quadratic_element(&mut self.writer, base_address, target_element)?;
            }
            Value::PeriodicColumn(access) => {
                let group: u32 = self
                    .periods
                    .iter()
                    .position(|&p| p == access.cycle)
                    .expect("All periods are added in the constructor")
                    .try_into()
                    .expect("periods are u32");
                load_quadratic_element(
                    &mut self.writer,
                    self.config.periodic_values_address,
                    periodic_group_to_memory_offset(group),
                )?;
            }
            Value::PublicInput(access) => {
                let start_offset = self
                    .public_input_to_offset
                    .get(&access.name)
                    .unwrap_or_else(|| panic!("public input {} unknown", access.name));

                self.writer.header(format!(
                    "Load public input {} pos {} with final offset {}",
                    access.name, access.index, start_offset,
                ));
                let index: u32 = (start_offset + access.index)
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
