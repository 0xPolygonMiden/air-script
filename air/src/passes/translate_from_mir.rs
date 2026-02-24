//! MIR to AIR lowering pass.
//!
//! The goal is to convert the MIR constraint graph into AIR operations after inlining/unrolling.
//! We do that by structurally translating MIR ops while memoizing repeated subgraphs using
//! canonical MIR keys (including commutative canonicalization) to avoid blow-ups. The tradeoff
//! is that normalization stays conservative (e.g. vector/accessor stripping relies on prior
//! passes), and caching is intentionally rigid so keys retain all semantic details, which means
//! we miss some reuse opportunities that aggressive hashing would allow.

use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};

use air_parser::{
    SemanticAnalysisError, Symbol,
    ast::{self, ConstraintTagSpec, TraceSegment},
};
use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, Severity, SourceSpan, Span, Spanned};
use mir::{
    ir::{
        Boundary as MirBoundary, ConstantValue, Link, Mir, MirAccessType, MirValue, Op, Parent,
        SpannedMirValue, TraceAccess as MirTraceAccess,
    },
    passes::get_inner_const,
};

use crate::{CompileError, graph::NodeIndex, ir::*};

/// Lowers a fully inlined/unrolled MIR graph into AIR.
///
/// After inlining and unrolling, MIR nodes correspond 1:1 with AIR operations,
/// so the translation is mostly structural. We add memoization to avoid
/// re-creating identical subgraphs when MIR contains repeated shapes.
pub struct MirToAir<'a> {
    diagnostics: &'a DiagnosticsHandler,
}
impl<'a> MirToAir<'a> {
    /// Create a new instance of this pass
    #[inline]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}
impl Pass for MirToAir<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, mir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut air = Air::new(mir.name);
        air.expected_max_constraint_id = mir.expected_max_constraint_id;

        let buses = mir.constraint_graph().buses.clone();

        assert!(
            mir.trace_columns.len() == 1,
            "Expected one trace segment, but found: {:?}",
            mir.trace_columns
        );
        let main_trace_segment = mir.trace_columns.first().unwrap();

        assert_eq!(
            main_trace_segment.id,
            TraceSegmentId::Main,
            "Expected trace segment to be the main segment, but found: {:?}",
            main_trace_segment.id
        );

        // Build trace segments shape: always include main; aux may be empty
        let trace_columns_main = main_trace_segment.clone();

        let mut bus_bindings_map = BTreeMap::new();
        let trace_columns_aux = if buses.is_empty() {
            TraceSegment::new(
                SourceSpan::default(),
                TraceSegmentId::Aux,
                Identifier::new(SourceSpan::default(), Symbol::intern("$aux")),
                vec![],
            )
        } else {
            let bus_raw_bindings: Vec<_> = buses
                .keys()
                .map(|k| Span::new(k.span(), (Identifier::new(k.span(), k.name()), 1)))
                .collect();

            // Add buses as `aux` trace columns
            let aux_trace_segment = TraceSegment::new(
                SourceSpan::default(),
                TraceSegmentId::Aux,
                Identifier::new(SourceSpan::default(), Symbol::intern("$aux")),
                bus_raw_bindings,
            );
            for binding in aux_trace_segment.bindings.iter() {
                bus_bindings_map.insert(binding.name.unwrap(), binding.offset);
            }
            aux_trace_segment
        };

        let trace_columns = TraceShape::new(trace_columns_main, trace_columns_aux);

        air.trace_segment_widths = vec![
            trace_columns[TraceSegmentId::Main].size as u16,
            trace_columns[TraceSegmentId::Aux].size as u16,
        ];
        air.num_random_values = mir.num_random_values;
        air.periodic_columns = mir.periodic_columns.clone();
        air.public_inputs = mir.public_inputs.clone();

        let mut builder = AirBuilder {
            diagnostics: self.diagnostics,
            air: &mut air,
            trace_columns: trace_columns.clone(),
            bus_bindings_map,
            tag_allocators: HashMap::new(),
            mir_node_cache: HashMap::new(),
            mir_key_by_ptr: HashMap::new(),
            mir_key_intern: HashMap::new(),
            mir_key_to_air: HashMap::new(),
            next_mir_key_id: 1,
            air_op_cache: HashMap::new(),
        };

        let graph = mir.constraint_graph();

        // We insert all the constraints into the AIR graph.
        // Note: We need to insert the boundary constraints before the integrity constraints
        // as it's a requirement for the CommonSubexpressionElimination pass to work with the
        // winterfell codegen.
        for bc in graph.boundary_constraints_roots.borrow().deref().iter() {
            builder.build_boundary_constraint(bc, None)?;
        }

        for ic in graph.integrity_constraints_roots.borrow().deref().iter() {
            builder.build_integrity_constraint(ic, None)?;
        }

        // Note: In the MIR, buses operations are kept in integrity constraints to
        // allow them to be handled in the graph (e.g. inlined via evaluators). This is why
        // we need to first visit the integrity constraints, update the corresponding bus
        // when encountering a `BusOp`, and then visit the buses to build them.
        for bus in buses.values() {
            builder.build_bus(bus)?;
        }

        Ok(air)
    }
}

/// Stateful builder for MIR to AIR translation.
///
/// This keeps per-run caches so repeated MIR subgraphs map to the same AIR nodes,
/// reducing graph size and memory usage after aggressive inlining.
struct AirBuilder<'a> {
    diagnostics: &'a DiagnosticsHandler,
    air: &'a mut Air,
    trace_columns: TraceShape<TraceSegment>,
    bus_bindings_map: BTreeMap<Identifier, usize>,
    tag_allocators: HashMap<SourceSpan, TagAllocator>,
    /// Fast path: map an exact MIR node pointer to the AIR node already created for it.
    /// This avoid re-lowering the same MIR node when it is referenced multiple times.
    mir_node_cache: HashMap<usize, NodeIndex>,
    /// Memoize the canonical key for a MIR node pointer to avoid recomputing it.
    /// The aim is to reduce repeated structural hashing when the same MIR node is visited again.
    mir_key_by_ptr: HashMap<usize, MirKeyId>,
    /// Intern table for canonical MIR keys (stable ids for structural shapes).
    /// This assigns compact ids to structural shapes so they can be referenced cheaply.
    mir_key_intern: HashMap<MirKey, MirKeyId>,
    /// Cross-pointer cache: canonical MIR key to AIR node for structurally identical subgraphs.
    /// Reuses AIR nodes even when MIR pointers differ but the structure is identical.
    mir_key_to_air: HashMap<MirKeyId, NodeIndex>,
    next_mir_key_id: MirKeyId,
    /// Deduplicate AIR operations so identical operations share a single graph node.
    /// Its purpose is to keep the AIR algebraic DAG compact by reusing identical ops.
    air_op_cache: HashMap<Operation, NodeIndex>,
}

/// Stable id for canonical MIR keys (interned).
type MirKeyId = u64;

/// Hashable identifier used in MIR key canonicalization.
#[derive(Hash, Eq, PartialEq, Clone)]
enum NamespacedIdentifierKey {
    Function(Symbol),
    Binding(Symbol),
}

/// Hashable fully-qualified identifier for periodic columns.
#[derive(Hash, Eq, PartialEq, Clone)]
struct QualifiedIdentifierKey {
    module: Vec<Symbol>,
    item: NamespacedIdentifierKey,
}

/// Canonical key for MIR values used in caching.
#[derive(Hash, Eq, PartialEq, Clone)]
enum MirValueKey {
    Constant(u64),
    TraceAccess {
        segment: TraceSegmentId,
        column: ast::TraceColumnIndex,
        row_offset: usize,
    },
    BusAccess {
        name: Symbol,
        row_offset: usize,
    },
    PeriodicColumn {
        name: QualifiedIdentifierKey,
        cycle: usize,
    },
    PublicInput {
        name: Symbol,
        index: usize,
    },
    PublicInputTable {
        name: Symbol,
        num_cols: usize,
        bus_type: BusType,
    },
    RandomValue(usize),
}

/// Canonical key for MIR ops used in caching.
#[derive(Hash, Eq, PartialEq, Clone)]
enum MirKey {
    Add { lhs: MirKeyId, rhs: MirKeyId },
    Sub { lhs: MirKeyId, rhs: MirKeyId },
    Mul { lhs: MirKeyId, rhs: MirKeyId },
    Exp { lhs: MirKeyId, rhs: u64 },
    Value(MirValueKey),
}

/// Allocates tags from a constraint tag specification.
///
/// This is used when a constraint declares a range/list of tags and we must
/// consume them in a deterministic order during lowering.
struct TagAllocator {
    spec: ConstraintTagSpec,
    next_idx: usize,
}

impl TagAllocator {
    /// Create a new allocator for the given tag specification.
    fn new(spec: ConstraintTagSpec) -> Self {
        Self { spec, next_idx: 0 }
    }

    /// Return the next available tag, or `None` once exhausted.
    fn next(&mut self) -> Option<Span<u64>> {
        let span = self.spec.span();
        match &self.spec {
            ConstraintTagSpec::Single(tag) => {
                if self.next_idx == 0 {
                    self.next_idx = 1;
                    Some(*tag)
                } else {
                    None
                }
            },
            ConstraintTagSpec::Range { start, .. } => {
                let len = self.spec.len();
                if self.next_idx >= len {
                    None
                } else {
                    let value = *start + self.next_idx as u64;
                    self.next_idx += 1;
                    Some(Span::new(span, value))
                }
            },
            ConstraintTagSpec::List { tags, .. } => {
                if self.next_idx >= tags.len() {
                    None
                } else {
                    let value = tags[self.next_idx];
                    self.next_idx += 1;
                    Some(Span::new(span, value))
                }
            },
        }
    }
}

/// In case of nested list comprehension, we may not have entirely unrolled outer loops iterators
/// so we need to ensure these cases are properly indexed.
fn accessor_to_scalar(mir_node: &Link<Op>) -> Link<Op> {
    if let Some(accessor) = mir_node.as_accessor() {
        match accessor.access_type.clone() {
            MirAccessType::Index(index) => {
                if let Some(vec) = accessor.indexable.as_vector() {
                    let index = get_inner_const(&index)
                        .expect("Index should be a constant value after constant propagation")
                        as usize;
                    let children = vec.elements.borrow();
                    if index >= children.len() {
                        panic!(
                            "Index out of bounds during indexed accessor translation from MIR to AIR: {index}",
                        );
                    }
                    children[index].clone()
                } else {
                    mir_node.clone()
                }
            },
            MirAccessType::Default => {
                add_row_offset_if_trace_access(&accessor.indexable, accessor.offset)
            },
            MirAccessType::Matrix(row, col) => {
                if let Some(matrix) = accessor.indexable.as_matrix() {
                    let row_index = get_inner_const(&row)
                        .expect("Row index should be a constant value after constant propagation")
                        as usize;
                    let col_index = get_inner_const(&col).expect(
                        "Column index should be a constant value after constant propagation",
                    ) as usize;
                    let rows = matrix.elements.borrow();
                    if row_index >= rows.len() {
                        panic!(
                            "Row index out of bounds during matrix accessor translation from MIR to AIR: {row_index}",
                        );
                    }
                    let row_node = rows[row_index].clone();
                    if let Some(row_vec) = row_node.as_vector() {
                        let cols = row_vec.elements.borrow();
                        if col_index >= cols.len() {
                            panic!(
                                "Column index out of bounds during matrix accessor translation from MIR to AIR: {col_index}",
                            );
                        }
                        cols[col_index].clone()
                    } else {
                        row_node
                    }
                } else {
                    mir_node.clone()
                }
            },
        }
    } else {
        mir_node.clone()
    }
}

/// Helper function to add a row offset to a TraceAccess value, and return the node unchanged
/// otherwise.
fn add_row_offset_if_trace_access(node: &Link<Op>, offset: usize) -> Link<Op> {
    if let Some(value) = node.clone().as_value() {
        let mir_value = value.value.value.clone();
        if let MirValue::TraceAccess(trace_access) = mir_value {
            mir::ir::Value::create(SpannedMirValue {
                span: value.value.span(),
                value: MirValue::TraceAccess(mir::ir::TraceAccess {
                    segment: trace_access.segment,
                    column: trace_access.column,
                    row_offset: trace_access.row_offset + offset,
                }),
            })
        } else {
            node.clone()
        }
    } else {
        node.clone()
    }
}

/// Helper function to remove the vector wrapper from a scalar operation
/// Will panic if the node is a vector of size > 1 (should not happen after unrolling)
fn vec_to_scalar(mir_node: &Link<Op>) -> Link<Op> {
    if let Some(vector) = mir_node.as_vector() {
        let size = vector.size;
        if size != 1 {
            panic!("Vector of len >1 after unrolling: {mir_node:?}");
        }
        let children = vector.elements.borrow();
        let child = children.first().unwrap().clone();
        let child = vec_to_scalar(&child);
        let child = accessor_to_scalar(&child);
        child.clone()
    } else {
        mir_node.clone()
    }
}

/// Helper function to remove the enf wrapper from a scalar operation
fn enf_to_scalar(mir_node: &Link<Op>) -> Link<Op> {
    if let Some(enf) = mir_node.as_enf() {
        let child = enf.expr.clone();
        let child = enf_to_scalar(&child);
        child.clone()
    } else {
        mir_node.clone()
    }
}

impl AirBuilder<'_> {
    /// Normalize MIR nodes to improve cache hit rate (strip vectors/accessors/enf wrappers).
    fn normalize_mir_node(&self, mir_node: &Link<Op>) -> Link<Op> {
        let mir_node = accessor_to_scalar(mir_node);
        let mir_node = vec_to_scalar(&mir_node);
        accessor_to_scalar(&mir_node)
    }

    fn intern_mir_key(&mut self, key: MirKey) -> MirKeyId {
        if let Some(existing) = self.mir_key_intern.get(&key) {
            return *existing;
        }
        let id = self.next_mir_key_id;
        self.next_mir_key_id += 1;
        self.mir_key_intern.insert(key, id);
        id
    }

    /// Build a canonical key for a MIR node, after normalization.
    fn mir_key_for(&mut self, mir_node: &Link<Op>) -> Result<MirKeyId, CompileError> {
        let mir_node = self.normalize_mir_node(mir_node);
        self.mir_key_for_normalized(&mir_node)
    }

    fn canonical_commutative(lhs: MirKeyId, rhs: MirKeyId) -> (MirKeyId, MirKeyId) {
        // Canonical order for commutative ops (Add/Mul) to increase cache hits.
        if lhs <= rhs { (lhs, rhs) } else { (rhs, lhs) }
    }

    /// Build a canonical key for a normalized MIR node.
    fn mir_key_for_normalized(&mut self, mir_node: &Link<Op>) -> Result<MirKeyId, CompileError> {
        if let Some(existing) = self.mir_key_by_ptr.get(&mir_node.get_ptr()) {
            return Ok(*existing);
        }

        let key = match mir_node.borrow().deref() {
            Op::Add(add) => {
                let lhs = self.mir_key_for(&add.lhs)?;
                let rhs = self.mir_key_for(&add.rhs)?;
                // Canonicalize commutative ops to maximize cache hits.
                let (lhs, rhs) = Self::canonical_commutative(lhs, rhs);
                MirKey::Add { lhs, rhs }
            },
            Op::Sub(sub) => {
                let lhs = self.mir_key_for(&sub.lhs)?;
                let rhs = self.mir_key_for(&sub.rhs)?;
                MirKey::Sub { lhs, rhs }
            },
            Op::Mul(mul) => {
                let lhs = self.mir_key_for(&mul.lhs)?;
                let rhs = self.mir_key_for(&mul.rhs)?;
                // Canonicalize commutative ops to maximize cache hits.
                let (lhs, rhs) = Self::canonical_commutative(lhs, rhs);
                MirKey::Mul { lhs, rhs }
            },
            Op::Exp(exp) => {
                let lhs = self.mir_key_for(&exp.lhs)?;
                let rhs = self.exp_rhs_const(&exp.rhs)?;
                MirKey::Exp { lhs, rhs }
            },
            Op::Value(value) => {
                let value_key = self.mir_value_key_from_value(&value.value.value)?;
                MirKey::Value(value_key)
            },
            Op::Enf(enf) => {
                return self.mir_key_for(&enf.expr);
            },
            Op::Accessor(accessor) => {
                let value_key = self.mir_value_key_for_accessor(accessor)?;
                MirKey::Value(value_key)
            },
            _ => panic!("Should not have Mir op in graph: {mir_node:?}"),
        };

        let key_id = self.intern_mir_key(key);
        self.mir_key_by_ptr.insert(mir_node.get_ptr(), key_id);
        Ok(key_id)
    }

    /// Canonicalize a MIR value into a cache key.
    fn mir_value_key_from_value(&self, mir_value: &MirValue) -> Result<MirValueKey, CompileError> {
        self.mir_value_key_from_value_with_offset(mir_value, None)
    }

    /// Canonicalize a MIR value into a cache key with an optional row-offset override.
    fn mir_value_key_from_value_with_offset(
        &self,
        mir_value: &MirValue,
        row_offset_override: Option<usize>,
    ) -> Result<MirValueKey, CompileError> {
        Ok(match mir_value {
            MirValue::Constant(ConstantValue::Felt(felt)) => MirValueKey::Constant(*felt),
            MirValue::Constant(constant_value) => {
                unreachable!("Unexpected MirValue: {:#?}", constant_value)
            },
            MirValue::TraceAccess(trace_access) => MirValueKey::TraceAccess {
                segment: trace_access.segment,
                column: trace_access.column,
                row_offset: row_offset_override.unwrap_or(trace_access.row_offset),
            },
            MirValue::BusAccess(bus_access) => {
                let name = bus_access.bus.borrow().deref().name().name();
                MirValueKey::BusAccess {
                    name,
                    row_offset: row_offset_override.unwrap_or(bus_access.row_offset),
                }
            },
            MirValue::PeriodicColumn(periodic_column_access) => MirValueKey::PeriodicColumn {
                name: Self::qualified_identifier_key(&periodic_column_access.name),
                cycle: periodic_column_access.cycle,
            },
            MirValue::PublicInput(public_input_access) => MirValueKey::PublicInput {
                name: public_input_access.name.name(),
                index: public_input_access.index,
            },
            MirValue::PublicInputTable(public_input_table_access) => {
                let bus_type = public_input_table_access.bus_type();
                MirValueKey::PublicInputTable {
                    name: public_input_table_access.table_name.name(),
                    num_cols: public_input_table_access.num_cols,
                    bus_type,
                }
            },
            MirValue::RandomValue(index) => MirValueKey::RandomValue(*index),
            _ => unreachable!("Unexpected MirValue: {:#?}", mir_value),
        })
    }

    fn mir_value_key_for_accessor(
        &self,
        accessor: &mir::ir::Accessor,
    ) -> Result<MirValueKey, CompileError> {
        let offset = accessor.offset;
        let child = accessor_to_scalar(&accessor.indexable);
        let value = child.as_value().expect("Expected value in accessor");
        self.mir_value_key_from_value_with_offset(&value.value.value, Some(offset))
    }

    fn air_value_from_mir_value(
        &self,
        mir_value: &MirValue,
        row_offset_override: Option<usize>,
    ) -> Result<Value, CompileError> {
        Ok(match mir_value {
            MirValue::Constant(constant_value) => {
                if let ConstantValue::Felt(felt) = constant_value {
                    crate::ir::Value::Constant(*felt)
                } else {
                    unreachable!("Unexpected MirValue: {:#?}", mir_value)
                }
            },
            MirValue::TraceAccess(trace_access) => {
                crate::ir::Value::TraceAccess(crate::ir::TraceAccess {
                    segment: trace_access.segment,
                    column: trace_access.column,
                    row_offset: row_offset_override.unwrap_or(trace_access.row_offset),
                })
            },
            MirValue::BusAccess(bus_access) => {
                let name = bus_access.bus.borrow().deref().name();
                let column = self.bus_bindings_map.get(&name).unwrap();
                crate::ir::Value::TraceAccess(crate::ir::TraceAccess {
                    segment: TraceSegmentId::Aux,
                    column: *column,
                    row_offset: row_offset_override.unwrap_or(bus_access.row_offset),
                })
            },
            MirValue::PeriodicColumn(periodic_column_access) => {
                crate::ir::Value::PeriodicColumn(crate::ir::PeriodicColumnAccess {
                    name: periodic_column_access.name.clone(),
                    cycle: periodic_column_access.cycle,
                })
            },
            MirValue::PublicInput(public_input_access) => {
                crate::ir::Value::PublicInput(crate::ir::PublicInputAccess {
                    name: public_input_access.name,
                    index: public_input_access.index,
                })
            },
            MirValue::PublicInputTable(public_input_table_access) => {
                crate::ir::Value::PublicInputTable(crate::ir::PublicInputTableAccess::new(
                    public_input_table_access.table_name,
                    public_input_table_access.num_cols,
                    public_input_table_access.bus_type(),
                ))
            },
            MirValue::RandomValue(index) => crate::ir::Value::RandomValue(*index),
            _ => unreachable!("Unexpected MirValue: {:#?}", mir_value),
        })
    }

    fn exp_rhs_const(&self, rhs: &Link<Op>) -> Result<u64, CompileError> {
        let rhs = match rhs.borrow().deref() {
            Op::Accessor(accessor) => accessor.indexable.clone(),
            _ => rhs.clone(),
        };
        let Some(value_ref) = rhs.as_value() else {
            return Err(CompileError::SemanticAnalysis(SemanticAnalysisError::InvalidExpr(
                ast::InvalidExprError::NonConstantExponent(rhs.span()),
            )));
        };
        let mir_value = value_ref.value.value.clone();
        let MirValue::Constant(constant_value) = mir_value else {
            return Err(CompileError::SemanticAnalysis(SemanticAnalysisError::InvalidExpr(
                ast::InvalidExprError::NonConstantExponent(rhs.span()),
            )));
        };
        let ConstantValue::Felt(rhs_value) = constant_value else {
            return Err(CompileError::SemanticAnalysis(SemanticAnalysisError::InvalidExpr(
                ast::InvalidExprError::NonConstantExponent(rhs.span()),
            )));
        };
        Ok(rhs_value)
    }

    fn qualified_identifier_key(ident: &ast::QualifiedIdentifier) -> QualifiedIdentifierKey {
        QualifiedIdentifierKey {
            module: Self::module_symbols(&ident.module),
            item: Self::namespaced_identifier_key(&ident.item),
        }
    }

    fn module_symbols(module: &ast::ModuleId) -> Vec<Symbol> {
        (0..module.len()).map(|idx| module[idx].name()).collect()
    }

    fn namespaced_identifier_key(ident: &ast::NamespacedIdentifier) -> NamespacedIdentifierKey {
        match ident {
            ast::NamespacedIdentifier::Function(id) => NamespacedIdentifierKey::Function(id.name()),
            ast::NamespacedIdentifier::Binding(id) => NamespacedIdentifierKey::Binding(id.name()),
        }
    }

    // Uses square and multiply algorithm to expand the exp into a series of multiplications
    fn expand_exp(&mut self, lhs: NodeIndex, rhs: u64) -> NodeIndex {
        match rhs {
            0 => self.insert_op(Operation::Value(Value::Constant(1))),
            1 => lhs,
            n if n.is_multiple_of(2) => {
                let square = self.insert_op(Operation::Mul(lhs, lhs));
                self.expand_exp(square, n / 2)
            },
            n => {
                let square = self.insert_op(Operation::Mul(lhs, lhs));
                let rec = self.expand_exp(square, (n - 1) / 2);
                self.insert_op(Operation::Mul(lhs, rec))
            },
        }
    }

    /// Recursively insert the MIR operations into the AIR graph
    /// Will panic when encountering an unexpected operation
    /// (i.e. that is not a binary operation, a value, enf node or an accessor)
    fn insert_mir_operation(&mut self, mir_node: &Link<Op>) -> Result<NodeIndex, CompileError> {
        // First, we need to remove accessors and vector wrappers to get the actual scalar operation
        // to insert. Notes:
        // - at this point, we expect trivial `Accessor` (with either constant index or default
        //   access type) or `Vector` with size 1.
        // - in case of nested list comprehensions, we may need to unwrap two accessors, so we
        //   unwrap them multiple times.
        let mir_node = self.normalize_mir_node(mir_node);

        // Cache flow (where each field participates):
        // 1) `mir_node_cache` fast-path: exact MIR pointer to AIR node.
        // 2) `mir_key_by_ptr` memoizes the canonical key computation for that pointer.
        // 3) `mir_key_intern` assigns a stable `MirKeyId` for each structural shape (i.e. op +
        //    canonicalized children/value keys, ignoring pointer identity).
        // 4) `mir_key_to_air` reuses AIR nodes across different MIR pointers that share the same
        //    canonical key/shape.
        // 5) On a miss, we build the AIR node and populate both pointer + key caches.
        // 6) `air_op_cache` (inside `insert_op`) deduplicates AIR operations themselves.
        if let Some(cached) = self.mir_node_cache.get(&mir_node.get_ptr()) {
            return Ok(*cached);
        }
        let key = self.mir_key_for_normalized(&mir_node)?;
        if let Some(cached) = self.mir_key_to_air.get(&key) {
            self.mir_node_cache.insert(mir_node.get_ptr(), *cached);
            return Ok(*cached);
        }
        let mir_node_ref = mir_node.borrow();
        let node = match mir_node_ref.deref() {
            Op::Add(add) => self.insert_binary_op(Operation::Add, &add.lhs, &add.rhs)?,
            Op::Sub(sub) => self.insert_binary_op(Operation::Sub, &sub.lhs, &sub.rhs)?,
            Op::Mul(mul) => self.insert_binary_op(Operation::Mul, &mul.lhs, &mul.rhs)?,
            Op::Exp(exp) => {
                let lhs = exp.lhs.clone();
                let lhs_node_index = self.insert_mir_operation(&lhs)?;
                let rhs_value = self.exp_rhs_const(&exp.rhs)?;
                self.expand_exp(lhs_node_index, rhs_value)
            },
            Op::Value(value) => {
                let mir_value = &value.value.value;
                let value = self.air_value_from_mir_value(mir_value, None)?;
                self.insert_op(Operation::Value(value))
            },
            Op::Enf(enf) => {
                let child = enf.expr.clone();
                self.insert_mir_operation(&child)?
            },
            Op::Accessor(accessor) => {
                let offset = accessor.offset;
                let child = accessor.indexable.clone();
                let child = accessor_to_scalar(&child);

                let value = child.as_value().expect("Expected value in accessor");
                let mir_value = &value.value.value;
                let value = self.air_value_from_mir_value(mir_value, Some(offset))?;
                self.insert_op(Operation::Value(value))
            },
            _ => panic!("Should not have Mir op in graph: {mir_node:?}"),
        };
        self.mir_node_cache.insert(mir_node.get_ptr(), node);
        self.mir_key_to_air.insert(key, node);
        Ok(node)
    }

    fn insert_binary_op(
        &mut self,
        op: fn(NodeIndex, NodeIndex) -> Operation,
        lhs: &Link<Op>,
        rhs: &Link<Op>,
    ) -> Result<NodeIndex, CompileError> {
        let lhs_node_index = self.insert_mir_operation(lhs)?;
        let rhs_node_index = self.insert_mir_operation(rhs)?;
        Ok(self.insert_op(op(lhs_node_index, rhs_node_index)))
    }

    fn build_boundary_constraint(
        &mut self,
        bc: &Link<Op>,
        tag: Option<ConstraintTagSpec>,
    ) -> Result<(), CompileError> {
        match bc.borrow().deref() {
            Op::Vector(vector) => {
                let vec = vector.elements.borrow().deref().clone();
                if let Some(tag_spec) = tag {
                    // Tag ranges are defined over the expanded constraint list, not the
                    // top-level vector shape. Compute the expanded length and slice tags
                    // per child accordingly.
                    let total = vec.iter().map(Self::expanded_boundary_len).sum();
                    let tags = self.expand_tag_spec(&tag_spec, total)?;
                    let mut tag_iter = tags.into_iter();
                    for node in vec.iter() {
                        let count = Self::expanded_boundary_len(node);
                        let node_tag = match count {
                            0 => None,
                            1 => Some(ConstraintTagSpec::Single(
                                tag_iter
                                    .next()
                                    .expect("tag length validated against expanded length"),
                            )),
                            _ => {
                                let mut tags = Vec::with_capacity(count);
                                for _ in 0..count {
                                    tags.push(
                                        tag_iter
                                            .next()
                                            .expect("tag length validated against expanded length")
                                            .item,
                                    );
                                }
                                Some(ConstraintTagSpec::List { span: tag_spec.span(), tags })
                            },
                        };
                        self.build_boundary_constraint(node, node_tag)?;
                    }
                } else {
                    for node in vec.iter() {
                        self.build_boundary_constraint(node, None)?;
                    }
                }
                Ok(())
            },
            Op::Matrix(matrix) => {
                let rows = matrix.elements.borrow().deref().clone();
                if let Some(tag_spec) = tag {
                    // Matrices are vectors of vectors; tag slicing follows the fully expanded
                    // row-major order.
                    let total = rows
                        .iter()
                        .map(|row| {
                            let vec = row.borrow().deref().children().borrow().deref().clone();
                            vec.iter().map(Self::expanded_boundary_len).sum::<usize>()
                        })
                        .sum::<usize>();
                    let tags = self.expand_tag_spec(&tag_spec, total)?;
                    let mut tag_iter = tags.into_iter();
                    for row in rows.iter() {
                        let vec = row.borrow().deref().children().borrow().deref().clone();
                        for node in vec.iter() {
                            let count = Self::expanded_boundary_len(node);
                            let node_tag = match count {
                                0 => None,
                                1 => Some(ConstraintTagSpec::Single(
                                    tag_iter
                                        .next()
                                        .expect("tag length validated against expanded length"),
                                )),
                                _ => {
                                    let mut tags = Vec::with_capacity(count);
                                    for _ in 0..count {
                                        tags.push(
                                            tag_iter
                                                .next()
                                                .expect(
                                                    "tag length validated against expanded length",
                                                )
                                                .item,
                                        );
                                    }
                                    Some(ConstraintTagSpec::List { span: tag_spec.span(), tags })
                                },
                            };
                            self.build_boundary_constraint(node, node_tag)?;
                        }
                    }
                } else {
                    for row in rows.iter() {
                        let vec = row.borrow().deref().children().borrow().deref().clone();
                        for node in vec.iter() {
                            self.build_boundary_constraint(node, None)?;
                        }
                    }
                }
                Ok(())
            },
            Op::Enf(enf) => {
                let child_op = enf.expr.clone();
                let child_op = accessor_to_scalar(&child_op);
                let next_tag = self.merge_tag_spec(enf.tag.clone(), tag)?;

                if child_op.as_vector().is_some() || child_op.as_matrix().is_some() {
                    return self.build_boundary_constraint(&child_op, next_tag);
                }

                let child_op = vec_to_scalar(&child_op);
                self.build_boundary_constraint(&child_op, next_tag)?;
                Ok(())
            },
            Op::Sub(sub) => {
                // Check that lhs is a Bounded trace access
                let lhs = sub.lhs.clone();
                let lhs = accessor_to_scalar(&lhs);
                let lhs = vec_to_scalar(&lhs);
                let rhs = sub.rhs.clone();
                let rhs = accessor_to_scalar(&rhs);
                let rhs = vec_to_scalar(&rhs);
                let lhs_span = lhs.span();
                let rhs_span = rhs.span();

                let boundary = lhs.as_boundary().unwrap().clone();

                let trace_access = self.extract_trace_from_boundary(boundary.clone())?;

                self.mark_constrained_boundary(trace_access, &boundary)?;

                let lhs = self.insert_trace_access_value(trace_access);
                let rhs = self.insert_mir_operation(&rhs)?;

                // Compare the inferred trace segment and domain of the operands
                let domain = boundary.kind.into();
                {
                    let graph = self.air.constraint_graph();
                    let (lhs_segment, lhs_domain) = graph.node_details(&lhs, domain)?;
                    let (rhs_segment, rhs_domain) = graph.node_details(&rhs, domain)?;
                    if lhs_segment < rhs_segment {
                        // trace segment inference defaults to the lowest segment (the main trace)
                        // and is adjusted according to the use of random
                        // values and trace columns.
                        let lhs_segment_name = self.trace_columns[lhs_segment].name;
                        let rhs_segment_name = self.trace_columns[rhs_segment].name;
                        self.diagnostics.diagnostic(Severity::Error)
                                    .with_message("invalid boundary constraint")
                                    .with_primary_label(lhs_span, format!("this constrains a column in the '{lhs_segment_name}' trace segment"))
                                    .with_secondary_label(rhs_span, format!("but this expression implies the '{rhs_segment_name}' trace segment"))
                                    .with_note("Boundary constraints require both sides of the constraint to apply to the same trace segment.")
                                    .emit();
                        return Err(CompileError::Failed);
                    }
                    if lhs_domain != rhs_domain {
                        self.diagnostics.diagnostic(Severity::Error)
                                    .with_message("invalid boundary constraint")
                                    .with_primary_label(lhs_span, format!("this has a constraint domain of {lhs_domain}"))
                                    .with_secondary_label(rhs_span, format!("this has a constraint domain of {rhs_domain}"))
                                    .with_note("Boundary constraints require both sides of the constraint to be in the same domain.")
                                    .emit();
                        return Err(CompileError::Failed);
                    }
                }

                // Merge the expressions into a single constraint
                let root = self.insert_op(Operation::Sub(lhs, rhs));

                let tag = self.resolve_single_tag(tag)?;

                // Store the generated constraint
                self.air.constraints.insert_constraint(trace_access.segment, root, domain, tag);
                Ok(())
            },
            Op::Boundary(boundary) => {
                let trace_access = self.extract_trace_from_boundary(boundary.clone())?;

                self.mark_constrained_boundary(trace_access, boundary)?;

                let root = self.insert_trace_access_value(trace_access);

                let domain = boundary.kind.into();

                let tag = self.resolve_single_tag(tag)?;

                // Store the generated constraint
                self.air.constraints.insert_constraint(trace_access.segment, root, domain, tag);
                Ok(())
            },
            _ => unreachable!(),
        }
    }

    fn build_integrity_constraint(
        &mut self,
        ic: &Link<Op>,
        tag: Option<ConstraintTagSpec>,
    ) -> Result<(), CompileError> {
        match ic.borrow().deref() {
            Op::Vector(vector) => {
                let vec = vector.children().borrow().deref().clone();
                if let Some(tag_spec) = tag {
                    // Bus ops are expanded into their own constraints later; do not consume tags
                    // here. Tag slicing follows the expanded constraint list.
                    let total =
                        vec.iter().map(|node| self.expanded_integrity_len(node)).sum::<usize>();
                    let tags = self.expand_tag_spec(&tag_spec, total)?;
                    let mut tag_iter = tags.into_iter();
                    for node in vec.iter() {
                        let count = self.expanded_integrity_len(node);
                        let node_tag = match count {
                            0 => None,
                            1 => Some(ConstraintTagSpec::Single(
                                tag_iter
                                    .next()
                                    .expect("tag length validated against expanded length"),
                            )),
                            _ => {
                                let mut tags = Vec::with_capacity(count);
                                for _ in 0..count {
                                    tags.push(
                                        tag_iter
                                            .next()
                                            .expect("tag length validated against expanded length")
                                            .item,
                                    );
                                }
                                Some(ConstraintTagSpec::List { span: tag_spec.span(), tags })
                            },
                        };
                        self.build_integrity_constraint(node, node_tag)?;
                    }
                } else {
                    for node in vec.iter() {
                        self.build_integrity_constraint(node, None)?;
                    }
                }
            },
            Op::Matrix(matrix) => {
                let rows = matrix.elements.borrow().deref().clone();
                if let Some(tag_spec) = tag {
                    // Matrices are vectors of vectors; tag slicing follows the fully expanded
                    // row-major order (excluding bus ops).
                    let total = rows
                        .iter()
                        .map(|row| {
                            let vec = row.borrow().deref().children().borrow().deref().clone();
                            vec.iter().map(|node| self.expanded_integrity_len(node)).sum::<usize>()
                        })
                        .sum::<usize>();
                    let tags = self.expand_tag_spec(&tag_spec, total)?;
                    let mut tag_iter = tags.into_iter();
                    for row in rows.iter() {
                        let vec = row.borrow().deref().children().borrow().deref().clone();
                        for node in vec.iter() {
                            let count = self.expanded_integrity_len(node);
                            let node_tag = match count {
                                0 => None,
                                1 => Some(ConstraintTagSpec::Single(
                                    tag_iter
                                        .next()
                                        .expect("tag length validated against expanded length"),
                                )),
                                _ => {
                                    let mut tags = Vec::with_capacity(count);
                                    for _ in 0..count {
                                        tags.push(
                                            tag_iter
                                                .next()
                                                .expect(
                                                    "tag length validated against expanded length",
                                                )
                                                .item,
                                        );
                                    }
                                    Some(ConstraintTagSpec::List { span: tag_spec.span(), tags })
                                },
                            };
                            self.build_integrity_constraint(node, node_tag)?;
                        }
                    }
                } else {
                    for row in rows.iter() {
                        let vec = row.borrow().deref().children().borrow().deref().clone();
                        for node in vec.iter() {
                            self.build_integrity_constraint(node, None)?;
                        }
                    }
                }
            },
            Op::Enf(enf) => {
                let child_op = enf.expr.clone();
                let child_op = accessor_to_scalar(&child_op);
                let enf_tag = self.merge_tag_spec(enf.tag.clone(), tag)?;

                let child_op = enf_to_scalar(&child_op);
                if child_op.as_vector().is_some() || child_op.as_matrix().is_some() {
                    return self.build_integrity_constraint(&child_op, enf_tag);
                }

                let child_op = vec_to_scalar(&child_op);
                match child_op.clone().borrow().deref() {
                    Op::Sub(_sub) => {
                        self.build_integrity_constraint(&child_op, enf_tag)?;
                    },
                    Op::BusOp(bus_op) => {
                        if enf_tag.is_some() {
                            self.diagnostics
                                .diagnostic(Severity::Error)
                                .with_message("bus constraints do not support @tag")
                                .with_primary_label(
                                    enf.span(),
                                    "remove the tag or move it to a non-bus constraint",
                                )
                                .emit();
                            return Err(CompileError::Failed);
                        }
                        let bus = bus_op.bus.to_link().unwrap();
                        let latch = bus_op.latch.clone();

                        bus.borrow_mut().latches.push(latch.clone());
                        bus.borrow_mut().columns.push(child_op.clone());
                    },
                    _ => {
                        let root = self.insert_mir_operation(&child_op)?;
                        let (trace_segment, domain) = self
                            .air
                            .constraint_graph()
                            .node_details(&root, ConstraintDomain::EveryRow)?;
                        let tag = self.resolve_single_tag(enf_tag)?;
                        self.air.constraints.insert_constraint(trace_segment, root, domain, tag);
                    },
                }
            },
            Op::Sub(sub) => {
                let lhs = sub.lhs.clone();
                let rhs = sub.rhs.clone();
                let lhs_node_index = self.insert_mir_operation(&lhs)?;
                let rhs_node_index = self.insert_mir_operation(&rhs)?;
                let root = self.insert_op(Operation::Sub(lhs_node_index, rhs_node_index));
                let (trace_segment, domain) =
                    self.air.constraint_graph().node_details(&root, ConstraintDomain::EveryRow)?;
                let tag = self.resolve_single_tag(tag)?;
                self.air.constraints.insert_constraint(trace_segment, root, domain, tag);
            },
            _ => unreachable!("Unexpected integrity constraint root: {:?}", ic),
        }
        Ok(())
    }

    // Returns the number of boundary constraints produced by `node` after expansion.
    fn expanded_boundary_len(node: &Link<Op>) -> usize {
        match node.borrow().deref() {
            Op::Vector(vector) => {
                vector.elements.borrow().iter().map(Self::expanded_boundary_len).sum()
            },
            Op::Matrix(matrix) => matrix
                .elements
                .borrow()
                .iter()
                .map(|row| {
                    let vec = row.borrow().deref().children().borrow().deref().clone();
                    vec.iter().map(Self::expanded_boundary_len).sum::<usize>()
                })
                .sum(),
            Op::Enf(enf) => {
                let child = accessor_to_scalar(&enf.expr);
                if child.as_vector().is_some() || child.as_matrix().is_some() {
                    Self::expanded_boundary_len(&child)
                } else {
                    1
                }
            },
            // After unrolling/constant propagation, any remaining op types should be scalars,
            // and accessors resolve to scalar values. These contribute exactly one constraint.
            _ => 1,
        }
    }

    // Returns the number of integrity constraints produced by `node` after expansion.
    // Bus ops are excluded because they become standalone constraints later.
    fn expanded_integrity_len(&self, node: &Link<Op>) -> usize {
        if self.is_bus_op_node(node) {
            return 0;
        }
        match node.borrow().deref() {
            Op::Vector(vector) => vector
                .children()
                .borrow()
                .iter()
                .map(|child| self.expanded_integrity_len(child))
                .sum(),
            Op::Matrix(matrix) => matrix
                .elements
                .borrow()
                .iter()
                .map(|row| {
                    let vec = row.borrow().deref().children().borrow().deref().clone();
                    vec.iter().map(|child| self.expanded_integrity_len(child)).sum::<usize>()
                })
                .sum(),
            Op::Enf(enf) => {
                let child = accessor_to_scalar(&enf.expr);
                if self.is_bus_op_node(&child) {
                    0
                } else if child.as_vector().is_some() || child.as_matrix().is_some() {
                    self.expanded_integrity_len(&child)
                } else {
                    1
                }
            },
            // By this stage we expect only scalar ops (no unrolled vectors/matrices/for/if),
            // so remaining nodes map to a single integrity constraint.
            _ => 1,
        }
    }

    fn is_bus_op_node(&self, node: &Link<Op>) -> bool {
        let node = accessor_to_scalar(node);
        let node = enf_to_scalar(&node);
        matches!(node.borrow().deref(), Op::BusOp(_))
    }

    fn merge_tag_spec(
        &self,
        tag: Option<ConstraintTagSpec>,
        parent_tag: Option<ConstraintTagSpec>,
    ) -> Result<Option<ConstraintTagSpec>, CompileError> {
        match (tag, parent_tag) {
            (Some(tag), Some(parent)) => {
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("multiple tags applied to a constraint")
                    .with_primary_label(tag.span(), "tag applied here")
                    .with_secondary_label(parent.span(), "and here")
                    .emit();
                Err(CompileError::Failed)
            },
            (Some(tag), None) => Ok(Some(tag)),
            (None, Some(tag)) => Ok(Some(tag)),
            (None, None) => Ok(None),
        }
    }

    fn expand_tag_spec(
        &self,
        tag: &ConstraintTagSpec,
        expected_len: usize,
    ) -> Result<Vec<Span<u64>>, CompileError> {
        let tags = tag.expand_spans();
        if tags.len() != expected_len {
            self.diagnostics
                .diagnostic(Severity::Error)
                .with_message("constraint tag count does not match expanded constraint length")
                .with_primary_label(
                    tag.span(),
                    format!("expected {expected_len} tags, got {}", tags.len()),
                )
                .emit();
            return Err(CompileError::Failed);
        }
        Ok(tags)
    }

    fn resolve_single_tag(
        &mut self,
        tag: Option<ConstraintTagSpec>,
    ) -> Result<Option<u64>, CompileError> {
        match tag {
            None => Ok(None),
            Some(spec) => match spec.as_single() {
                Some(tag) => Ok(Some(tag)),
                None => {
                    let next = self.next_tag_from_spec(&spec)?;
                    Ok(Some(next.item))
                },
            },
        }
    }

    fn next_tag_from_spec(&mut self, spec: &ConstraintTagSpec) -> Result<Span<u64>, CompileError> {
        let span = spec.span();
        let entry = self
            .tag_allocators
            .entry(span)
            .or_insert_with(|| TagAllocator::new(spec.clone()));

        if let Some(tag) = entry.next() {
            Ok(tag)
        } else {
            self.diagnostics
                .diagnostic(Severity::Error)
                .with_message("constraint tag range exhausted")
                .with_primary_label(span, "no tags left to assign here")
                .emit();
            Err(CompileError::Failed)
        }
    }

    /// Builds the bus struct, containing the bus operations and boundaries.
    fn build_bus(&mut self, mir_bus: &Link<mir::ir::Bus>) -> Result<(), CompileError> {
        let mir_bus = mir_bus.borrow();

        let first = build_bus_boundary(self.diagnostics, mir_bus.span(), &mir_bus.get_first())?;
        let last = build_bus_boundary(self.diagnostics, mir_bus.span(), &mir_bus.get_last())?;

        let mut bus_ops = vec![];
        for (mir_column, mir_latch) in mir_bus.columns.iter().zip(mir_bus.latches.iter()) {
            let mut column = vec![];

            // Note: we have checked this will not panic in the MIR pass
            let mir_bus_op = mir_column.as_bus_op().expect("Bus column should be a bus operation");
            let mir_bus_op_args = mir_bus_op.args.clone();
            for arg in mir_bus_op_args.iter() {
                let arg = self.insert_mir_operation(arg)?;
                column.push(arg);
            }
            let latch = self.insert_mir_operation(mir_latch)?;

            let bus_op = BusOp::new(column, latch, mir_bus_op.kind);
            bus_ops.push(bus_op);
        }
        self.air.buses.insert(
            mir_bus.name(),
            Bus::new(
                mir_bus.name(),
                mir_bus.bus_type,
                mir_bus.constraint_form,
                first,
                last,
                bus_ops,
                mir_bus.first_tag(),
                mir_bus.last_tag(),
                mir_bus.transition_tag(),
            ),
        );
        Ok(())
    }

    /// Adds the specified operation to the graph and returns the index of its node.
    #[inline]
    fn insert_op(&mut self, op: Operation) -> NodeIndex {
        self.air.constraint_graph_mut().insert_node_cached(op, &mut self.air_op_cache)
    }

    fn insert_trace_access_value(&mut self, trace_access: MirTraceAccess) -> NodeIndex {
        self.insert_op(Operation::Value(crate::ir::Value::TraceAccess(crate::ir::TraceAccess {
            segment: trace_access.segment,
            column: trace_access.column,
            row_offset: trace_access.row_offset,
        })))
    }

    /// Extracts the trace access information from a given [Mir] `Boundary`.
    /// Returns a [Mir] `TraceAccess` with the corresponding segment id and column if the boundary
    /// wraps a valid trace access column, or raises a diagnostic if the trace access has a size
    /// greater than 1.
    ///
    /// Note: the boundary expression must only reference the constrained trace access, not the
    /// whole boundary constraint expression.
    ///
    /// # Panics
    /// Panics if the boundary does not wrap a trace access column, which should have been caught
    /// during semantic analysis.
    fn extract_trace_from_boundary(
        &self,
        boundary: MirBoundary,
    ) -> Result<MirTraceAccess, CompileError> {
        let Op::Value(value) = boundary.expr.borrow().deref().clone() else {
            unreachable!(); // Raise diag
        };

        let trace_access = match value.value.clone() {
            SpannedMirValue {
                value: MirValue::TraceAccess(trace_access),
                ..
            } => trace_access,
            SpannedMirValue {
                value: MirValue::TraceAccessBinding(trace_access_binding),
                span,
            } => {
                if trace_access_binding.size != 1 {
                    self.diagnostics.diagnostic(Severity::Error)
                                .with_message("invalid boundary constraint")
                                .with_primary_label(span, "this has a trace access binding with a size greater than 1")
                                .with_note("Boundary constraints require both sides of the constraint to be single columns.")
                                .emit();
                    return Err(CompileError::Failed);
                }
                MirTraceAccess {
                    segment: trace_access_binding.segment,
                    column: trace_access_binding.offset,
                    row_offset: 0,
                }
            },
            SpannedMirValue {
                value: MirValue::BusAccess(bus_access), ..
            } => {
                let bus = bus_access.bus;
                let name = bus.borrow().deref().name();
                let column = self.bus_bindings_map.get(&name).unwrap();
                MirTraceAccess::new(TraceSegmentId::Aux, *column, bus_access.row_offset)
            },
            _ => unreachable!("Expected TraceAccess or BusAccess, received {:?}", value.value), /* Raise diag */
        };

        Ok(trace_access)
    }

    /// Marks a boundary as constrained by the given trace access information.
    /// This is used to ensure that we do not insert duplicate boundary constraints in the graph.
    fn mark_constrained_boundary(
        &mut self,
        trace_access: MirTraceAccess,
        boundary: &MirBoundary,
    ) -> Result<(), CompileError> {
        if let Some(prev) = self.trace_columns[trace_access.segment].mark_constrained(
            boundary.span(),
            trace_access.column,
            boundary.kind,
        ) {
            self.diagnostics
                .diagnostic(Severity::Error)
                .with_message("overlapping boundary constraints")
                .with_primary_label(
                    boundary.span(),
                    "this constrains a column and boundary that has already been constrained",
                )
                .with_secondary_label(prev, "previous constraint occurs here")
                .emit();
            return Err(CompileError::Failed);
        }
        Ok(())
    }
}

// HELPERS FUNCTIONS
// ================================================================================================

/// Helper function to convert a MIR bus boundary node into an AIR bus boundary.
fn build_bus_boundary(
    diagnostics: &DiagnosticsHandler,
    bus_span: SourceSpan,
    mir_bus_boundary_node: &Link<Op>,
) -> Result<BusBoundary, CompileError> {
    let mir_node = vec_to_scalar(mir_bus_boundary_node);
    let mir_node_ref = mir_node.borrow();
    match mir_node_ref.deref() {
        Op::Value(value) => match &value.value.value {
            // This represents public input table boundary
            MirValue::PublicInputTable(public_input_table) => Ok(
                crate::ir::BusBoundary::PublicInputTable(crate::ir::PublicInputTableAccess::new(
                    public_input_table.table_name,
                    public_input_table.num_cols,
                    public_input_table.bus_type(),
                )),
            ),
            // This represents an empty bus
            MirValue::Null => Ok(crate::ir::BusBoundary::Null),
            MirValue::Unconstrained => Ok(crate::ir::BusBoundary::Unconstrained),
            _ => Err(CompileError::Failed),
        },
        Op::None(_) => {
            diagnostics
                .diagnostic(Severity::Error)
                .with_message("invalid bus boundary")
                .with_primary_label(bus_span, "this bus has unconstrained boundaries")
                .with_note(
                    "Bus boundaries must be either a public input table or null for empty buses.",
                )
                .emit();
            Err(CompileError::Failed)
        },
        _ => unreachable!("Unexpected Mir Op in bus boundary: {:#?}", mir_node_ref),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_allocator_consumes_range_in_order() {
        let span = SourceSpan::default();
        let spec = ConstraintTagSpec::range(span, 10, 13, false); // 10..13 -> 10,11,12
        let mut alloc = TagAllocator::new(spec);

        let first = alloc.next().map(|tag| tag.item);
        let second = alloc.next().map(|tag| tag.item);
        let third = alloc.next().map(|tag| tag.item);
        let fourth = alloc.next().map(|tag| tag.item);

        assert_eq!(first, Some(10));
        assert_eq!(second, Some(11));
        assert_eq!(third, Some(12));
        assert_eq!(fourth, None);
    }

    #[test]
    fn tag_allocator_consumes_list_in_order() {
        let span = SourceSpan::default();
        let spec = ConstraintTagSpec::list(span, vec![3, 1, 4]);
        let mut alloc = TagAllocator::new(spec);

        let first = alloc.next().map(|tag| tag.item);
        let second = alloc.next().map(|tag| tag.item);
        let third = alloc.next().map(|tag| tag.item);
        let fourth = alloc.next().map(|tag| tag.item);

        assert_eq!(first, Some(3));
        assert_eq!(second, Some(1));
        assert_eq!(third, Some(4));
        assert_eq!(fourth, None);
    }
}
