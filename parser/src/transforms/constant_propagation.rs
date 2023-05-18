use std::{collections::HashMap, ops::ControlFlow};

use miden_diagnostics::{SourceSpan, Span, Spanned};

use crate::ast::{visit::VisitMut, *};

#[derive(Debug, thiserror::Error)]
pub enum InvalidConstantError {
    #[error("this value is too large for an exponent")]
    InvalidExponent(SourceSpan),
}

/// This pass performs constant propagation on a [Program], replacing all uses of a constant
/// with the constant itself, converting accesses into constant aggregates with the accessed
/// value, replacing local variables bound to constants with the constant value, and folding
/// constant expressions into constant values.
///
/// It is expected that the provided [Program] has already been run through semantic analysis,
/// so it will panic if it encounters invalid constructions to help catch bugs in the semantic
/// analysis pass, should they exist.
#[derive(Default)]
pub struct ConstantPropagator {
    global: HashMap<QualifiedIdentifier, Span<ConstantExpr>>,
    local: HashMap<Identifier, Span<ConstantExpr>>,
    in_constraint_comprehension: bool,
}
impl ConstantPropagator {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(mut self, program: &mut Program) -> Result<(), InvalidConstantError> {
        self.global.reserve(program.constants.len());

        match self.run_visitor(program) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(err) => Err(err),
        }
    }

    fn run_visitor(&mut self, program: &mut Program) -> ControlFlow<InvalidConstantError> {
        // Record all of the constant declarations
        for (name, constant) in program.constants.iter() {
            assert_eq!(
                self.global
                    .insert(*name, Span::new(constant.span(), constant.value.clone())),
                None
            );
        }

        // Visit all of the evaluators
        for evaluator in program.evaluators.values_mut() {
            self.visit_mut_evaluator_function(evaluator)?;
        }

        // Visit all of the constraints
        self.visit_mut_boundary_constraints(&mut program.boundary_constraints)?;
        self.visit_mut_integrity_constraints(&mut program.integrity_constraints)
    }

    fn try_fold_binary_expr(
        &mut self,
        expr: &mut BinaryExpr,
    ) -> Result<Option<u64>, InvalidConstantError> {
        // Visit operands first to ensure they are reduced to constants if possible
        if let ControlFlow::Break(err) = self.visit_mut_scalar_expr(expr.lhs.as_mut()) {
            return Err(err);
        }
        if let ControlFlow::Break(err) = self.visit_mut_scalar_expr(expr.rhs.as_mut()) {
            return Err(err);
        }
        // If both operands are constant, fold
        if let (ScalarExpr::Const(l), ScalarExpr::Const(r)) = (expr.lhs.as_mut(), expr.rhs.as_mut())
        {
            let folded = match expr.op {
                BinaryOp::Add => l.item + r.item,
                BinaryOp::Sub => l.item - r.item,
                BinaryOp::Mul => l.item * r.item,
                BinaryOp::Exp => match r.item.try_into() {
                    Ok(exp) => l.item.pow(exp),
                    Err(_) => return Err(InvalidConstantError::InvalidExponent(expr.span())),
                },
                // This op cannot be folded
                BinaryOp::Eq => return Ok(None),
            };
            Ok(Some(folded))
        } else {
            Ok(None)
        }
    }
}
impl VisitMut<InvalidConstantError> for ConstantPropagator {
    /// Fold constant expressions
    fn visit_mut_scalar_expr(
        &mut self,
        expr: &mut ScalarExpr,
    ) -> ControlFlow<InvalidConstantError> {
        match expr {
            // Expression is already folded
            ScalarExpr::Const(_) => ControlFlow::Continue(()),
            // Need to check if this access is to a constant value, and transform to a constant if so
            ScalarExpr::SymbolAccess(sym) => {
                let constant_value = match sym.name {
                    // Possibly a reference to a constant declaration
                    ResolvableIdentifier::Resolved(ref qid) => {
                        self.global.get(qid).cloned().map(|s| (s.span(), s.item))
                    }
                    // Possibly a reference to a local bound to a constant
                    ResolvableIdentifier::Local(ref id) => {
                        self.local.get(id).cloned().map(|s| (s.span(), s.item))
                    }
                    // Other identifiers cannot possibly be constant
                    _ => None,
                };
                if let Some((span, constant_expr)) = constant_value {
                    match constant_expr {
                        ConstantExpr::Scalar(value) => {
                            assert_eq!(sym.access_type, AccessType::Default);
                            *expr = ScalarExpr::Const(Span::new(span, value));
                        }
                        ConstantExpr::Vector(value) => match sym.access_type {
                            AccessType::Index(idx) => {
                                *expr = ScalarExpr::Const(Span::new(span, value[idx]));
                            }
                            ref ty => panic!(
                                "invalid constant reference, expected scalar access, got {:?}",
                                ty
                            ),
                        },
                        ConstantExpr::Matrix(value) => match sym.access_type {
                            AccessType::Matrix(row, col) => {
                                *expr = ScalarExpr::Const(Span::new(span, value[row][col]));
                            }
                            ref ty => panic!(
                                "invalid constant reference, expected scalar access, got {:?}",
                                ty
                            ),
                        },
                    }
                }
                ControlFlow::Continue(())
            }
            // Fold constant expressions
            ScalarExpr::Binary(ref mut binary_expr) => {
                let span = binary_expr.span();
                match self.try_fold_binary_expr(binary_expr) {
                    Ok(maybe_folded) => {
                        if let Some(folded) = maybe_folded {
                            *expr = ScalarExpr::Const(Span::new(span, folded));
                        }
                        ControlFlow::Continue(())
                    }
                    Err(err) => ControlFlow::Break(err),
                }
            }
            // While calls cannot be constant folded, arguments can be
            ScalarExpr::Call(ref mut call) => self.visit_mut_call(call),
            // This cannot be constant folded
            ScalarExpr::BoundedSymbolAccess(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) -> ControlFlow<InvalidConstantError> {
        let span = expr.span();
        match expr {
            // Already constant
            Expr::Const(_) => ControlFlow::Continue(()),
            // Lift to `Expr::Const` if the scalar expression is constant
            //
            // We deal with symbol accesses directly, as they may evaluate to an aggregate constant
            Expr::SymbolAccess(ref mut access) => {
                let constant_value = match access.name {
                    // Possibly a reference to a constant declaration
                    ResolvableIdentifier::Resolved(ref qid) => {
                        self.global.get(qid).cloned().map(|s| (s.span(), s.item))
                    }
                    // Possibly a reference to a local bound to a constant
                    ResolvableIdentifier::Local(ref id) => {
                        self.local.get(id).cloned().map(|s| (s.span(), s.item))
                    }
                    // Other identifiers cannot possibly be constant
                    _ => None,
                };
                if let Some((span, constant_expr)) = constant_value {
                    match constant_expr {
                        cexpr @ ConstantExpr::Scalar(_) => {
                            assert_eq!(access.access_type, AccessType::Default);
                            *expr = Expr::Const(Span::new(span, cexpr));
                        }
                        ConstantExpr::Vector(value) => match access.access_type.clone() {
                            AccessType::Default => {
                                *expr = Expr::Const(Span::new(span, ConstantExpr::Vector(value)));
                            }
                            AccessType::Slice(range) => {
                                let vector = value[range.start..range.end].to_vec();
                                *expr = Expr::Const(Span::new(span, ConstantExpr::Vector(vector)));
                            }
                            AccessType::Index(idx) => {
                                *expr =
                                    Expr::Const(Span::new(span, ConstantExpr::Scalar(value[idx])));
                            }
                            ref ty => panic!(
                                "invalid constant reference, expected scalar access, got {:?}",
                                ty
                            ),
                        },
                        ConstantExpr::Matrix(value) => match access.access_type.clone() {
                            AccessType::Default => {
                                *expr = Expr::Const(Span::new(span, ConstantExpr::Matrix(value)));
                            }
                            AccessType::Slice(range) => {
                                let matrix = value[range.start..range.end].to_vec();
                                *expr = Expr::Const(Span::new(span, ConstantExpr::Matrix(matrix)));
                            }
                            AccessType::Index(idx) => {
                                *expr = Expr::Const(Span::new(
                                    span,
                                    ConstantExpr::Vector(value[idx].clone()),
                                ));
                            }
                            AccessType::Matrix(row, col) => {
                                *expr = Expr::Const(Span::new(
                                    span,
                                    ConstantExpr::Scalar(value[row][col]),
                                ));
                            }
                        },
                    }
                }
                ControlFlow::Continue(())
            }
            Expr::Call(ref mut call) => self.visit_mut_call(call),
            Expr::Binary(ref mut binary_expr) => {
                let span = binary_expr.span();
                match self.try_fold_binary_expr(binary_expr) {
                    Ok(maybe_folded) => {
                        if let Some(folded) = maybe_folded {
                            *expr = Expr::Const(Span::new(span, ConstantExpr::Scalar(folded)));
                        }
                        ControlFlow::Continue(())
                    }
                    Err(err) => ControlFlow::Break(err),
                }
            }
            // Ranges are constant
            Expr::Range(_) => ControlFlow::Continue(()),
            // Visit vector elements, and promote the vector to `Expr::Const` if possible
            Expr::Vector(ref mut vector) => {
                let mut is_constant = true;
                for elem in vector.iter_mut() {
                    self.visit_mut_scalar_expr(elem)?;
                    is_constant &= elem.is_constant();
                }
                if is_constant {
                    let vector = ConstantExpr::Vector(
                        vector
                            .iter()
                            .map(|sexpr| match sexpr {
                                ScalarExpr::Const(elem) => elem.item,
                                _ => unreachable!(),
                            })
                            .collect(),
                    );
                    *expr = Expr::Const(Span::new(span, vector));
                }
                ControlFlow::Continue(())
            }
            // Visit matrix elements, and promote the matrix to `Expr::Const` if possible
            Expr::Matrix(ref mut matrix) => {
                let mut is_constant = true;
                for row in matrix.iter_mut() {
                    for column in row.iter_mut() {
                        self.visit_mut_scalar_expr(column)?;
                        is_constant &= column.is_constant();
                    }
                }
                if is_constant {
                    let matrix = ConstantExpr::Matrix(
                        matrix
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(|col| match col {
                                        ScalarExpr::Const(elem) => elem.item,
                                        _ => unreachable!(),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect(),
                    );
                    *expr = Expr::Const(Span::new(span, matrix));
                }
                ControlFlow::Continue(())
            }
            // Visit list comprehensions and convert to constant if possible
            Expr::ListComprehension(ref mut lc) => {
                let mut has_constant_iterables = true;
                for iterable in lc.iterables.iter_mut() {
                    self.visit_mut_expr(iterable)?;
                    has_constant_iterables &= iterable.is_constant();
                }

                // If we have constant iterables, drive the comprehension, evaluating it at
                // each step. If any part of the body cannot be compile-time evaluated, then
                // we bail early, as the comprehension can only be folded if all parts of it
                // are constant.
                if !has_constant_iterables {
                    return ControlFlow::Continue(());
                }

                // Start a new lexical scope
                let prev = self.local.clone();

                // All iterables must be the same length, so determine the number of
                // steps based on the length of the first iterable
                let max_len = match &lc.iterables[0] {
                    Expr::Const(Span {
                        item: ConstantExpr::Vector(elems),
                        ..
                    }) => elems.len(),
                    Expr::Const(Span {
                        item: ConstantExpr::Matrix(rows),
                        ..
                    }) => rows.len(),
                    Expr::Const(_) => panic!("expected iterable constant, got scalar"),
                    Expr::Range(range) => range.end - range.start,
                    _ => unreachable!(),
                };

                // Drive the comprehension step-by-step
                let mut folded = vec![];
                for step in 0..max_len {
                    for (binding, iterable) in lc.bindings.iter().copied().zip(lc.iterables.iter())
                    {
                        let span = iterable.span();
                        match iterable {
                            Expr::Const(Span {
                                item: ConstantExpr::Vector(elems),
                                ..
                            }) => {
                                let value = ConstantExpr::Scalar(elems[step]);
                                self.local.insert(binding, Span::new(span, value));
                            }
                            Expr::Const(Span {
                                item: ConstantExpr::Matrix(elems),
                                ..
                            }) => {
                                let value = ConstantExpr::Vector(elems[step].clone());
                                self.local.insert(binding, Span::new(span, value));
                            }
                            Expr::Range(range) => {
                                assert!(range.end > range.start + step);
                                let value = ConstantExpr::Scalar((range.start + step) as u64);
                                self.local.insert(binding, Span::new(span, value));
                            }
                            _ => unreachable!(),
                        }
                    }

                    if let Some(mut selector) = lc.selector.as_ref().cloned() {
                        self.visit_mut_scalar_expr(&mut selector)?;
                        match selector {
                            ScalarExpr::Const(selected) => {
                                // If the selector returns false on this iteration, go to the next step
                                if *selected == 0 {
                                    continue;
                                }
                            }
                            // The selector cannot be evaluated, bail out early
                            _ => return ControlFlow::Continue(()),
                        }
                    }

                    let mut body = lc.body.as_ref().clone();
                    self.visit_mut_scalar_expr(&mut body)?;

                    // If the body is constant, store the result in the vector, otherwise we must
                    // bail because this comprehension cannot be folded
                    if let ScalarExpr::Const(folded_body) = body {
                        folded.push(folded_body.item);
                    } else {
                        return ControlFlow::Continue(());
                    }
                }

                // Exit lexical scope
                self.local = prev;

                // If we reach here, the comprehension was expanded to a constant vector
                *expr = Expr::Const(Span::new(span, ConstantExpr::Vector(folded)));
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_mut_statement_block(
        &mut self,
        statements: &mut Vec<Statement>,
    ) -> ControlFlow<InvalidConstantError> {
        let mut current_statement = 0;

        let mut buffer = vec![];
        while current_statement < statements.len() {
            let num_statements = statements.len();
            match &mut statements[current_statement] {
                Statement::Let(ref mut expr) => {
                    // A `let` may only appear once in a statement block, and must be the
                    // last statement in the block
                    assert_eq!(
                        current_statement,
                        num_statements - 1,
                        "let is not in tail position of block"
                    );
                    // Visit the binding expression first
                    self.visit_mut_expr(&mut expr.value)?;
                    // Enter a new lexical scope
                    let prev = self.local.clone();
                    // If the value is constant, record it in our bindings map
                    let is_constant = expr.value.is_constant();
                    if is_constant {
                        match expr.value {
                            Expr::Const(ref value) => {
                                self.local.insert(expr.name, value.clone());
                            }
                            Expr::Range(ref range) => {
                                let vector =
                                    range.item.clone().into_iter().map(|i| i as u64).collect();
                                self.local.insert(
                                    expr.name,
                                    Span::new(range.span(), ConstantExpr::Vector(vector)),
                                );
                            }
                            _ => unreachable!(),
                        }
                    }

                    // Visit the let body
                    self.visit_mut_statement_block(&mut expr.body)?;

                    // If this let is constant, then the binding is no longer
                    // used in the body after constant propagation, flatten its
                    // body into the current block.
                    if is_constant {
                        buffer.append(&mut expr.body);
                    }

                    // Restore the previous scope
                    self.local = prev;
                }
                Statement::Enforce(ref mut expr) => {
                    self.visit_mut_enforce(expr)?;
                }
                Statement::EnforceAll(ref mut expr) => {
                    self.in_constraint_comprehension = true;
                    self.visit_mut_list_comprehension(expr)?;
                    self.in_constraint_comprehension = false;
                }
            }

            // If we have a non-empty buffer, then we are collapsing a let into the current block,
            // and that let must have been the last expression in the block, so as soon as we fold
            // its body into the current block, we're done
            if buffer.is_empty() {
                current_statement += 1;
                continue;
            }

            // Drop the let statement being folded in to this block
            statements.pop();

            // Append the buffer
            statements.append(&mut buffer);

            // We're done
            break;
        }

        ControlFlow::Continue(())
    }

    /// It should not be possible to reach this, as we handle statements at the block level
    fn visit_mut_statement(&mut self, _: &mut Statement) -> ControlFlow<InvalidConstantError> {
        panic!("unexpectedly reached visit_mut_statement");
    }
}
