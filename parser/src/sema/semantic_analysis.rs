use std::{
    collections::{HashMap, HashSet},
    fmt, mem,
    ops::ControlFlow,
};

use miden_diagnostics::{DiagnosticsHandler, Severity, SourceSpan, Span, Spanned};

use crate::{
    ast::{visit::VisitMut, *},
    symbols::{self, Symbol},
};

use super::Imported;

/// Represents the graph of dependencies between module items and items they
/// reference, either in the same module, or via imports.
///
/// The dependency graph is used to construct the final [Program] representation,
/// containing only those parts of the program which are referenced from the root
/// module.
pub type DependencyGraph = petgraph::graphmap::DiGraphMap<QualifiedIdentifier, DependencyType>;

/// Represents the graph of dependencies between modules, with no regard to what
/// items in those modules are actually used. In other words, this graph tells us
/// what modules depend on what other modules in the program.
pub type ModuleGraph = petgraph::graphmap::DiGraphMap<ModuleId, ()>;

/// Represents the type of edges in the dependency graph
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DependencyType {
    /// Depends on an imported constant
    Constant,
    /// Depends on an imported evaluator function
    Evaluator,
    /// Depends on an imported function
    Function,
    /// Depends on a periodic_columns declaration (not visible as an export)
    PeriodicColumn,
}

/// Represents the type signature of a function
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionType {
    /// An evaluator function, which has no results, and has
    /// a complex type signature due to the nature of trace bindings
    Evaluator(Vec<TraceSegment>),
    /// A standard function with one or more inputs, and a result
    #[allow(dead_code)]
    Function(Vec<Type>, Type),
}
impl FunctionType {
    pub fn result(&self) -> Option<Type> {
        match self {
            Self::Evaluator(_) => None,
            Self::Function(_, result) => Some(*result),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Spanned)]
pub struct TraceRef {
    /// The span of the declaration this is derived from
    #[span]
    pub span: SourceSpan,
    pub segment: TraceSegmentId,
    pub index: usize,
    pub size: usize,
}
impl TraceRef {
    /// Returns a [Type] that describes what type of value this binding represents
    pub fn ty(&self) -> Type {
        if self.size == 1 {
            Type::Felt
        } else {
            Type::Vector(self.size)
        }
    }

    /// Derive a new [TraceBinding] derived from the current one given an [AccessType]
    pub fn access(&self, access_type: AccessType) -> Result<Self, InvalidAccessError> {
        match access_type {
            AccessType::Default => Ok(*self),
            AccessType::Slice(_) if self.size == 1 => Err(InvalidAccessError::SliceOfScalar),
            AccessType::Slice(range) if range.end > self.size => {
                Err(InvalidAccessError::IndexOutOfBounds)
            }
            AccessType::Slice(range) => {
                let index = self.index + range.start;
                Ok(Self {
                    index,
                    size: range.end - range.start,
                    ..*self
                })
            }
            AccessType::Index(_) if self.size == 1 => Err(InvalidAccessError::IndexIntoScalar),
            AccessType::Index(idx) if idx >= self.size => Err(InvalidAccessError::IndexOutOfBounds),
            AccessType::Index(idx) => {
                let index = self.index + idx;
                Ok(Self {
                    index,
                    size: 1,
                    ..*self
                })
            }
            AccessType::Matrix(_, _) => Err(InvalidAccessError::IndexIntoScalar),
        }
    }
}

/// This type provides type and contextual information about a binding,
/// i.e. not only does it tell us the type of a binding, but what type
/// of value was bound. This is used during analysis to check whether a
/// particular access is valid for the context it is in, as well as to
/// propagate type information while retaining information about where
/// the type was derived from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingType {
    /// A local variable whose value is not an alias of a global/module declaration
    Local(Type),
    /// A local variable that aliases a global/module declaration
    Alias(Box<BindingType>),
    /// A direct reference to a constant declaration
    Constant(Type),
    /// A type associated with a function signature
    ///
    /// The result type is None if the function is an evaluator
    Function(FunctionType),
    /// A function parameter corresponding to trace columns
    TraceParam(TraceRef),
    /// A direct reference to one or more contiguous trace columns
    TraceColumn(TraceRef),
    /// A potentially non-contiguous set of trace columns
    Vector(Vec<BindingType>),
    /// A direct reference to a random value binding
    RandomValue(RandBinding),
    /// A direct reference to a public input
    PublicInput(Type),
    /// A direct reference to a periodic column
    PeriodicColumn(usize),
}
impl BindingType {
    /// Get the value type of this binding, if applicable
    pub fn ty(&self) -> Option<Type> {
        match self {
            Self::TraceColumn(tb) | Self::TraceParam(tb) => Some(tb.ty()),
            Self::Vector(elems) => Some(Type::Vector(elems.len())),
            Self::RandomValue(rb) => Some(rb.ty()),
            Self::Alias(aliased) => aliased.ty(),
            Self::Local(ty) | Self::Constant(ty) | Self::PublicInput(ty) => Some(*ty),
            Self::PeriodicColumn(_) => Some(Type::Felt),
            Self::Function(ty) => ty.result(),
        }
    }

    pub fn access(&self, access_type: AccessType) -> Result<Self, InvalidAccessError> {
        match self {
            Self::Alias(aliased) => aliased.access(access_type),
            Self::Local(ty) => ty.access(access_type).map(Self::Local),
            Self::Constant(ty) => ty
                .access(access_type)
                .map(|t| Self::Alias(Box::new(Self::Constant(t)))),
            Self::TraceColumn(tb) => tb.access(access_type).map(Self::TraceColumn),
            Self::TraceParam(tb) => tb.access(access_type).map(Self::TraceParam),
            Self::Vector(elems) => match access_type {
                AccessType::Default => Ok(Self::Vector(elems.clone())),
                AccessType::Index(idx) if idx >= elems.len() => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                }
                AccessType::Index(idx) => Ok(elems[idx].clone()),
                AccessType::Slice(range) if range.end > elems.len() => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                }
                AccessType::Slice(range) => {
                    Ok(Self::Vector(elems[range.start..range.end].to_vec()))
                }
                AccessType::Matrix(row, _) if row >= elems.len() => {
                    Err(InvalidAccessError::IndexOutOfBounds)
                }
                AccessType::Matrix(row, col) => elems[row].access(AccessType::Index(col)),
            },
            Self::RandomValue(tb) => tb
                .access(access_type)
                .map(|tb| Self::Alias(Box::new(Self::RandomValue(tb)))),
            Self::PublicInput(ty) => ty.access(access_type).map(Self::PublicInput),
            Self::PeriodicColumn(period) => match access_type {
                AccessType::Default => Ok(Self::PeriodicColumn(*period)),
                _ => Err(InvalidAccessError::IndexIntoScalar),
            },
            Self::Function(_) => Err(InvalidAccessError::InvalidBinding),
        }
    }
}
impl fmt::Display for BindingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Alias(aliased) => write!(f, "{}", aliased),
            Self::Local(_) => f.write_str("local"),
            Self::Constant(_) => f.write_str("constant"),
            Self::Vector(_) => f.write_str("vector"),
            Self::Function(_) => f.write_str("function"),
            Self::TraceColumn(_) | Self::TraceParam(_) => f.write_str("trace column(s)"),
            Self::RandomValue(_) => f.write_str("random value(s)"),
            Self::PublicInput(_) => f.write_str("public input(s)"),
            Self::PeriodicColumn(_) => f.write_str("periodic column(s)"),
        }
    }
}

/// A helper enum for representing what constraint mode is active
#[derive(Copy, Clone, PartialEq, Eq)]
enum AllowedConstraintsType {
    /// A context in which constraints are not permitted
    None,
    /// A context in which only boundary constraints are permitted
    Boundary,
    /// A context in which only integrity constraints are permitted
    Integrity,
}

/// This pass is used to perform a variety of semantic analysis tasks in a single traversal of a module AST
///
/// * Resolves all identifiers to their fully-qualified names, or raises appropriate errors if unable
/// * Warns/errors as appropriate when declarations/bindings shadow or conflict with previous declarations/bindings
/// * Assigns binding context and type information to identifiers, or raises appropriate errors if unable
/// * Performs type checking
/// * Tracks references to imported items, and updates the dependency graph with that information
/// * Ensures constraints are valid in the context they are defined in
/// * Verifies comprehension invariants
///
/// These could each be done as separate passes, but since we don't have good facilities presently for fusing
/// multiple traversals into a single traversal, or for sharing analyses, it is best for us to take advantage
/// of the information being gathered to perform many of these tasks simultaneously.
pub struct SemanticAnalysis<'a> {
    diagnostics: &'a DiagnosticsHandler,
    program: &'a Program,
    library: &'a Library,
    deps: &'a mut DependencyGraph,
    imported: Imported,
    globals: HashMap<Identifier, BindingType>,
    locals: HashMap<NamespacedIdentifier, BindingType>,
    referenced: HashMap<QualifiedIdentifier, DependencyType>,
    current_module: Option<ModuleId>,
    allowed_constraints: AllowedConstraintsType,
    has_undefined_variables: bool,
    has_type_errors: bool,
    in_constraint_comprehension: bool,
}
impl<'a> SemanticAnalysis<'a> {
    /// Create a new instance of the semantic analyzer
    pub fn new(
        diagnostics: &'a DiagnosticsHandler,
        program: &'a Program,
        library: &'a Library,
        deps: &'a mut DependencyGraph,
        imported: Imported,
    ) -> Self {
        Self {
            diagnostics,
            program,
            library,
            deps,
            imported,
            globals: Default::default(),
            locals: Default::default(),
            referenced: Default::default(),
            current_module: None,
            allowed_constraints: AllowedConstraintsType::None,
            has_undefined_variables: false,
            has_type_errors: false,
            in_constraint_comprehension: false,
        }
    }

    /// Run semantic analysis on the given module
    pub fn run(mut self, module: &mut Module) -> Result<(), ModuleError> {
        if let ControlFlow::Break(err) = self.visit_mut_module(module) {
            return Err(err);
        }

        // If this is the root module, we may have top-level dependencies
        if module.name == self.program.name {
            // Update the dependency graph with the collected information
            //
            // We use a special node to represent the references which occur in
            // the top-level boundary_constraints and integrity_constraints sections
            let root_node = QualifiedIdentifier::new(
                self.program.name,
                NamespacedIdentifier::Binding(Identifier::new(
                    SourceSpan::UNKNOWN,
                    Symbol::intern("$$root"),
                )),
            );
            for (referenced_item, ref_type) in self.referenced.iter() {
                let referenced_item = self.deps.add_node(*referenced_item);
                self.deps.add_edge(root_node, referenced_item, *ref_type);
            }
        } else {
            // We should never have top-level dependencies here
            assert!(
                self.referenced.is_empty(),
                "it should be impossible to have import references here"
            );
        }

        Ok(())
    }
}

impl<'a> VisitMut<ModuleError> for SemanticAnalysis<'a> {
    fn visit_mut_module(&mut self, module: &mut Module) -> ControlFlow<ModuleError> {
        self.current_module = Some(module.name);

        // Register all globals implicitly defined in the module before all locally bound names
        //
        // Currently this consists only of the `random_values` declarations.
        //
        // Because a module is guaranteed to have no top-level name conflicts when parsed successfully,
        // we know that all of the globally visible declarations from the root module cannot conflict
        // with each other, but we assert that this is so to catch any potentially invalid modules that
        // bypassed that validation somehow.
        if let Some(rv) = self.program.random_values.as_ref() {
            assert_eq!(
                self.globals.insert(
                    rv.name,
                    BindingType::RandomValue(RandBinding::new(
                        rv.name.span(),
                        rv.name,
                        rv.size,
                        0,
                        Type::Vector(rv.size)
                    ))
                ),
                None
            );
            for binding in rv.bindings.iter().copied() {
                assert_eq!(
                    self.globals
                        .insert(binding.name, BindingType::RandomValue(binding)),
                    None
                );
            }
        }

        // Next, add all the top-level root module declarations as locals, if this is the root module
        //
        // As above, we are guaranteed that these names have no conflicts, but we assert that anyway
        if module.is_root() {
            for segment in self.program.trace_columns.iter() {
                assert_eq!(
                    self.locals.insert(
                        NamespacedIdentifier::Binding(segment.name),
                        BindingType::TraceColumn(TraceBinding {
                            span: segment.span(),
                            segment: segment.id,
                            name: Some(segment.name),
                            offset: 0,
                            size: segment.size,
                            ty: Type::Vector(segment.size),
                        })
                    ),
                    None
                );
                for binding in segment.bindings.iter().copied() {
                    assert_eq!(
                        self.locals.insert(
                            NamespacedIdentifier::Binding(binding.name.unwrap()),
                            BindingType::TraceColumn(TraceBinding {
                                span: segment.name.span(),
                                segment: segment.id,
                                name: binding.name,
                                offset: binding.offset,
                                size: binding.size,
                                ty: binding.ty,
                            })
                        ),
                        None
                    );
                }
            }
            for input in self.program.public_inputs.values() {
                assert_eq!(
                    self.locals.insert(
                        NamespacedIdentifier::Binding(input.name),
                        BindingType::PublicInput(Type::Vector(input.size))
                    ),
                    None
                );
            }
        }

        // Next, add all the top-level declarations which may conflict with wildcard imports,
        // namely constants and functions. These are guaranteed not to conflict with top-level
        // declarations in this module, but we only just performed import resolution, so we have
        // not yet been able to guarantee that there are no conflicts with those imported symbols.

        // First, constants
        for constant in module.constants.values() {
            let namespaced_name = NamespacedIdentifier::Binding(constant.name);
            // See if a constant with the same name was previously imported
            if let Some((prev, _)) = self.imported.get_key_value(&namespaced_name) {
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("declaration conflicts with an imported item")
                    .with_primary_label(constant.span(), "this name is already in use")
                    .with_secondary_label(prev.span(), "it was declared via this import")
                    .emit();
                return ControlFlow::Break(ModuleError::NameConflict(constant.name.span()));
            }
            // It should be impossible for there to be a local by this name at this point
            assert_eq!(
                self.locals
                    .insert(namespaced_name, BindingType::Constant(constant.ty())),
                None
            );
        }

        // Next, functions.
        //
        // Functions are in their own namespace, but may conflict with imported items
        for (function_name, function) in module.evaluators.iter() {
            let namespaced_name = NamespacedIdentifier::Function(*function_name);
            if let Some((prev, _)) = self.imported.get_key_value(&namespaced_name) {
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("declaration conflicts with an imported item")
                    .with_primary_label(namespaced_name.span(), "this name is already in use")
                    .with_secondary_label(prev.span(), "it was declared via this import")
                    .emit();
                return ControlFlow::Break(ModuleError::NameConflict(function_name.span()));
            }
            assert_eq!(
                self.locals.insert(
                    namespaced_name,
                    BindingType::Function(FunctionType::Evaluator(function.params.clone()))
                ),
                None
            );
        }

        // Next, we add any periodic columns to the set of local bindings.
        //
        // These _can_ conflict with globally defined names, but are guaranteed not to conflict
        // with other module-local declarations.
        for periodic in module.periodic_columns.values() {
            if let Some((prev, prev_binding)) = self.globals.get_key_value(&periodic.name) {
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message(format!(
                        "periodic column conflicts with {} declared in root module",
                        prev_binding
                    ))
                    .with_primary_label(periodic.name.span(), "this name is already in use")
                    .with_secondary_label(prev.span(), "previously declared here")
                    .emit();
                return ControlFlow::Break(ModuleError::Invalid);
            }
            assert_eq!(
                self.locals.insert(
                    NamespacedIdentifier::Binding(periodic.name),
                    BindingType::PeriodicColumn(periodic.period())
                ),
                None
            );
        }

        // From this point forward, we use the standard visitor traversal to visit every node
        // which can reference an identifier, and rewrite any references to imported names to
        // use the fully-qualified identifier. Likewise, any time we visit an imported item, we
        // rewrite its name to be fully-qualified,
        for evaluator in module.evaluators.values_mut() {
            self.visit_mut_evaluator_function(evaluator)?;
        }

        if let Some(boundary_constraints) = module.boundary_constraints.as_mut() {
            if !boundary_constraints.is_empty() {
                self.visit_mut_boundary_constraints(boundary_constraints)?;
            }
        }

        if let Some(integrity_constraints) = module.integrity_constraints.as_mut() {
            if !integrity_constraints.is_empty() {
                self.visit_mut_integrity_constraints(integrity_constraints)?;
            }
        }

        self.current_module = None;

        // We're done
        if self.has_type_errors || self.has_undefined_variables {
            ControlFlow::Break(ModuleError::Invalid)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_mut_evaluator_function(
        &mut self,
        function: &mut EvaluatorFunction,
    ) -> ControlFlow<ModuleError> {
        // Only allow integrity constraints in this context
        self.allowed_constraints = AllowedConstraintsType::Integrity;
        // Start a new lexical scope
        let prev = self.locals.clone();
        // Track referenced imports in a new context, as we want to update the dependency graph
        // for this function using only those imports referenced from this function body
        let referenced = mem::take(&mut self.referenced);

        // Add the set of parameters to the current scope, check for conflicts
        for trace_segment in function.params.iter_mut() {
            for trace_binding in trace_segment.bindings.iter() {
                if let Some((prev, prev_binding)) = self.globals.get_key_value(&trace_binding.name)
                {
                    // Warn if we shadow other global declarations
                    self.diagnostics
                        .diagnostic(Severity::Warning)
                        .with_message(format!(
                            "trace binding shadows {} declaration in root module",
                            prev_binding
                        ))
                        .with_primary_label(
                            trace_binding.name.span(),
                            "this name shadows a previous declaration",
                        )
                        .with_secondary_label(prev.span(), "previously declared here")
                        .emit();
                }
                let namespaced_name = NamespacedIdentifier::Binding(trace_binding.name);
                if let Some((prev, prev_binding)) = self.locals.get_key_value(&namespaced_name) {
                    self.diagnostics
                        .diagnostic(Severity::Warning)
                        .with_message(format!(
                            "trace binding would shadow {} declaration",
                            prev_binding
                        ))
                        .with_primary_label(
                            trace_binding.name.span(),
                            "this name shadows a previous declaration",
                        )
                        .with_secondary_label(prev.span(), "previously declared here")
                        .emit();
                }
                self.locals.insert(
                    namespaced_name,
                    BindingType::TraceParam(TraceBinding {
                        span: trace_binding.span,
                        name: Some(name),
                        segment: trace_segment.id,
                        offset: trace_binding.offset,
                        size: trace_binding.size,
                        ty: trace_binding.ty,
                    }),
                );
            }
        }

        // Visit all of the statements in the body
        self.visit_mut_statement_block(&mut function.body)?;

        // Update the dependency graph for this function
        let current_item = QualifiedIdentifier::new(
            self.current_module.unwrap(),
            NamespacedIdentifier::Function(function.name),
        );
        for (referenced_item, ref_type) in self.referenced.iter() {
            let referenced_item = self.deps.add_node(*referenced_item);
            self.deps.add_edge(current_item, referenced_item, *ref_type);
        }

        // Restore the original references metadata
        self.referenced = referenced;
        // Restore the original lexical scope
        self.locals = prev;
        // Disallow constraints
        self.allowed_constraints = AllowedConstraintsType::None;

        ControlFlow::Continue(())
    }

    fn visit_mut_boundary_constraints(
        &mut self,
        body: &mut Vec<Statement>,
    ) -> ControlFlow<ModuleError> {
        // Only allow boundary constraints in this context
        self.allowed_constraints = AllowedConstraintsType::Boundary;
        // Save the current bindings set, as we're entering a new lexical scope
        let prev = self.locals.clone();
        // Visit all of the statements, check variable usage, and track referenced imports
        self.visit_mut_statement_block(body)?;
        // Restore the original lexical scope
        self.locals = prev;
        // Disallow any constraints
        self.allowed_constraints = AllowedConstraintsType::None;

        ControlFlow::Continue(())
    }

    fn visit_mut_integrity_constraints(
        &mut self,
        body: &mut Vec<Statement>,
    ) -> ControlFlow<ModuleError> {
        // Only allow integrity constraints in this context
        self.allowed_constraints = AllowedConstraintsType::Integrity;
        // Save the current bindings set, as we're entering a new lexical scope
        let prev = self.locals.clone();
        // Visit all of the statements, check variable usage, and track referenced imports
        self.visit_mut_statement_block(body)?;
        // Restore the original lexical scope
        self.locals = prev;
        // Disallow any constraints
        self.allowed_constraints = AllowedConstraintsType::None;

        ControlFlow::Continue(())
    }

    // Visit scalar constraints and ensure that they are valid semantically, and have correct types
    fn visit_mut_enforce(&mut self, expr: &mut ScalarExpr) -> ControlFlow<ModuleError> {
        // Verify that constraints are permitted here
        match self.allowed_constraints {
            AllowedConstraintsType::None => {
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("invalid constraint")
                    .with_primary_label(
                        expr.span(),
                        "constraints are not permitted in this context",
                    )
                    .emit();
                ControlFlow::Break(ModuleError::Invalid)
            }
            AllowedConstraintsType::Boundary => {
                // Only equality expressions are permitted in boundary constraints
                match expr {
                    ScalarExpr::Binary(ref mut expr) if expr.op == BinaryOp::Eq => {
                        // Ensure that the left-hand expression is a boundary access
                        match expr.lhs.as_mut() {
                            ScalarExpr::BoundedSymbolAccess(ref mut access) => {
                                // Visit the expression operands
                                self.visit_mut_symbol_access(&mut access.column)?;

                                // Ensure the referenced symbol was a trace column, and that it produces a scalar value
                                let found = match self.resolvable_binding_type(&access.column.name)
                                {
                                    Ok(ty) => match &ty.item {
                                        BindingType::TraceColumn(tr)
                                        | BindingType::TraceParam(tr) => {
                                            if tr.size == 1 {
                                                ty
                                            } else {
                                                self.diagnostics.diagnostic(Severity::Error)
                                                        .with_message("invalid constraint")
                                                        .with_primary_label(access.span(), "boundary constraints must reference a single trace column")
                                                        .with_secondary_label(ty.span(), format!("but this value is a set of {} columns", tr.size))
                                                        .emit();
                                                return ControlFlow::Break(ModuleError::Invalid);
                                            }
                                        }
                                        _ => {
                                            self.diagnostics.diagnostic(Severity::Error)
                                                    .with_message("invalid constraint")
                                                    .with_primary_label(access.span(), "boundary constraints must reference a single trace column")
                                                    .with_secondary_label(ty.span(), format!("but this value is a {}", &ty))
                                                    .emit();
                                            return ControlFlow::Break(ModuleError::Invalid);
                                        }
                                    },
                                    Err(_) => {
                                        // We've already raised a diagnostic for the undefined variable
                                        return ControlFlow::Break(ModuleError::Invalid);
                                    }
                                };
                                // Validate that the symbol access produces a scalar value
                                //
                                // If no type is known, a diagnostic is already emitted, so proceed as if it is valid
                                if let Some(ty) = access.column.ty.as_ref() {
                                    if !ty.is_scalar() {
                                        // Invalid constraint, only scalar values are allowed
                                        self.diagnostics.diagnostic(Severity::Error)
                                            .with_message("invalid constraint")
                                            .with_primary_label(access.span(), format!("expected a scalar value here, but this expression has type '{}'", &ty))
                                            .with_secondary_label(found.span(), "the type is inferred from this declaration")
                                            .emit();
                                        return ControlFlow::Break(ModuleError::Invalid);
                                    }
                                }

                                // Verify that the right-hand expression evaluates to a scalar
                                //
                                // The only way this is not the case, is if it is a a symbol access which produces an aggregate
                                self.visit_mut_scalar_expr(expr.rhs.as_mut())?;
                                if let ScalarExpr::SymbolAccess(access) = expr.rhs.as_ref() {
                                    // Ensure this access produces a scalar, or if the type is unknown, assume it is valid
                                    // because a diagnostic will have already been emitted
                                    if !access.ty.as_ref().map(|t| t.is_scalar()).unwrap_or(true) {
                                        self.diagnostics.diagnostic(Severity::Error)
                                            .with_message("invalid constraint")
                                            .with_primary_label(access.span(), format!("expected a scalar value here, but this expression has type '{}'", access.ty.as_ref().unwrap()))
                                            .emit();
                                        return ControlFlow::Break(ModuleError::Invalid);
                                    }
                                }

                                ControlFlow::Continue(())
                            }
                            other => {
                                self.diagnostics.diagnostic(Severity::Error)
                                    .with_message("invalid boundary constraint")
                                    .with_primary_label(other.span(), "expected this to be a reference to a trace column boundary, e.g. `a.first`")
                                    .with_note("The given constraint is not a boundary constraint, and only boundary constraints are valid here.")
                                    .emit();
                                ControlFlow::Break(ModuleError::Invalid)
                            }
                        }
                    }
                    ScalarExpr::Call(ref expr) => {
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid boundary constraint")
                            .with_primary_label(expr.span(), "expected an equality expression here")
                            .with_note("Calls to evaluator functions are only permitted in integrity constraints")
                            .emit();
                        ControlFlow::Break(ModuleError::Invalid)
                    }
                    expr => {
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid boundary constraint")
                            .with_primary_label(expr.span(), "expected an equality expression here")
                            .with_note("Boundary constraints must be expressed as an equality, e.g. `a.first = 0`")
                            .emit();
                        ControlFlow::Break(ModuleError::Invalid)
                    }
                }
            }
            AllowedConstraintsType::Integrity => {
                // If a boundary access is encountered at any point, an error will be raised, so
                // we do not need to validate that the constraint has no boundary references.
                //
                // However, we do need to validate two things:
                //
                // 1. That the constraint produces a scalar value
                // 2. That the expression is either an equality, or a call to an evaluator function
                //
                match expr {
                    ScalarExpr::Binary(ref mut expr) if expr.op == BinaryOp::Eq => {
                        self.visit_mut_binary_expr(expr)
                    }
                    ScalarExpr::Call(ref mut expr) => {
                        // Visit the call normally, so we can resolve the callee identifier
                        self.visit_mut_call(expr)?;

                        // Check that the call references an evaluator
                        //
                        // If unresolved, we've already raised a diagnostic for the invalid call
                        match expr.callee {
                            ResolvableIdentifier::Resolved(callee) => {
                                match callee.id() {
                                    id @ NamespacedIdentifier::Function(_) => {
                                        match self.locals.get_key_value(&id) {
                                            // Binding is to a local evaluator
                                            Some((_, BindingType::Function(FunctionType::Evaluator(_)))) => ControlFlow::Continue(()),
                                            // Binding is to a local non-evaluator function
                                            Some((local_name, _)) => {
                                                self.diagnostics.diagnostic(Severity::Error)
                                                    .with_message("invalid integrity constraint")
                                                    .with_primary_label(id.span(), "calls in constraints must be to evaluator functions")
                                                    .with_secondary_label(local_name.span(), "this function is not an evaluator")
                                                    .emit();
                                                ControlFlow::Break(ModuleError::Invalid)
                                            }
                                            None => {
                                                // If the call was resolved, it must be to an imported function,
                                                // and we will have already validated the reference
                                                let (import_id, module_id) = self.imported.get_key_value(&id).unwrap();
                                                let module = self.library.get(module_id).unwrap();
                                                if module.evaluators.get(&id.id()).is_none() {
                                                    self.diagnostics.diagnostic(Severity::Error)
                                                        .with_message("invalid integrity constraint")
                                                        .with_primary_label(id.span(), "calls in constraints must be to evaluator functions")
                                                        .with_secondary_label(import_id.span(), "the function imported here is not an evaluator")
                                                        .emit();
                                                    return ControlFlow::Break(ModuleError::Invalid);
                                                }
                                                ControlFlow::Continue(())
                                            }
                                        }
                                    }
                                    // We take care to only allow constructing Call with a function identifier, but it
                                    // is possible for someone to unintentionally set the callee to a binding identifer, which is
                                    // a compiler internal error, hence the panic
                                    id => panic!("invalid callee identifier, expected function id, got binding: {:#?}", id),
                                }
                            }
                            ResolvableIdentifier::Local(id) => {
                                self.diagnostics.diagnostic(Severity::Error)
                                    .with_message("invalid call target")
                                    .with_primary_label(id.span(), "local variables are not callable")
                                    .with_note("A local binding by this name is in scope, but no such function is declared in this module. Are you missing an import?")
                                    .emit();
                                ControlFlow::Break(ModuleError::Invalid)
                            }
                            ResolvableIdentifier::Global(id) => {
                                self.diagnostics.diagnostic(Severity::Error)
                                    .with_message("invalid call target")
                                    .with_primary_label(id.span(), "global declarations are not callable")
                                    .with_note("A global declaration with this name is in scope, but no such function is declared in this module. Are you missing an import?")
                                    .emit();
                                ControlFlow::Break(ModuleError::Invalid)
                            }
                            ResolvableIdentifier::Unresolved(_) => ControlFlow::Continue(()),
                        }
                    }
                    expr => {
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid integrity constraint")
                            .with_primary_label(expr.span(), "expected either an equality expression, or a call to an evaluator here")
                            .with_note("Integrity constraints must be expressed as an equality, e.g. `a = 0`, or a call, e.g. `evaluator(a)`")
                            .emit();
                        ControlFlow::Break(ModuleError::Invalid)
                    }
                }
            }
        }
    }

    /// Comprehension constraints are very similar to those handled by `visit_mut_enforce`, except that they occur in
    /// the body of a list comprehension. The comprehension itself is validated normally, but the body of the comprehension
    /// must be checked using `visit_mut_enforce`, rather than `visit_mut_scalar_expr`. We do this by setting a flag in the
    /// state that is checked in `visit_mut_list_comprehension` to enable checks that are specific to constraints.
    fn visit_mut_enforce_all(&mut self, expr: &mut ListComprehension) -> ControlFlow<ModuleError> {
        self.in_constraint_comprehension = true;
        let result = self.visit_mut_list_comprehension(expr);
        self.in_constraint_comprehension = false;

        result
    }

    fn visit_mut_let(&mut self, expr: &mut Let) -> ControlFlow<ModuleError> {
        // Visit the binding expression first
        self.visit_mut_expr(&mut expr.value)?;

        // Start new lexical scope for the body
        let prev = self.locals.clone();

        // Check if the new binding shadows a previous local declaration
        let namespaced_name = NamespacedIdentifier::Binding(expr.name);
        if let Some((prev, prev_binding)) = self.locals.get_key_value(&namespaced_name) {
            self.diagnostics
                .diagnostic(Severity::Warning)
                .with_message(format!(
                    "let-bound variable shadows previous {} declaration",
                    prev_binding
                ))
                .with_primary_label(
                    expr.name.span(),
                    "this binding shadows a previous declaration",
                )
                .with_secondary_label(prev.span(), "previously declared here")
                .emit();
        } else {
            let binding_ty = self.expr_binding_type(&expr.value).unwrap();
            self.locals
                .insert(NamespacedIdentifier::Binding(expr.name), binding_ty);
        }

        // Visit the let body
        self.visit_mut_statement_block(&mut expr.body)?;

        // Restore the original lexical scope
        self.locals = prev;

        ControlFlow::Continue(())
    }

    fn visit_mut_list_comprehension(
        &mut self,
        expr: &mut ListComprehension,
    ) -> ControlFlow<ModuleError> {
        // Visit the iterables first, and resolve their identifiers
        for iterable in expr.iterables.iter_mut() {
            self.visit_mut_expr(iterable)?;
        }

        // Start a new lexical scope
        let prev = self.locals.clone();

        // Track the result type of this comprehension expression
        let mut result_ty = None;
        // Add all of the bindings to the local scope, warn on shadowing, error on conflicting bindings
        let mut bound = HashSet::<Identifier>::default();
        // Track the successfully typed check bindings for validation
        let mut binding_tys: Vec<(Identifier, SourceSpan, Option<BindingType>)> = vec![];
        for (i, binding) in expr.bindings.iter().copied().enumerate() {
            if let Some(prev) = bound.get(&binding) {
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("invalid binding in list comprehension")
                    .with_primary_label(
                        binding.span(),
                        "this name is already bound in this comprehension",
                    )
                    .with_secondary_label(prev.span(), "previously bound here")
                    .emit();
                return ControlFlow::Break(ModuleError::NameConflict(binding.span()));
            }

            if let Some((prev, prev_binding)) = self
                .locals
                .get_key_value(&NamespacedIdentifier::Binding(binding))
            {
                self.diagnostics
                    .diagnostic(Severity::Warning)
                    .with_message(format!(
                        "comprehension binding shadows previous {} declaration",
                        prev_binding,
                    ))
                    .with_primary_label(
                        binding.span(),
                        "this binding shadows a previous declaration",
                    )
                    .with_secondary_label(prev.span(), "previously declared here")
                    .emit();
            }

            bound.insert(binding);

            let iterable = &expr.iterables[i];
            let iterable_ty = iterable.ty().unwrap();
            if let Some(expected_ty) = result_ty.replace(iterable_ty) {
                if expected_ty != iterable_ty {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("invalid list comprehension")
                        .with_primary_label(
                            iterable.span(),
                            format!(
                                "expected an iterable of type {} here, got {}",
                                &expected_ty, &iterable_ty
                            ),
                        )
                        .with_secondary_label(
                            expr.iterables[0].span(),
                            "expected type was derived here",
                        )
                        .emit();
                }
            }
            match self.expr_binding_type(iterable) {
                Ok(iterable_binding_ty) => {
                    binding_tys.push((binding, iterable.span(), Some(iterable_binding_ty)));
                }
                Err(InvalidAccessError::InvalidBinding) => {
                    // We tried to call an evaluator function
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("invalid iterable")
                        .with_primary_label(
                            iterable.span(),
                            "this expression is not a valid iterable",
                        )
                        .emit();
                    return ControlFlow::Break(ModuleError::Invalid);
                }
                Err(_) => {
                    // The iterable type is undefined/unresolvable
                    //
                    // In order to proceed with semantic analysis without erroring
                    // out too early, we track each binding and its type as we traverse
                    // the binding/iterable pairs. If a type is unknown, we try to fill
                    // in its spot with a fabricated type which matches the other iterables,
                    // but if no other types are available, we select a large vector size
                    // as a default type, which allows type checking to proceed temporarily
                    //
                    // For now, we record `None` until all iterables have been visited
                    binding_tys.push((binding, iterable.span(), None));
                }
            }
        }

        // If we were unable to determine a type for any of the bindings, use a large vector as a placeholder
        let expected = BindingType::Local(result_ty.unwrap_or(Type::Vector(u32::MAX as usize)));

        // Bind everything now, resolving any deferred types using our fallback expected type
        for (binding, _, binding_ty) in binding_tys.drain(..) {
            self.locals.insert(
                NamespacedIdentifier::Binding(binding),
                binding_ty.unwrap_or(expected.clone()),
            );
        }

        // Visit the selector
        if let Some(selector) = expr.selector.as_mut() {
            self.visit_mut_scalar_expr(selector)?;
        }

        // Visit the comprehension body
        if self.in_constraint_comprehension {
            self.visit_mut_enforce(expr.body.as_mut())?;
        } else {
            self.visit_mut_scalar_expr(expr.body.as_mut())?;
        }

        // Store the result type of this comprehension
        expr.ty = result_ty;

        // Restore the original lexical scope
        self.locals = prev;

        ControlFlow::Continue(())
    }

    fn visit_mut_call(&mut self, expr: &mut Call) -> ControlFlow<ModuleError> {
        // Ensure the callee exists, and resolve the type if possible
        self.visit_mut_resolvable_identifier(&mut expr.callee)?;

        // Validate the callee type
        let callee_binding_ty = self.resolvable_binding_type(&expr.callee);
        match callee_binding_ty {
            Ok(ref binding_ty) => {
                let derived_from = binding_ty.span();
                if let BindingType::Function(ref fty) = binding_ty.item {
                    // There must be an evaluator by this name
                    let qid = expr.callee.resolved().unwrap();
                    let dependency_type = match fty {
                        FunctionType::Evaluator(_) => DependencyType::Evaluator,
                        _ => DependencyType::Function,
                    };
                    let prev = self.referenced.insert(qid, dependency_type);
                    if prev.is_some() {
                        assert_eq!(prev, Some(dependency_type));
                    }
                    // TODO: When we have non-evaluator functions, we must fetch the type in its signature here,
                    // and store it as the type of the Call expression
                } else {
                    self.has_type_errors = true;
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("invalid callee")
                        .with_primary_label(expr.callee.span(), "expected a function name")
                        .with_secondary_label(
                            derived_from,
                            "instead a reference to this declaration was given",
                        )
                        .emit();
                    return ControlFlow::Break(ModuleError::Invalid);
                }
            }
            Err(_) => {
                // We've already raised a diagnostic for this when visiting the access expression
                assert!(self.has_undefined_variables || self.has_type_errors);
            }
        }

        // Visit the call arguments
        for expr in expr.args.iter_mut() {
            self.visit_mut_expr(expr)?;
        }

        // Validate arguments for builtin functions, which currently consist only of the sum/prod reducers
        if expr.is_builtin() {
            match expr.callee.as_ref().name() {
                // The known reducers - each takes a single argument, which must be an aggregate or comprehension
                symbols::Sum | symbols::Prod => {
                    match expr.args.as_slice() {
                        [arg] => {
                            match self.expr_binding_type(arg) {
                                Ok(binding_ty) => {
                                    if !binding_ty.ty().map(|t| t.is_aggregate()).unwrap_or(false) {
                                        self.has_type_errors = true;
                                        self.diagnostics.diagnostic(Severity::Error)
                                            .with_message("invalid call")
                                            .with_primary_label(expr.span(), "this function expects an argument of aggregate type")
                                            .with_secondary_label(arg.span(), format!("but this argument is a {}", &binding_ty))
                                            .emit();
                                    }
                                }
                                Err(_) => {
                                    // We've already raised a diagnostic for this when visiting the access expression
                                    assert!(self.has_undefined_variables || self.has_type_errors);
                                }
                            }
                        }
                        _ => {
                            self.has_type_errors = true;
                            self.diagnostics
                                .diagnostic(Severity::Error)
                                .with_message("invalid call")
                                .with_primary_label(
                                    expr.span(),
                                    format!(
                                        "the callee expects a single argument, but got {}",
                                        expr.args.len()
                                    ),
                                )
                                .emit();
                        }
                    }
                }
                other => unimplemented!("unrecognized builtin function: {}", other),
            }
            return ControlFlow::Continue(());
        }

        // Validate arguments for evaluator functions:
        //
        // * Must be trace bindings or aliases of same
        // * Must match the type signature of the callee
        if let Ok(Span {
            item: BindingType::Function(FunctionType::Evaluator(ref params)),
            ..
        }) = callee_binding_ty
        {
            for (arg, param) in expr.args.iter().zip(params.iter()) {
                // We're expecting either a vector of bindings, or an aggregate binding
                match arg {
                    Expr::SymbolAccess(ref access) => {
                        match self.access_binding_type(access) {
                            Ok(BindingType::TraceColumn(tr) | BindingType::TraceParam(tr)) => {
                                if tr.size == param.size {
                                    // Success, the argument and parameter types match up, but
                                    // we must make sure the segments also match
                                    let same_segment = tr.segment == param.id;
                                    if !same_segment {
                                        let expected_segment = segment_id_to_name(param.id);
                                        let segment_name = segment_id_to_name(tr.segment);
                                        self.has_type_errors = true;
                                        self.diagnostics
                                            .diagnostic(Severity::Error)
                                            .with_message("invalid evaluator function argument")
                                            .with_primary_label(
                                                arg.span(),
                                                format!(
                                                    "callee expects columns from the {} trace",
                                                    expected_segment
                                                ),
                                            )
                                            .with_secondary_label(
                                                tr.span,
                                                format!(
                                                    "but this column is from the {} trace",
                                                    segment_name
                                                ),
                                            )
                                            .emit();
                                    }
                                } else {
                                    self.has_type_errors = true;
                                    self.diagnostics.diagnostic(Severity::Error)
                                                .with_message("invalid call")
                                                .with_primary_label(expr.span(), "type mismatch in function argument")
                                                .with_secondary_label(arg.span(), format!("callee expects {} trace columns here, but this binding provides {}", param.size, tr.size))
                                                .emit();
                                }
                            }
                            Ok(BindingType::Vector(ref elems)) => {
                                let mut size = 0;
                                for elem in elems.iter() {
                                    match elem {
                                        BindingType::TraceColumn(tr)
                                        | BindingType::TraceParam(tr) => {
                                            if tr.segment == param.id {
                                                size += tr.size;
                                            } else {
                                                let expected_segment = segment_id_to_name(param.id);
                                                let segment_name = segment_id_to_name(tr.segment);
                                                self.has_type_errors = true;
                                                self.diagnostics
                                                    .diagnostic(Severity::Error)
                                                    .with_message("invalid evaluator function argument")
                                                    .with_primary_label(
                                                        arg.span(),
                                                        format!(
                                                            "callee expects columns from the {} trace",
                                                            expected_segment
                                                        ),
                                                    )
                                                    .with_secondary_label(
                                                        tr.span,
                                                        format!(
                                                            "but this column is from the {} trace",
                                                            segment_name
                                                        ),
                                                    )
                                                    .emit();
                                                return ControlFlow::Continue(());
                                            }
                                        }
                                        invalid => {
                                            self.has_type_errors = true;
                                            self.diagnostics
                                                .diagnostic(Severity::Error)
                                                .with_message("invalid call")
                                                .with_primary_label(
                                                    expr.span(),
                                                    "type mismatch in function argument",
                                                )
                                                .with_secondary_label(
                                                    access.span(),
                                                    format!(
                                                        "expected trace column(s), got {}",
                                                        &invalid
                                                    ),
                                                )
                                                .emit();
                                            return ControlFlow::Continue(());
                                        }
                                    }
                                }

                                if size != param.size {
                                    self.has_type_errors = true;
                                    self.diagnostics.diagnostic(Severity::Error)
                                                .with_message("invalid call")
                                                .with_primary_label(expr.span(), "type mismatch in function argument")
                                                .with_secondary_label(arg.span(), format!("callee expects {} trace columns here, but this binding provides {}", param.size, size))
                                                .emit();
                                }
                            }
                            Ok(binding_ty) => {
                                self.has_type_errors = true;
                                self.diagnostics.diagnostic(Severity::Error)
                                            .with_message("invalid call")
                                            .with_primary_label(expr.span(), "invalid argument for evaluator function")
                                            .with_secondary_label(arg.span(), format!("expected a trace binding, or vector of trace bindings here, but got a {}", &binding_ty))
                                            .emit();
                            }
                            Err(_) => {
                                // We've already raised a diagnostic for this when visiting the access expression
                                assert!(self.has_undefined_variables || self.has_type_errors);
                            }
                        }
                    }
                    Expr::Vector(ref elems) => {
                        // We need to make sure that the number of columns represented by the vector corresponds to those
                        // expected by the callee, which requires us to first check each element of the vector, and then
                        // at the end determine if the sizes line up
                        let mut size = 0;
                        for elem in elems.iter() {
                            match self.scalar_expr_binding_type(elem) {
                                Ok(BindingType::TraceColumn(tr) | BindingType::TraceParam(tr)) => {
                                    if tr.segment == param.id {
                                        size += tr.size;
                                    } else {
                                        let expected_segment = segment_id_to_name(param.id);
                                        let segment_name = segment_id_to_name(tr.segment);
                                        self.has_type_errors = true;
                                        self.diagnostics
                                            .diagnostic(Severity::Error)
                                            .with_message("invalid evaluator function argument")
                                            .with_primary_label(
                                                arg.span(),
                                                format!(
                                                    "callee expects columns from the {} trace",
                                                    expected_segment
                                                ),
                                            )
                                            .with_secondary_label(
                                                elem.span(),
                                                format!(
                                                    "but this column is from the {} trace",
                                                    segment_name
                                                ),
                                            )
                                            .emit();
                                        return ControlFlow::Continue(());
                                    }
                                }
                                Ok(invalid) => {
                                    self.has_type_errors = true;
                                    self.diagnostics
                                        .diagnostic(Severity::Error)
                                        .with_message("invalid call")
                                        .with_primary_label(
                                            arg.span(),
                                            "invalid argument for evaluator function",
                                        )
                                        .with_secondary_label(
                                            elem.span(),
                                            format!(
                                                "expected a trace binding here, but got a {}",
                                                &invalid
                                            ),
                                        )
                                        .emit();
                                }
                                Err(_) => {
                                    // We've already raised a diagnostic for this when visiting the access expression
                                    assert!(self.has_undefined_variables || self.has_type_errors);
                                }
                            }
                        }
                        if size != param.size {
                            self.has_type_errors = true;
                            self.diagnostics.diagnostic(Severity::Error)
                                .with_message("invalid call")
                                .with_primary_label(expr.span(), "type mismatch in function argument")
                                .with_secondary_label(arg.span(), format!("callee expects {} trace columns here, but this argument only provides {}", param.size, size))
                                .emit();
                        }
                    }
                    wrong => {
                        self.has_type_errors = true;
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid call")
                            .with_primary_label(expr.span(), "invalid argument for evaluator function")
                            .with_secondary_label(arg.span(), format!("expected a trace binding, or vector of trace bindings here, but got a {}", wrong))
                            .emit();
                    }
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_mut_bounded_symbol_access(
        &mut self,
        expr: &mut BoundedSymbolAccess,
    ) -> ControlFlow<ModuleError> {
        // Any access to a bounded symbol is to be considered invalid, because the only places
        // in which they are valid are explicitly checked in the handling of `visit_mut_enforce`
        // Visit the underlying access first
        self.diagnostics
            .diagnostic(Severity::Error)
            .with_message("invalid expression")
            .with_primary_label(
                expr.span(),
                "references to column boundaries are not permitted here",
            )
            .emit();
        ControlFlow::Break(ModuleError::Invalid)
    }

    fn visit_mut_symbol_access(&mut self, expr: &mut SymbolAccess) -> ControlFlow<ModuleError> {
        self.visit_mut_resolvable_identifier(&mut expr.name)?;

        let resolved_binding_ty = match self.resolvable_binding_type(&expr.name) {
            Ok(ty) => ty,
            // An unresolved identifier at this point means that it is undefined,
            // but we've already raised a diagnostic
            //
            // There is nothing useful we can do here other than continue traversing the module
            // gathering as many undefined variable usages as possible before bailing
            Err(_) => return ControlFlow::Continue(()),
        };

        if let ResolvableIdentifier::Resolved(qid) = &expr.name {
            // Determine the type of dependency this represents in the dependency graph
            let dep_type = match &resolved_binding_ty.item {
                BindingType::Constant(_) => Some(DependencyType::Constant),
                BindingType::PeriodicColumn(_) => Some(DependencyType::PeriodicColumn),
                BindingType::Function(_) => {
                    panic!("unexpected function binding in symbol access context")
                }
                _ => None,
            };

            // Update the dependency graph
            if let Some(dep_type) = dep_type {
                // If the item is already in the referenced set, it should have the same type
                let prev = self.referenced.insert(*qid, dep_type);
                if prev.is_some() {
                    assert_eq!(prev, Some(dep_type));
                }
            }
        }

        // The symbol is resolved, check to see if the access is valid
        let derived_from = resolved_binding_ty.span();
        let resolved_binding_ty = resolved_binding_ty.item;
        match resolved_binding_ty.access(expr.access_type.clone()) {
            Ok(binding_ty) => {
                assert_eq!(expr.ty.replace(binding_ty.ty().unwrap()), None);
                ControlFlow::Continue(())
            }
            Err(err) => {
                self.has_type_errors = true;
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("invalid variable access")
                    .with_primary_label(expr.span(), err.to_string())
                    .with_secondary_label(derived_from, "references this declaration")
                    .emit();
                // Continue with a fabricated type
                let ty = match &expr.access_type {
                    AccessType::Slice(ref range) => Type::Vector(range.end - range.start),
                    _ => Type::Felt,
                };
                assert_eq!(expr.ty.replace(ty), None);
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_mut_resolvable_identifier(
        &mut self,
        expr: &mut ResolvableIdentifier,
    ) -> ControlFlow<ModuleError> {
        let current_module = self.current_module.unwrap();
        match expr {
            // If already resolved, and referencing a local variable, there is nothing to do
            ResolvableIdentifier::Local(_) => ControlFlow::Continue(()),
            // If already resolved, and referencing a global declaration, there is nothing to do
            ResolvableIdentifier::Global(_) => ControlFlow::Continue(()),
            // If already resolved, and not referencing the current module or the root module, add it to the referenced set
            ResolvableIdentifier::Resolved(id) => {
                // Ignore references to functions in the builtin module
                if id.is_builtin() {
                    return ControlFlow::Continue(());
                }

                ControlFlow::Continue(())
            }
            ResolvableIdentifier::Unresolved(namespaced_id) => {
                // If locally defined, resolve it to the current module
                let namespaced_id = *namespaced_id;
                if let Some(binding_ty) = self.locals.get(&namespaced_id) {
                    match binding_ty {
                        // This identifier is a local variable, alias to a declaration, or a function parameter
                        BindingType::Alias(_)
                        | BindingType::Local(_)
                        | BindingType::Vector(_)
                        | BindingType::PublicInput(_)
                        | BindingType::TraceColumn(_)
                        | BindingType::TraceParam(_) => {
                            *expr = ResolvableIdentifier::Local(namespaced_id.id());
                        }
                        // These binding types are module-local declarations
                        BindingType::Constant(_)
                        | BindingType::Function(_)
                        | BindingType::PeriodicColumn(_) => {
                            *expr = ResolvableIdentifier::Resolved(QualifiedIdentifier::new(
                                current_module,
                                namespaced_id,
                            ));
                        }
                        // Locals never hold these binding types, which represent global declarations,
                        // they use Alias instead
                        BindingType::RandomValue(_) => unreachable!(),
                    }
                    return ControlFlow::Continue(());
                }

                // If globally defined, convert to a globally resolved identifier
                let id = namespaced_id.id();
                if self.globals.contains_key(&id) {
                    *expr = ResolvableIdentifier::Global(id);
                    return ControlFlow::Continue(());
                }

                // If imported, resolve it to the imported module, and add it to the referenced set
                if let Some((imported_id, imported_from)) =
                    self.imported.get_key_value(&namespaced_id)
                {
                    let qualified_id = QualifiedIdentifier::new(*imported_from, *imported_id);
                    *expr = ResolvableIdentifier::Resolved(qualified_id);

                    return ControlFlow::Continue(());
                }

                // If we reach here, we were unable to resolve this identifier, raise a diagnostic
                self.has_undefined_variables = true;
                match namespaced_id {
                    NamespacedIdentifier::Function(_) => {
                        self.diagnostics
                            .diagnostic(Severity::Error)
                            .with_message("reference to undefined function")
                            .with_primary_label(
                                namespaced_id.span(),
                                "no function by this name is declared in scope",
                            )
                            .emit();
                    }
                    NamespacedIdentifier::Binding(_) => {
                        self.diagnostics
                            .diagnostic(Severity::Error)
                            .with_message("reference to undefined variable")
                            .with_primary_label(
                                namespaced_id.span(),
                                "this variable is not defined",
                            )
                            .emit();
                    }
                }

                ControlFlow::Continue(())
            }
        }
    }
}

impl<'a> SemanticAnalysis<'a> {
    fn expr_binding_type(&self, expr: &Expr) -> Result<BindingType, InvalidAccessError> {
        match expr {
            Expr::Const(constant) => Ok(BindingType::Local(constant.ty())),
            Expr::Range(range) => Ok(BindingType::Local(Type::Vector(range.end - range.start))),
            Expr::Vector(ref elems) => {
                let mut binding_tys = Vec::with_capacity(elems.len());
                for elem in elems.iter() {
                    binding_tys.push(self.scalar_expr_binding_type(elem)?);
                }

                Ok(BindingType::Vector(binding_tys))
            }
            Expr::Matrix(expr) => {
                let rows = expr.len();
                let columns = expr[0].len();
                Ok(BindingType::Local(Type::Matrix(rows, columns)))
            }
            Expr::SymbolAccess(ref expr) => self.access_binding_type(expr),
            Expr::Call(Call { ty: None, .. }) => Err(InvalidAccessError::InvalidBinding),
            Expr::Call(Call { ty: Some(ty), .. }) => Ok(BindingType::Local(*ty)),
            Expr::Binary(_) => Ok(BindingType::Local(Type::Felt)),
            Expr::ListComprehension(ref lc) => {
                match lc.ty {
                    Some(ty) => Ok(BindingType::Local(ty)),
                    None => {
                        // The types of all iterables must be the same, so the type of
                        // the comprehension is given by the type of the iterables. We
                        // just pick the first iterable to tell us the type
                        self.expr_binding_type(&lc.iterables[0])
                    }
                }
            }
        }
    }

    fn scalar_expr_binding_type(
        &self,
        expr: &ScalarExpr,
    ) -> Result<BindingType, InvalidAccessError> {
        match expr {
            ScalarExpr::SymbolAccess(ref expr) => self.access_binding_type(expr),
            ScalarExpr::Call(Call { ty: None, .. }) => Err(InvalidAccessError::InvalidBinding),
            ScalarExpr::Call(Call { ty: Some(ty), .. }) => Ok(BindingType::Local(*ty)),
            ScalarExpr::Const(_) | ScalarExpr::Binary(_) | ScalarExpr::BoundedSymbolAccess(_) => {
                Ok(BindingType::Local(Type::Felt))
            }
        }
    }

    fn access_binding_type(&self, expr: &SymbolAccess) -> Result<BindingType, InvalidAccessError> {
        let binding_ty = self.resolvable_binding_type(&expr.name)?;
        binding_ty.access(expr.access_type.clone())
    }

    fn resolvable_binding_type(
        &self,
        id: &ResolvableIdentifier,
    ) -> Result<Span<BindingType>, InvalidAccessError> {
        match id {
            ResolvableIdentifier::Local(id) => {
                // The item is a let-bound variable or function parameter
                let namespaced_id = NamespacedIdentifier::Binding(*id);
                self.locals
                    .get_key_value(&namespaced_id)
                    .map(|(nid, ty)| Span::new(nid.span(), ty.clone()))
                    .ok_or(InvalidAccessError::UndefinedVariable)
            }
            ResolvableIdentifier::Global(id) => {
                // The item is a declaration in the root module
                self.globals
                    .get_key_value(id)
                    .map(|(nid, ty)| Span::new(nid.span(), ty.clone()))
                    .ok_or(InvalidAccessError::UndefinedVariable)
            }
            ResolvableIdentifier::Resolved(ref qid) => self.resolved_binding_type(qid),
            ResolvableIdentifier::Unresolved(_) => Err(InvalidAccessError::UndefinedVariable),
        }
    }

    fn resolved_binding_type(
        &self,
        qid: &QualifiedIdentifier,
    ) -> Result<Span<BindingType>, InvalidAccessError> {
        if qid.module == self.program.name {
            // This is the root module, so the value will be in either locals or globals
            self.locals
                .get_key_value(&qid.item)
                .map(|(k, v)| Span::new(k.span(), v.clone()))
                .or_else(|| {
                    self.globals
                        .get_key_value(qid.as_ref())
                        .map(|(k, v)| Span::new(k.span(), v.clone()))
                })
                .ok_or(InvalidAccessError::UndefinedVariable)
        } else if qid.module == self.current_module.unwrap() {
            // This is a reference to a module-local declaration
            self.locals
                .get_key_value(&qid.item)
                .map(|(k, v)| Span::new(k.span(), v.clone()))
                .ok_or(InvalidAccessError::UndefinedVariable)
        } else {
            // This is an imported item, and it must exist or we would have failed during
            // import resolution. It must also be a constant, as function identifiers are
            // not valid in a binding context
            let imported_from = self.library.get(&qid.module).unwrap();
            imported_from
                .constants
                .get(qid.as_ref())
                .map(|c| Span::new(c.span(), BindingType::Constant(c.ty())))
                .or_else(|| {
                    imported_from.evaluators.get(qid.as_ref()).map(|e| {
                        Span::new(
                            e.span(),
                            BindingType::Function(FunctionType::Evaluator(e.params.clone())),
                        )
                    })
                })
                .ok_or(InvalidAccessError::UndefinedVariable)
        }
    }
}

fn segment_id_to_name(id: TraceSegmentId) -> Symbol {
    match id {
        0 => symbols::Main,
        1 => symbols::Aux,
        _ => unimplemented!(),
    }
}
