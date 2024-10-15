use air_parser::{ast, LexicalScope};
use air_pass::Pass;

use miden_diagnostics::{DiagnosticsHandler, SourceSpan, Spanned};

use crate::{graph::NodeIndex, ir::*, CompileError};

pub struct AstToMir<'a> {
    diagnostics: &'a DiagnosticsHandler,
}
impl<'a> AstToMir<'a> {
    /// Create a new instance of this pass
    #[inline]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}
impl<'p> Pass for AstToMir<'p> {
    type Input<'a> = ast::Program;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, program: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut mir = Mir::new(program.name);

        //TODO MIR: Implement AST > MIR lowering
        // 1. Start from the previous lowering from AST to AIR
        // 2. Understand what changes when starting from an unoptimized AST
        // (with no constant prop and no inlining)
        // 3. Implement the needed changes

        let random_values = program.random_values;
        let trace_columns = program.trace_columns;
        let boundary_constraints = program.boundary_constraints;
        let integrity_constraints = program.integrity_constraints;

        let mut builder = MirBuilder {
            diagnostics: self.diagnostics,
            mir: &mut mir,
            random_values,
            trace_columns,
            bindings: Default::default(),
        };

        // Insert placeholders nodes for future Operation::Definition (needed for function bodies to call other functions)
        for (ident, _func) in program.functions.iter() {
            let node_index = builder.insert_typed_constant(None, ast::ConstantExpr::Scalar(0));
            builder
                .mir
                .constraint_graph_mut()
                .functions
                .insert(*ident, node_index);
        }

        for (ident, func) in program.functions.iter() {
            builder.insert_function_body(ident, func)?;
        }

        for bc in boundary_constraints.iter() {
            builder.build_boundary_constraint(bc)?;
        }

        for ic in integrity_constraints.iter() {
            builder.build_integrity_constraint(ic)?;
        }

        Ok(mir)
    }
}

struct MirBuilder<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
    mir: &'a mut Mir,
    random_values: Option<ast::RandomValues>,
    trace_columns: Vec<ast::TraceSegment>,
    bindings: LexicalScope<Identifier, NodeIndex>,
}
impl<'a> MirBuilder<'a> {
    fn insert_variable(&mut self, span: SourceSpan, ty: ast::Type, index: usize) -> NodeIndex {
        let mir_type = match ty {
            ast::Type::Felt => MirType::Felt,
            ast::Type::Vector(n) => MirType::Vector(n),
            ast::Type::Matrix(n, m) => MirType::Matrix(n, m),
        };

        self.insert_op(Operation::Variable(SpannedVariable::new(
            span, mir_type, index,
        )))
    }

    fn insert_function_body(
        &mut self,
        ident: &QualifiedIdentifier,
        func: &ast::Function,
    ) -> Result<(), CompileError> {
        let body = &func.body;
        let params = &func.params;

        let mut params_node_indices = Vec::with_capacity(params.len());
        for (index, (ident, ty)) in params.iter().enumerate() {
            let node_index = self.insert_variable(ident.span(), *ty, index);
            params_node_indices.push(node_index);
        }

        let return_variable_node_index = self.insert_variable(ident.span(), func.return_type, 0);

        // Get the number of nodes before representing the body
        let before_node_count = self.mir.constraints.graph().num_nodes();

        // Insert the function body
        for stmt in body.iter() {
            self.build_function_body_statement(stmt)?;
        }

        let after_node_count = self.mir.constraints.graph().num_nodes();

        let range = before_node_count..after_node_count;

        // Reference all the new nodes created by the body in the definition
        let node_index_to_update = *self.mir.constraint_graph().functions.get(ident).unwrap();
        let operation_definition = Operation::Definition(
            params_node_indices,
            return_variable_node_index,
            range.map(|i| NodeIndex::default() + i).collect(),
        );

        self.mir
            .constraint_graph_mut()
            .update_node(&node_index_to_update, operation_definition);

        Ok(())
    }

    // TODO: Handle other types of statements
    fn build_boundary_constraint(&mut self, bc: &ast::Statement) -> Result<(), CompileError> {
        self.build_statement(bc)
    }

    fn build_integrity_constraint(&mut self, ic: &ast::Statement) -> Result<(), CompileError> {
        self.build_statement(ic)
    }

    fn build_function_body_statement(&mut self, s: &ast::Statement) -> Result<(), CompileError> {
        self.build_statement(s)
    }

    fn build_statement(&mut self, c: &ast::Statement) -> Result<(), CompileError> {
        match c {
            // If we have a let, update scoping and insertuate the body
            ast::Statement::Let(expr) => {
                self.build_let(expr, |bldr, stmt| bldr.build_statement(stmt))
            }
            // Depending on the expression, we can have different types of operations in the
            // If we have a symbol access, we have to get it depending on the scope and add the
            // identifier to the graph nodes (SSA)
            ast::Statement::Expr(expr) => {
                self.insert_expr(expr)?;
                Ok(())
            }
            // Enforce statements can be translated to Enf operations in the MIR on scalar expressions
            ast::Statement::Enforce(scalar_expr) => {
                let scalar_expr = self.insert_scalar_expr(scalar_expr)?;
                self.insert_op(Operation::Enf(scalar_expr));

                Ok(())
            }
            ast::Statement::EnforceIf(_, _) => unreachable!(), // This variant was only available after AST's inlining, we should handle EnforceAll instead
            ast::Statement::EnforceAll(_list_comprehension) => {
                //self.build_statement(&ast::Statement::Expr(ScalarExpr(list_comprehension.body))?;

                // let scalar_expr = self.insert_scalar_expr(scalar_expr)?;
                // let insert_op = self.insert_op(Operation::For(scalar_expr));

                Ok(())
            }
        }
    }

    fn build_let<F>(
        &mut self,
        expr: &ast::Let,
        mut statement_builder: F,
    ) -> Result<(), CompileError>
    where
        F: FnMut(&mut MirBuilder, &ast::Statement) -> Result<(), CompileError>,
    {
        let bound = self.insert_expr(&expr.value)?;
        self.bindings.enter();
        self.bindings.insert(expr.name, bound);
        for stmt in expr.body.iter() {
            statement_builder(self, stmt)?;
        }
        self.bindings.exit();
        Ok(())
    }

    fn insert_expr(&mut self, expr: &ast::Expr) -> Result<NodeIndex, CompileError> {
        match expr {
            ast::Expr::Const(span) => {
                let value = self.insert_typed_constant(Some(span.span()), span.item.clone());
                Ok(value)
            }
            ast::Expr::Range(_range_expr) => todo!(),
            ast::Expr::Vector(_span) => todo!(),
            ast::Expr::Matrix(_span) => todo!(),
            ast::Expr::SymbolAccess(_symbol_access) => {
                // Should resolve the identifier depending on the scope, and add the access to the graph once it's resolved
                todo!()
            }
            ast::Expr::Binary(binary_expr) => self.insert_binary_expr(binary_expr),
            ast::Expr::Call(call) => {

                // Insert the call args: as 1 node or N nodes?
                let args_node_index: Vec<_> = call
                    .args
                    .iter()
                    .map(|arg| self.insert_expr(arg).unwrap())
                    .collect();

                // Get the known callee in the functions hashmap
                // First, resolve the callee, panic if it's not resolved
                let resolved_callee = call.callee.resolved().unwrap();
                // Then, get the node index of the function definition
                let callee_node_index = *self
                    .mir
                    .constraint_graph()
                    .functions
                    .get(&resolved_callee)
                    .unwrap();

                let call_node_index =
                    self.insert_op(Operation::Call(callee_node_index, args_node_index));

                Ok(call_node_index)
            }
            ast::Expr::ListComprehension(_list_comprehension) => {
                /*/// 1. Add all the bindings of the list_comprehension
                // 2. Constuct the body of the list comprehension
                // 3. Add a "For" node to represent the list comprehension

                let b = list_comprehension.bindings;
                let bindings_node_index = self.insert_scalar_expr(&list_comprehension.body)?;

                for binding in list_comprehension.bindings.iter() {
                    let binding_node_index = // insert vae
                }

                for iterator in list_comprehension.iterables.iter() {
                    let iterator_node_index = self.insert_expr(iterator)?;
                }
                let iterator_node_index = self.insert_expr(&list_comprehension.iterables)?;
                let selector_node_index = if let Some(selector) = &list_comprehension.selector {
                    Some(self.insert_scalar_expr(selector)?)
                } else {
                    None
                };
                let body_node_index = self.insert_scalar_expr(&list_comprehension.body)?;

                let for_node_index = self.insert_op(Operation::For(bindings_node_index, body_node_index, selector_node_index));

                Ok(for_node_index)*/

                todo!()
            }
            ast::Expr::Let(expr) => self.expand_let_expr(expr),
        }
    }

    fn expand_let_expr(&mut self, expr: &ast::Let) -> Result<NodeIndex, CompileError> {
        let mut next_let = Some(expr);
        let snapshot = self.bindings.clone();
        loop {
            let let_expr = next_let.take().expect("invalid empty let body");
            let bound = self.insert_expr(&let_expr.value)?;
            self.bindings.enter();
            self.bindings.insert(let_expr.name, bound);
            match let_expr.body.last().unwrap() {
                ast::Statement::Let(ref inner_let) => {
                    next_let = Some(inner_let);
                }
                ast::Statement::Expr(ref expr) => {
                    let value = self.insert_expr(expr);
                    self.bindings = snapshot;
                    break value;
                }
                ast::Statement::Enforce(_)
                | ast::Statement::EnforceIf(_, _)
                | ast::Statement::EnforceAll(_) => {
                    unreachable!()
                }
            }
        }
    }

    // TODO: Merge with other insert_expr
    /*fn insert_expr(&mut self, expr: &ast::Expr) -> Result<NodeIndex, CompileError> {
        match expr {
            ast::Expr::Const(ref constant) => match &constant.item {
                ast::ConstantExpr::Scalar(value) => {
                    let value = self.insert_constant(Some(constant.span()), *value);
                    Ok(MemoizedBinding::Scalar(value))
                }
                ast::ConstantExpr::Vector(values) => {
                    let values = self.insert_constants(Some(constant.span()), values.as_slice());
                    Ok(MemoizedBinding::Vector(values))
                }
                ast::ConstantExpr::Matrix(values) => {
                    let values = values
                        .iter()
                        .map(|vs| self.insert_constants(Some(constant.span()), vs.as_slice()))
                        .collect();
                    Ok(MemoizedBinding::Matrix(values))
                }
            },
            ast::Expr::Range(ref values) => {
                let values = values
                    .to_slice_range()
                    .map(|v| self.insert_constant(Some(values.span()), v as u64))
                    .collect();
                Ok(MemoizedBinding::Vector(values))
            }
            ast::Expr::Vector(ref values) => match values[0].ty().unwrap() {
                ast::Type::Felt => {
                    let mut nodes = vec![];
                    for value in values.iter().cloned() {
                        let value = value.try_into().unwrap();
                        nodes.push(self.insert_scalar_expr(&value)?);
                    }
                    Ok(MemoizedBinding::Vector(nodes))
                }
                ast::Type::Vector(n) => {
                    let mut nodes = vec![];
                    for row in values.iter().cloned() {
                        match row {
                            ast::Expr::Const(const_expr) => {
                                self.insert_typed_constant(Some(const_expr.span()), const_expr.item);
                            }
                            // Rework based on Continuous Symbol Access in the MIR ?
                            ast::Expr::SymbolAccess(access) => {
                                let mut cols = vec![];
                                for i in 0..n {
                                    let access = ast::ScalarExpr::SymbolAccess(
                                        access.access(AccessType::Index(i)).unwrap(),
                                    );
                                    let node = self.insert_scalar_expr(&access)?;
                                    cols.push(node);
                                }
                                nodes.push(cols);
                            }
                            ast::Expr::Vector(ref elems) => {
                                let mut cols = vec![];
                                for elem in elems.iter().cloned() {
                                    let elem: ast::ScalarExpr = elem.try_into().unwrap();
                                    let node = self.insert_scalar_expr(&elem)?;
                                    cols.push(node);
                                }
                                nodes.push(cols);
                            }
                            _ => unreachable!(),
                        }
                    }
                    Ok(MemoizedBinding::Matrix(nodes))
                }
                _ => unreachable!(),
            },
            ast::Expr::Matrix(ref values) => {
                let mut rows = Vec::with_capacity(values.len());
                for vs in values.iter() {
                    let mut cols = Vec::with_capacity(vs.len());
                    for value in vs {
                        cols.push(self.insert_scalar_expr(value)?);
                    }
                    rows.push(cols);
                }
                Ok(MemoizedBinding::Matrix(rows))
            }
            ast::Expr::Binary(ref bexpr) => {
                let value = self.insert_binary_expr(bexpr)?;
                Ok(MemoizedBinding::Scalar(value))
            }
            ast::Expr::SymbolAccess(ref access) => {
                match self.bindings.get(access.name.as_ref()) {
                    None => {
                        // Must be a reference to a declaration
                        let value = self.insert_symbol_access(access);
                        Ok(MemoizedBinding::Scalar(value))
                    }
                    Some(MemoizedBinding::Scalar(node)) => {
                        assert_eq!(access.access_type, AccessType::Default);
                        Ok(MemoizedBinding::Scalar(*node))
                    }
                    Some(MemoizedBinding::Vector(nodes)) => {
                        let value = match &access.access_type {
                            AccessType::Default => MemoizedBinding::Vector(nodes.clone()),
                            AccessType::Index(idx) => MemoizedBinding::Scalar(nodes[*idx]),
                            AccessType::Slice(range) => {
                                MemoizedBinding::Vector(nodes[range.to_slice_range()].to_vec())
                            }
                            AccessType::Matrix(_, _) => unreachable!(),
                        };
                        Ok(value)
                    }
                    Some(MemoizedBinding::Matrix(nodes)) => {
                        let value = match &access.access_type {
                            AccessType::Default => MemoizedBinding::Matrix(nodes.clone()),
                            AccessType::Index(idx) => MemoizedBinding::Vector(nodes[*idx].clone()),
                            AccessType::Slice(range) => {
                                MemoizedBinding::Matrix(nodes[range.to_slice_range()].to_vec())
                            }
                            AccessType::Matrix(row, col) => {
                                MemoizedBinding::Scalar(nodes[*row][*col])
                            }
                        };
                        Ok(value)
                    }
                }
            }
            ast::Expr::Let(ref let_expr) => self.insert_let_expr(let_expr),
            ast::Expr::Call(call) => {

            }
            // These node types should not exist at this point
            ast::Expr::Call(_) | ast::Expr::ListComprehension(_) => unreachable!(),
        }
    }*/

    fn insert_scalar_expr(&mut self, expr: &ast::ScalarExpr) -> Result<NodeIndex, CompileError> {
        match expr {
            ast::ScalarExpr::Const(value) => {
                Ok(self.insert_op(Operation::Value(SpannedMirValue {
                    span: value.span(),
                    value: MirValue::Constant(ConstantValue::Felt(value.item)),
                })))
            }
            ast::ScalarExpr::SymbolAccess(access) => Ok(self.insert_symbol_access(access)),
            ast::ScalarExpr::Binary(expr) => self.insert_binary_expr(expr),
            ast::ScalarExpr::Let(ref let_expr) => {
                let index = self.expand_let_expr(let_expr)?;

                // TODO: Check that the resulting expr is a scalar expr
                Ok(index)
            }
            ast::ScalarExpr::Call(call) => {
                // 1. Recup le NodeIndex correspondant a la Definition de func
                // 2. Représenter z (l'argument) via son NodeIndex
                // 3. Décrire le Call avec Operation::Call(NodeIndex, Vec<NodeIndex>)

                let args_node_index: Vec<_> = call
                    .args
                    .iter()
                    .map(|arg| self.insert_expr(arg).unwrap())
                    .collect();

                // Get the known callee in the functions hashmap
                // First, resolve the callee, panic if it's not resolved
                let resolved_callee = call.callee.resolved().unwrap();
                // Then, get the node index of the function definition
                let callee_node_index = *self
                    .mir
                    .constraint_graph()
                    .functions
                    .get(&resolved_callee)
                    .unwrap();

                let call_node_index =
                    self.insert_op(Operation::Call(callee_node_index, args_node_index));

                match self.mir.constraint_graph().node(&callee_node_index).op() {
                    Operation::Definition(_, return_node_index, _) => {
                        match self.mir.constraint_graph().node(return_node_index).op() {
                            Operation::Variable(var) => {
                                assert_eq!(
                                    var.ty,
                                    MirType::Felt,
                                    "Call to a function that does not return a scalar value"
                                );
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }

                Ok(call_node_index)
            }
            ast::ScalarExpr::BoundedSymbolAccess(_) => unreachable!(),
        }
    }

    // Use square and multiply algorithm to expand the exp into a series of multiplications
    fn expand_exp(&mut self, lhs: NodeIndex, rhs: u64, span: SourceSpan) -> NodeIndex {
        match rhs {
            0 => self.insert_typed_constant(Some(span), ast::ConstantExpr::Scalar(1)),
            1 => lhs,
            n if n % 2 == 0 => {
                let square = self.insert_op(Operation::Mul(lhs, lhs));
                self.expand_exp(square, n / 2, span)
            }
            n => {
                let square = self.insert_op(Operation::Mul(lhs, lhs));
                let rec = self.expand_exp(square, (n - 1) / 2, span);
                self.insert_op(Operation::Mul(lhs, rec))
            }
        }
    }

    fn insert_binary_expr(&mut self, expr: &ast::BinaryExpr) -> Result<NodeIndex, CompileError> {
        if expr.op == ast::BinaryOp::Exp {
            let lhs = self.insert_scalar_expr(expr.lhs.as_ref())?;
            let ast::ScalarExpr::Const(rhs) = expr.rhs.as_ref() else {
                unreachable!();
            };
            return Ok(self.expand_exp(lhs, rhs.item, expr.span()));
        }

        let lhs = self.insert_scalar_expr(expr.lhs.as_ref())?;
        let rhs = self.insert_scalar_expr(expr.rhs.as_ref())?;
        Ok(match expr.op {
            ast::BinaryOp::Add => self.insert_op(Operation::Add(lhs, rhs)),
            ast::BinaryOp::Sub => self.insert_op(Operation::Sub(lhs, rhs)),
            ast::BinaryOp::Mul => self.insert_op(Operation::Mul(lhs, rhs)),
            _ => unreachable!(),
        })
    }

    // Assumed inlining was done, to update
    fn insert_symbol_access(&mut self, access: &ast::SymbolAccess) -> NodeIndex {
        use air_parser::ast::ResolvableIdentifier;
        match access.name {
            // At this point during compilation, fully-qualified identifiers can only possibly refer
            // to a periodic column, as all functions have been inlined, and constants propagated.
            ResolvableIdentifier::Resolved(ref qid) => {
                if let Some(pc) = self.mir.periodic_columns.get(qid) {
                    self.insert_op(Operation::Value(SpannedMirValue {
                        span: qid.span(),
                        value: MirValue::PeriodicColumn(PeriodicColumnAccess::new(
                            *qid,
                            pc.period(),
                        )),
                    }))
                } else {
                    // This is a qualified reference that should have been eliminated
                    // during inlining or constant propagation, but somehow slipped through.
                    unreachable!(
                        "expected reference to periodic column, got `{:?}` instead",
                        qid
                    );
                }
            }
            // This must be one of public inputs, random values, or trace columns
            ResolvableIdentifier::Global(id) | ResolvableIdentifier::Local(id) => {
                // Special identifiers are those which are `$`-prefixed, and must refer to
                // the random values array (generally the case), or the names of trace segments (e.g. `$main`)
                if id.is_special() {
                    if let Some(rv) = self.random_value_access(access) {
                        return self.insert_op(Operation::Value(SpannedMirValue {
                            span: id.span(),
                            value: MirValue::RandomValue(rv),
                        }));
                    }

                    // Must be a trace segment name
                    if let Some(ta) = self.trace_access(access) {
                        return self.insert_op(Operation::Value(SpannedMirValue {
                            span: id.span(),
                            value: MirValue::TraceAccess(ta),
                        }));
                    }

                    // It should never be possible to reach this point - semantic analysis
                    // would have caught that this identifier is undefined.
                    unreachable!(
                        "expected reference to random values array or trace segment: {:#?}",
                        access
                    );
                }

                // Otherwise, we check the trace bindings, random value bindings, and public inputs, in that order
                if let Some(trace_access) = self.trace_access(access) {
                    return self.insert_op(Operation::Value(SpannedMirValue {
                        span: id.span(),
                        value: MirValue::TraceAccess(trace_access),
                    }));
                }

                if let Some(random_value) = self.random_value_access(access) {
                    return self.insert_op(Operation::Value(SpannedMirValue {
                        span: id.span(),
                        value: MirValue::RandomValue(random_value),
                    }));
                }

                if let Some(public_input) = self.public_input_access(access) {
                    return self.insert_op(Operation::Value(SpannedMirValue {
                        span: id.span(),
                        value: MirValue::PublicInput(public_input),
                    }));
                }

                // If we reach here, this must be a let-bound variable
                return *self
                    .bindings
                    .get(access.name.as_ref())
                    .expect("undefined variable");
                /*{
                    MemoizedBinding::Scalar(node) => {
                        assert_eq!(access.access_type, AccessType::Default);
                        *node
                    }
                    MemoizedBinding::Vector(nodes) => {
                        if let AccessType::Index(idx) = &access.access_type {
                            return nodes[*idx];
                        }
                        unreachable!("impossible vector access: {:?}", access)
                    }
                    MemoizedBinding::Matrix(nodes) => {
                        if let AccessType::Matrix(row, col) = &access.access_type {
                            return nodes[*row][*col];
                        }
                        unreachable!("impossible matrix access: {:?}", access)
                    }
                }*/
            }
            // These should have been eliminated by previous compiler passes
            ResolvableIdentifier::Unresolved(_) => {
                unreachable!(
                    "expected fully-qualified or global reference, got `{:?}` instead",
                    &access.name
                );
            }
        }
    }

    // Check assumptions, probably this assumed that the inlining pass did some work
    fn random_value_access(&self, access: &ast::SymbolAccess) -> Option<usize> {
        let rv = self.random_values.as_ref()?;
        let id = access.name.as_ref();
        if rv.name == id {
            if let AccessType::Index(index) = access.access_type {
                assert!(index < rv.size);
                return Some(index);
            } else {
                // This should have been caught earlier during compilation
                unreachable!("invalid access to random values array: {:#?}", access);
            }
        }

        // This must be a reference to a binding, if it is a random value access
        let binding = rv.bindings.iter().find(|rb| rb.name == id)?;

        match access.access_type {
            AccessType::Default if binding.size == 1 => Some(binding.offset),
            AccessType::Index(extra) if binding.size > 1 => Some(binding.offset + extra),
            // This should have been caught earlier during compilation
            _ => unreachable!(
                "unexpected random value access type encountered during lowering: {:#?}",
                access
            ),
        }
    }

    // Check assumptions, probably this assumed that the inlining pass did some work
    fn public_input_access(&self, access: &ast::SymbolAccess) -> Option<PublicInputAccess> {
        let public_input = self.mir.public_inputs.get(access.name.as_ref())?;
        if let AccessType::Index(index) = access.access_type {
            Some(PublicInputAccess::new(public_input.name, index))
        } else {
            // This should have been caught earlier during compilation
            unreachable!(
                "unexpected public input access type encountered during lowering: {:#?}",
                access
            )
        }
    }

    // Check assumptions, probably this assumed that the inlining pass did some work
    fn trace_access(&self, access: &ast::SymbolAccess) -> Option<TraceAccess> {
        let id = access.name.as_ref();
        for (i, segment) in self.trace_columns.iter().enumerate() {
            if segment.name == id {
                if let AccessType::Index(column) = access.access_type {
                    return Some(TraceAccess::new(i, column, access.offset));
                } else {
                    // This should have been caught earlier during compilation
                    unreachable!(
                        "unexpected trace access type encountered during lowering: {:#?}",
                        &access
                    );
                }
            }

            if let Some(binding) = segment
                .bindings
                .iter()
                .find(|tb| tb.name.as_ref() == Some(id))
            {
                return match access.access_type {
                    AccessType::Default if binding.size == 1 => Some(TraceAccess::new(
                        binding.segment,
                        binding.offset,
                        access.offset,
                    )),
                    AccessType::Index(extra_offset) if binding.size > 1 => Some(TraceAccess::new(
                        binding.segment,
                        binding.offset + extra_offset,
                        access.offset,
                    )),
                    // This should have been caught earlier during compilation
                    _ => unreachable!(
                        "unexpected trace access type encountered during lowering: {:#?}",
                        access
                    ),
                };
            }
        }

        None
    }

    /// Adds the specified operation to the graph and returns the index of its node.
    #[inline]
    fn insert_op(&mut self, op: Operation) -> NodeIndex {
        self.mir.constraint_graph_mut().insert_node(op)
    }

    fn insert_typed_constant(
        &mut self,
        span: Option<SourceSpan>,
        value: ast::ConstantExpr,
    ) -> NodeIndex {
        let mir_value = match value {
            ast::ConstantExpr::Scalar(val) => ConstantValue::Felt(val),
            ast::ConstantExpr::Vector(val) => ConstantValue::Vector(val),
            ast::ConstantExpr::Matrix(val) => ConstantValue::Matrix(val),
        };
        self.insert_op(Operation::Value(SpannedMirValue {
            span: span.unwrap_or_default(),
            value: MirValue::Constant(mir_value),
        }))
    }
}
