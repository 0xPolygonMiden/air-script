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

use processor::math::{Felt, StarkField};
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

    /// Holds the count of public inputs seen so far, this is used to compute the offset.
    public_input_count: usize,

    /// Map of the constants found while visitint the [AirIR].
    ///
    /// These values are later used to emit immediate values.
    constants: BTreeMap<&'ast Identifier, &'ast ConstantValueExpr>,

    /// The [AirIR] to visit.
    ir: &'ast AirIR,

    /// Configuration for the codegen.
    config: CodegenConfig,
}

/// Converts a column ID to an element position.
///
/// The element position is then used to determine the memory location to load, and which elements
/// of the word must be kept. Values of periodic columns are stored at distinct memory addresses
/// such that each value occupies a single memory word with the two most significant word elements
/// set to zeros (i.e., [q0, q1, 0, 0])
fn periodic_column_to_target_el(column: u32) -> u32 {
    // each period column has its own address, and the element is in the lower half
    (column * 2) + 1
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
        CodeGenerator {
            writer: Writer::new(),
            periodic_column: 0,
            composition_coefficient_count: 0,
            integrity_contraints: 0,
            boundary_contraints: 0,
            public_input_to_offset: BTreeMap::new(),
            public_input_count: 0,
            constants: BTreeMap::new(),
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
    /// The emitted procedure computes a `z**exp` for each periodic column, these values are later
    /// on used to evaluate the polynomial of each periodic column.
    ///
    /// The emitted code is optimized to performn the fewest number of exponentiations, this is
    /// achieved by observing that periodic columns and trace length are both power-of-two in
    /// length, since the exponent is defined as `exp = trace_len / periodic_column_len`, all
    /// exponents are themselves powers-of-two.
    ///
    /// The algorithm computes the exponents from smallest to largest, using the previous result as
    /// a cache value.
    fn gen_cache_z_exp(&mut self) -> Result<(), CodegenError> {
        // NOTE:
        // - the trace length is a power-of-two.
        //   Ref: https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/random_coin.masm#L82-L87
        // - the periodic columns are powers-of-two.
        //   Ref: https://github.com/0xPolygonMiden/air-script/blob/next/ir/src/symbol_table/mod.rs#L305-L309

        let mut m: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        for (p, c) in self.ir.periodic_columns().iter().enumerate() {
            let idx = p.try_into().or(Err(CodegenError::InvalidIndex))?;
            let len = c
                .len()
                .try_into()
                .expect("length will be used as a memory address, it must fit in a u32");
            m.entry(len).or_insert(vec![]).push(idx);
        }

        self.writer.proc("cache_z_exp");

        self.writer.header("Load trace length");
        self.writer.mem_load(self.config.trace_len_address);
        self.writer.header("=> [trace_len, ...]");

        self.writer.header("Push initial num_of_cycles");
        self.writer.push(1);
        self.writer.header("=> [num_of_cycles, trace_len, ...]");

        self.writer.header("Load Z");
        self.writer.padw();
        self.writer.mem_loadw(self.config.z_address);
        self.writer.drop();
        self.writer.drop();
        self.writer
            .header("=> [z_1, z_0, num_of_cycles, trace_len, ...]");

        for (divisor, columns) in m.iter().rev() {
            self.writer.header(format!(
                "Compute exponentiations based on the number of cycles for a period of {}",
                divisor
            ));
            self.writer.dup(3);
            self.writer.div(Some(*divisor));
            self.writer
                .header("=> [num_of_cycles', z_1, z_0, num_of_cycles, trace_len, ...]");

            self.writer
                .header("Update next num_of_cycles and compute number of iterations");
            self.writer.movup(3);
            self.writer.dup(1);
            self.writer.movdn(4);
            self.writer.div(None);
            self.writer
                .header("=> [i, z_1, z_0, num_of_cycles', trace_len, ...]");

            self.writer
                .header("Exponentiate the existing `z**num_of_cycles` an additional `i` times");
            self.writer.dup(0);
            self.writer.neq(1);

            self.writer.r#while();
            self.writer.movdn(2);
            self.writer.dup(1);
            self.writer.dup(1);
            self.writer.ext2mul();
            self.writer
                .header("=> [z_1^2, z_0^2, i, num_of_cycles', trace_len, ...]");

            self.writer.movup(2);
            self.writer.div(Some(2));
            self.writer.dup(0);
            self.writer.neq(1);
            self.writer
                .header("=> [b, i+1, z_1^2, z_0^2, num_of_cycles', trace_len, ...]");
            self.writer.end();

            self.writer.drop();
            self.writer.push(0);
            self.writer.push(0);

            for c in columns {
                let addr = self.config.z_exp_address + c;
                // each memory address contains the data for a single periodic column, this means
                // the memory has to be zeroed and then we can overwrite the value.
                self.writer.mem_storew(addr);
                self.writer
                    .comment(format!("Save the exponentiation of z for column {}", c));
            }

            self.writer
                .header("=> [0, 0, z_1^2, z_0^2, num_of_cycles', trace_len, ...]");
            self.writer.drop();
            self.writer.drop();
        }
        self.writer
            .header("=> [z_1^2, z_0^2, num_of_cycles', trace_len, ...]");
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
        self.writer.exec("cache_z_exp");
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

        if !self.ir.periodic_columns().is_empty() {
            self.gen_cache_z_exp()?;
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
        constant: &'ast ConstantBinding,
    ) -> Result<Self::Value, Self::Error> {
        match self.constants.entry(constant.name()) {
            Entry::Occupied(_) => Err(CodegenError::DuplicatedConstant),
            Entry::Vacant(entry) => {
                entry.insert(constant.value());
                Ok(())
            }
        }
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
        load_quadratic_element(
            &mut self.writer,
            self.config.z_exp_address,
            periodic_column_to_target_el(self.periodic_column),
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
        constant: &'ast PublicInput,
    ) -> Result<Self::Value, Self::Error> {
        debug_assert!(
            !self.public_input_to_offset.contains_key(&constant.0),
            "public input {} has already been visited",
            constant.0,
        );

        let start_offset = self.public_input_count;
        self.public_input_to_offset
            .insert(constant.0.clone(), start_offset);

        self.public_input_count += constant.1;
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
            Value::PeriodicColumn(column, _) => {
                let column: u32 = (*column).try_into().or(Err(CodegenError::InvalidIndex))?;
                load_quadratic_element(
                    &mut self.writer,
                    self.config.periodic_values_address,
                    periodic_column_to_target_el(column),
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
