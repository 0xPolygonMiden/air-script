use super::{
    symbol_table::IdentifierType, Boundary, BoundaryStmt, ConstantType, Expression, Identifier,
    IndexedTraceAccess, IntegrityStmt, MatrixAccess, NamedTraceAccess, SemanticError, SymbolTable,
    TraceSegment, VariableType, VectorAccess,
};
use std::collections::BTreeMap;

mod constraint;
use air_script_core::{Iterable, ListComprehension};
pub use constraint::{ConstraintDomain, ConstraintRoot};

mod degree;
pub use degree::IntegrityConstraintDegree;

mod graph;
pub use graph::{AlgebraicGraph, ConstantValue, NodeIndex, Operation, VariableValue};

// TYPES
// ================================================================================================
type VariableRoots = BTreeMap<VariableValue, ExprDetails>;

// CONSTANTS
// ================================================================================================

pub const MIN_CYCLE_LENGTH: usize = 2;

// HELPER STRUCTS
// ================================================================================================

/// A struct containing the node index that is the root of the expression, the trace segment to
/// which the expression is applied, and the constraint domain against which any constraint
/// containing this expression must be applied.
#[derive(Debug, Copy, Clone)]
struct ExprDetails {
    root_idx: NodeIndex,
    trace_segment: TraceSegment,
    domain: ConstraintDomain,
    exp_evaluated_value: Option<usize>,
}

impl ExprDetails {
    fn new(
        root_idx: NodeIndex,
        trace_segment: TraceSegment,
        domain: ConstraintDomain,
        exp_evaluated_value: Option<usize>,
    ) -> Self {
        Self {
            root_idx,
            trace_segment,
            domain,
            exp_evaluated_value,
        }
    }

    fn root_idx(&self) -> NodeIndex {
        self.root_idx
    }

    fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    fn domain(&self) -> ConstraintDomain {
        self.domain
    }

    fn exp_evaluated_value(&self) -> Option<usize> {
        self.exp_evaluated_value
    }
}

pub struct ExprContext {
    expression_type: ExpressionType,
    list_comprehension_context: ListComprehensionContext,
    is_exponent: bool,
}

impl ExprContext {
    pub fn new(
        expression_type: ExpressionType,
        list_comprehension_context: ListComprehensionContext,
        is_exponent: bool,
    ) -> Self {
        Self {
            expression_type,
            list_comprehension_context,
            is_exponent,
        }
    }

    pub fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    pub fn list_comprehension_context(&self) -> &ListComprehensionContext {
        &self.list_comprehension_context
    }

    pub fn is_exponent(&self) -> bool {
        self.is_exponent
    }

    pub fn set_list_comprehension_context(
        &mut self,
        list_comprehension_context: ListComprehensionContext,
    ) {
        self.list_comprehension_context = list_comprehension_context
    }

    pub fn set_exponent(&mut self, is_exponent: bool) {
        self.is_exponent = is_exponent;
    }
}

pub enum ExpressionType {
    BoundaryVariable,
    IntegrityVariable,
    BoundaryConstraint,
    IntegrityConstraint,
}

#[derive(Default)]
pub struct ListComprehensionContext {
    context: Vec<(Identifier, Iterable)>,
    idx: usize,
}

impl ListComprehensionContext {
    pub fn new(context: Vec<(Identifier, Iterable)>, idx: usize) -> Self {
        Self { context, idx }
    }

    pub fn context(&self) -> &[(Identifier, Iterable)] {
        &self.context
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

// CONSTRAINTS
// ================================================================================================

/// Contains the graph representing all of the constraints and their subexpressions, the set of
/// variables used in the integrity constraints, and a matrix for each constraint type (boundary,
/// validity, transition), where each index contains a vector of the constraint roots for all the
/// constraints of that type against the segment of the trace corresponding to that index. For
/// example, transition constraints against the main execution trace, which is trace segment 0, will
/// be specified by a vector in transition_constraints[0] containing a [ConstraintRoot] in the graph
/// for each constraint against the main trace.
#[derive(Default, Debug)]
pub(super) struct Constraints {
    /// Constraint roots for all boundary constraints against the execution trace, by trace segment,
    /// where boundary constraints are any constraints that apply to either the first or the last
    /// row of the trace.
    boundary_constraints: Vec<Vec<ConstraintRoot>>,

    /// Constraint roots for all validity constraints against the execution trace, by trace segment,
    /// where validity constraints are any constraints that apply to every row.
    validity_constraints: Vec<Vec<ConstraintRoot>>,

    /// Constraint roots for all transition constraints against the execution trace, by trace
    /// segment, where transition constraints are any constraints that apply to a frame of multiple
    /// rows.
    transition_constraints: Vec<Vec<ConstraintRoot>>,

    /// A directed acyclic graph which represents all of the constraints and their subexpressions.
    graph: AlgebraicGraph,

    /// Variable roots for the variables used in integrity constraints. For each element in a
    /// vector or a matrix, a new root is added with a key equal to the [VariableValue] of the
    /// element.
    variable_roots: VariableRoots,
}

impl Constraints {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    pub fn new(num_trace_segments: usize) -> Self {
        Self {
            boundary_constraints: vec![Vec::new(); num_trace_segments],
            validity_constraints: vec![Vec::new(); num_trace_segments],
            transition_constraints: vec![Vec::new(); num_trace_segments],
            graph: AlgebraicGraph::default(),
            variable_roots: BTreeMap::new(),
        }
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns the number of boundary constraints applied against the specified trace segment.
    pub fn num_boundary_constraints(&self, trace_segment: TraceSegment) -> usize {
        if self.boundary_constraints.len() <= trace_segment.into() {
            return 0;
        }

        self.boundary_constraints[trace_segment as usize].len()
    }

    /// Returns all boundary constraints against the specified trace segment as a slice of
    /// [ConstraintRoot] where each index is the tip of the subgraph representing the constraint
    /// within the constraints [AlgebraicGraph].
    pub fn boundary_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        if self.boundary_constraints.len() <= trace_segment.into() {
            return &[];
        }

        &self.boundary_constraints[trace_segment as usize]
    }

    /// Returns a vector of the degrees of the validity constraints for the specified trace
    /// segment.
    pub fn validity_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        if self.validity_constraints.len() <= trace_segment.into() {
            return Vec::new();
        }

        self.validity_constraints[trace_segment as usize]
            .iter()
            .map(|entry_index| self.graph.degree(entry_index.node_index()))
            .collect()
    }

    /// Returns all validity constraints against the specified trace segment as a vector of
    /// references to [ConstraintRoot] where each index is the tip of the subgraph representing the
    /// constraint within the [AlgebraicGraph].
    pub fn validity_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        if self.validity_constraints.len() <= trace_segment.into() {
            return &[];
        }

        &self.validity_constraints[trace_segment as usize]
    }

    /// Returns a vector of the degrees of the transition constraints for the specified trace
    /// segment.
    pub fn transition_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        if self.transition_constraints.len() <= trace_segment.into() {
            return Vec::new();
        }

        self.transition_constraints[trace_segment as usize]
            .iter()
            .map(|entry_index| self.graph.degree(entry_index.node_index()))
            .collect()
    }

    /// Returns all transition constraints against the specified trace segment as a vector of
    /// references to [ConstraintRoot] where each index is the tip of the subgraph representing the
    /// constraint within the [AlgebraicGraph].
    pub fn transition_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        if self.transition_constraints.len() <= trace_segment.into() {
            return &[];
        }

        &self.transition_constraints[trace_segment as usize]
    }

    /// Returns the [AlgebraicGraph] representing all constraints and sub-expressions.
    pub fn graph(&self) -> &AlgebraicGraph {
        &self.graph
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Adds the provided parsed integrity statement to the graph. The statement can either be a
    /// variable defined in the integrity constraints section or an integrity constraint.
    ///
    /// In case the statement is a variable, it is added to the symbol table.
    ///
    /// In case the statement is a constraint, the constraint is turned into a subgraph which is
    /// added to the [AlgebraicGraph] (reusing any existing nodes). The index of its entry node
    /// is then saved in the validity_constraints or transition_constraints matrices.
    pub(super) fn insert_integrity_stmt(
        &mut self,
        symbol_table: &mut SymbolTable,
        stmt: &IntegrityStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            IntegrityStmt::Constraint(constraint) => {
                // add the left hand side expression to the graph.
                let lhs = self.graph.insert_expr(
                    symbol_table,
                    constraint.lhs(),
                    &mut self.variable_roots,
                    ConstraintDomain::EveryRow,
                    &mut ExprContext::new(
                        ExpressionType::IntegrityConstraint,
                        ListComprehensionContext::default(),
                        false,
                    ),
                )?;

                // add the right hand side expression to the graph.
                let rhs = self.graph.insert_expr(
                    symbol_table,
                    constraint.rhs(),
                    &mut self.variable_roots,
                    ConstraintDomain::EveryRow,
                    &mut ExprContext::new(
                        ExpressionType::IntegrityConstraint,
                        ListComprehensionContext::default(),
                        false,
                    ),
                )?;

                // merge the two sides of the expression into a constraint.
                self.insert_constraint(symbol_table, lhs, rhs)?
            }
            IntegrityStmt::Variable(variable) => {
                if let VariableType::ListComprehension(list_comprehension) = variable.value() {
                    self.validate_list_comprehension(symbol_table, list_comprehension)?;
                }
                symbol_table.insert_integrity_variable(variable)?
            }
        }

        Ok(())
    }

    /// Adds the provided parsed boundary statement to the graph. The statement can either be a
    /// variable defined in the boundary constraints section or a boundary constraint expression.
    ///
    /// In case the statement is a variable, it is added to the symbol table.
    ///
    /// In case the statement is a constraint, the constraint is turned into a subgraph which is
    /// added to the [AlgebraicGraph] (reusing any existing nodes). The index of its entry node
    /// is then saved in the boundary_constraints matrix.
    pub(super) fn insert_boundary_stmt(
        &mut self,
        symbol_table: &mut SymbolTable,
        stmt: &BoundaryStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            BoundaryStmt::Constraint(constraint) => {
                let domain = constraint.boundary().into();
                // add the trace access at the specified boundary to the graph.
                // TODO: need to validate that only one constraint is specified against this column and boundary
                let lhs = self.graph.insert_named_trace_access(
                    symbol_table,
                    constraint.access(),
                    domain,
                )?;

                // add its expression to the constraints graph.
                // TODO: need to validate the expression.
                let rhs = self.graph.insert_expr(
                    symbol_table,
                    constraint.value(),
                    &mut self.variable_roots,
                    domain,
                    &mut ExprContext::new(
                        ExpressionType::BoundaryConstraint,
                        ListComprehensionContext::default(),
                        false,
                    ),
                )?;

                debug_assert!(lhs.domain() == rhs.domain());

                // merge the two sides of the expression into a constraint.
                self.insert_constraint(symbol_table, lhs, rhs)?
            }
            BoundaryStmt::Variable(_variable) => {
                unimplemented!("TODO: add support for boundary variables")
            }
        }

        Ok(())
    }

    /// Takes two expressions which are expected to be equal and merges them into a constraint (a
    /// subtree in the graph that must be equal to zero for a particular domain). The constraint is
    /// then saved in the appropriate constraint list (boundary, validity, or transition).
    fn insert_constraint(
        &mut self,
        symbol_table: &mut SymbolTable,
        lhs: ExprDetails,
        rhs: ExprDetails,
    ) -> Result<(), SemanticError> {
        let constraint = self.graph.merge_equal_exprs(&lhs, &rhs)?;
        let trace_segment = constraint.trace_segment as usize;

        // the constraint should not be against an undeclared trace segment.
        if symbol_table.num_trace_segments() <= trace_segment {
            return Err(SemanticError::InvalidConstraint(
                "Constraint against undeclared trace segment".to_string(),
            ));
        }

        let constraint_root = ConstraintRoot::new(constraint.root_idx(), constraint.domain());
        // add the constraint to the appropriate set of constraints.
        match constraint.domain() {
            ConstraintDomain::FirstRow | ConstraintDomain::LastRow => {
                self.boundary_constraints[trace_segment].push(constraint_root);
            }
            ConstraintDomain::EveryRow => {
                self.validity_constraints[trace_segment].push(constraint_root);
            }
            ConstraintDomain::EveryFrame(_) => {
                self.transition_constraints[trace_segment].push(constraint_root);
            }
        }

        Ok(())
    }

    // --- HELPERS --------------------------------------------------------------------------------

    fn validate_list_comprehension(
        &self,
        symbol_table: &mut SymbolTable,
        list_comprehension: &ListComprehension,
    ) -> Result<(), SemanticError> {
        let expected_iterable_len =
            self.iterable_length(symbol_table, &list_comprehension.context()[0].1)?;
        for (_, iterable) in list_comprehension.context().iter().skip(1) {
            let iterable_len = self.iterable_length(symbol_table, iterable)?;
            if iterable_len != expected_iterable_len {
                return Err(SemanticError::InvalidListComprehension(
                    "All iterables in a list comprehension must have the same length".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn iterable_length(
        &self,
        symbol_table: &SymbolTable,
        iterable: &Iterable,
    ) -> Result<usize, SemanticError> {
        match iterable {
            Iterable::Identifier(ident) => {
                let ident_type = symbol_table.get_type(ident.name())?;
                match ident_type {
                    IdentifierType::IntegrityVariable(var_type) => {
                        match var_type.value() {
                            VariableType::Vector(vector) => Ok(vector.len()),
                            VariableType::ListComprehension(_) =>
                            // TODO: change error
                            {
                                Err(SemanticError::InvalidListComprehension(
                                    "List comprehensions cannot be used as list indices"
                                        .to_string(),
                                ))
                            }
                            _ => Err(SemanticError::InvalidListComprehension(
                                "List comprehensions cannot be used as list indices".to_string(),
                            )),
                        }
                    }
                    IdentifierType::PublicInput(size) => Ok(*size),
                    IdentifierType::TraceColumns(trace_columns) => Ok(trace_columns.size()),
                    // TODO: change error
                    _ => Err(SemanticError::InvalidListComprehension(
                        "List comprehensions cannot be used as list indices".to_string(),
                    )),
                }
            }
            Iterable::Range(range) | Iterable::Slice(_, range) => Ok(range.end() - range.start()),
        }
    }
}
