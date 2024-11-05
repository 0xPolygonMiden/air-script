#![allow(unused)]
use std::{
    cell::{Cell, RefCell, RefMut},
    rc::Rc,
};

use air_parser::ast::{Boundary as AstBoundary, Type};
use miden_diagnostics::{SourceSpan, Spanned};

use super::*;

pub trait Value: Spanned {}

pub trait Op {
    fn as_operation_mut(&self) -> RefMut<'_, Operation>;
}

/// We implement some functionality on `dyn Op` so that we can keep `Op` object safe, but still
/// provide some of the richer mutation/transformation actions we want to support.
impl dyn Op {
    /// Add `value` as a new operand of this operation, adding the resulting operand
    /// as a user of that value.
    pub fn append_operand(self: Rc<Self>, value: Rc<dyn Value>, span: SourceSpan) -> Rc<Operand> {
        let owner = RefCell::new(Rc::clone(&self));
        let mut operation = self.as_operation_mut();
        let index = operation.operands.len();
        let operand = Rc::new(Operand {
            span,
            index,
            value: RefCell::new(value),
            owner,
        });
        operation.operands.push(Rc::clone(&operand));
        operand
    }
}

pub struct Context {
    value_id: Cell<usize>,
}
impl Context {
    pub fn make_value_id(&self) -> usize {
        self.value_id.set(self.value_id.get() + 1);
        self.value_id.get()
    }
}

#[derive(Spanned)]
pub struct Operand {
    owner: RefCell<Rc<dyn Op>>,
    value: RefCell<Rc<dyn Value>>,
    #[span]
    span: SourceSpan,
    // the index of the operand in the operation
    index: usize,
}
impl Value for Operand {}

#[derive(Spanned)]
pub struct OpResult {
    owner: Option<Rc<dyn Op>>,
    // SSA id
    id: usize,
    #[span]
    span: SourceSpan,
}
impl OpResult {
    pub fn new(span: SourceSpan, owner: Rc<dyn Op>, context: &Context) -> Rc<Self> {
        Rc::new(Self {
            owner: Some(owner),
            id: context.make_value_id(),
            span,
        })
    }
}

#[derive(Spanned)]
pub struct Operation {
    pub operands: Vec<Rc<Operand>>,
    pub owner: Option<Rc<Block>>,
    pub result: RefCell<OpResult>,
    #[span]
    pub span: SourceSpan,
}

pub struct Add(RefCell<Operation>);
impl Add {
    pub fn new(lhs: Rc<dyn Value>, rhs: Rc<dyn Value>, span: SourceSpan) -> Rc<Add> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let lhs = Rc::clone(&lhs);
        let rhs = Rc::clone(&rhs);
        <dyn Op>::append_operand(operation.clone(), lhs, span);
        <dyn Op>::append_operand(operation.clone(), rhs, span);
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Add {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

pub struct Sub(RefCell<Operation>);
impl Sub {
    pub fn new(lhs: Rc<dyn Value>, rhs: Rc<dyn Value>, span: SourceSpan) -> Rc<Sub> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let lhs = Rc::clone(&lhs);
        let rhs = Rc::clone(&rhs);
        <dyn Op>::append_operand(operation.clone(), lhs, span);
        <dyn Op>::append_operand(operation.clone(), rhs, span);
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Sub {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

pub struct Mul(RefCell<Operation>);
impl Mul {
    pub fn new(lhs: Rc<dyn Value>, rhs: Rc<dyn Value>, span: SourceSpan) -> Rc<Mul> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let lhs = Rc::clone(&lhs);
        let rhs = Rc::clone(&rhs);
        <dyn Op>::append_operand(operation.clone(), lhs, span);
        <dyn Op>::append_operand(operation.clone(), rhs, span);
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Mul {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

pub struct Enf(RefCell<Operation>);
impl Enf {
    pub fn new(value: Rc<dyn Value>, span: SourceSpan) -> Rc<Enf> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let value = Rc::clone(&value);
        <dyn Op>::append_operand(operation.clone(), value, span);
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Enf {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

pub struct Call(RefCell<Operation>);
impl Call {
    pub fn new(func: Rc<dyn Value>, args: Vec<Rc<dyn Value>>, span: SourceSpan) -> Rc<Call> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let func = Rc::clone(&func);
        <dyn Op>::append_operand(operation.clone(), func, span);
        for arg in args {
            <dyn Op>::append_operand(operation.clone(), arg, span);
        }
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Call {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FoldOperatorOpts {
    Add,
    Mul,
}
#[derive(Spanned)]
pub struct FoldOperator {
    operator: FoldOperatorOpts,
    #[span]
    span: SourceSpan,
}
impl FoldOperator {
    pub fn new(operator: FoldOperatorOpts, span: SourceSpan) -> Rc<FoldOperator> {
        Rc::new(Self { operator, span })
    }
}
impl Value for FoldOperator {}

pub struct Fold(RefCell<Operation>);
impl Fold {
    pub fn new(
        iterable: Rc<dyn Value>,
        operator: Rc<dyn Value>,
        init: Rc<dyn Value>,
        span: SourceSpan,
    ) -> Rc<Fold> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let iterable = Rc::clone(&iterable);
        let operator = Rc::clone(&operator);
        let init = Rc::clone(&init);
        let operator = <dyn Op>::append_operand(operation.clone(), iterable, span);
        let operator = <dyn Op>::append_operand(operation.clone(), operator, span);
        let operator = <dyn Op>::append_operand(operation.clone(), init, span);
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Fold {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

pub struct For(RefCell<Operation>);
impl For {
    pub fn new(
        iterators: Vec<Rc<dyn Value>>,
        body: Rc<dyn Value>,
        selector: Option<Rc<dyn Value>>,
        span: SourceSpan,
    ) -> Rc<For> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        for iterator in iterators {
            <dyn Op>::append_operand(operation.clone(), iterator.clone(), span);
        }
        <dyn Op>::append_operand(operation.clone(), body.clone(), span);
        if let Some(selector) = selector {
            <dyn Op>::append_operand(operation.clone(), selector.clone(), span);
        }
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for For {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

pub struct If(RefCell<Operation>);
impl If {
    pub fn new(
        condition: Rc<dyn Value>,
        then_branch: Rc<dyn Value>,
        else_branch: Rc<dyn Value>,
        span: SourceSpan,
    ) -> Rc<If> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let condition = Rc::clone(&condition);
        let then_branch = Rc::clone(&then_branch);
        let else_branch = Rc::clone(&else_branch);
        <dyn Op>::append_operand(operation.clone(), condition, span);
        <dyn Op>::append_operand(operation.clone(), then_branch, span);
        <dyn Op>::append_operand(operation.clone(), else_branch, span);
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for If {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

#[derive(Spanned)]
pub struct Vector {
    operation: RefCell<Operation>,
    #[span]
    pub span: SourceSpan,
}
impl Vector {
    pub fn new(values: Vec<Rc<dyn Value>>, span: SourceSpan) -> Rc<Vector> {
        let operation = Rc::new(Self {
            operation: RefCell::new(Operation {
                operands: vec![],
                owner: None,
                result: RefCell::new(OpResult {
                    owner: None,
                    id: 0,
                    span,
                }),
                span,
            }),
            span,
        });
        for value in values {
            <dyn Op>::append_operand(operation.clone(), value, span);
        }
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Vector {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.operation.borrow_mut()
    }
}
impl Value for Vector {}

pub struct Matrix(RefCell<Operation>);
impl Matrix {
    pub fn new(values: Vec<Vec<Rc<dyn Value>>>, span: SourceSpan) -> Rc<Matrix> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        for row in values {
            let row = Vector::new(row, span);
            <dyn Op>::append_operand(operation.clone(), row, span);
        }
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Matrix {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

#[derive(Spanned)]
pub struct BoundaryOpts {
    pub boundary: AstBoundary,
    #[span]
    pub span: SourceSpan,
}
impl Value for BoundaryOpts {}

pub struct Boundary(RefCell<Operation>);
impl Boundary {
    pub fn new(boundary: BoundaryOpts, value: Rc<dyn Value>, span: SourceSpan) -> Rc<Boundary> {
        let operation = Rc::new(Self(RefCell::new(Operation {
            operands: vec![],
            owner: None,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        })));
        let value = Rc::clone(&value);
        <dyn Op>::append_operand(operation.clone(), value, span);
        let dyn_op = Rc::clone(&operation);
        operation.as_operation_mut().result.borrow_mut().owner = Some(dyn_op);
        operation
    }
}
impl Op for Boundary {
    fn as_operation_mut(&self) -> RefMut<'_, Operation> {
        self.0.borrow_mut()
    }
}

// old name was Variable
#[derive(Spanned)]
pub struct Parameter {
    pub ty: Type,
    pub position: usize,
    #[span]
    pub span: SourceSpan,
}

pub struct Block {
    pub operations: Vec<Operation>,
    pub owner: Option<Rc<Region>>,
    pub span: SourceSpan,
}

pub struct Region {
    pub blocks: Vec<Block>,
    pub owner: Option<Rc<Function>>,
    pub span: SourceSpan,
}

#[derive(Spanned)]
pub struct Function {
    pub name: QualifiedIdentifier,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub blocks: Vec<Block>,
    #[span]
    pub span: SourceSpan,
}
