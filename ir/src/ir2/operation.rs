use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};

use air_parser::ast::{Boundary as AstBoundary, QualifiedIdentifier, Type};
use miden_diagnostics::{SourceSpan, Spanned};

use super::*;

pub trait Value: Spanned {}

impl Debug for dyn Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl PartialEq for dyn Value {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl Eq for dyn Value {}

pub trait Op {
    fn as_operation_mut(&self) -> RefMut<'_, Operation>;
    fn as_operation(&self) -> Ref<'_, Operation>;
}
/// We implement some functionality on `dyn Op` so that we can keep `Op` object safe, but still
/// provide some of the richer mutation/transformation actions we want to support.
impl dyn Op {
    /// Add `value` as a new operand of this operation, adding the resulting operand
    /// as a user of that value.
    pub fn append_operand(self: Rc<Self>, value: Rc<dyn Value>, span: SourceSpan) -> Rc<Operand> {
        let mut operation = self.as_operation_mut();
        operation.append_operand(value, span)
    }
}
impl Debug for dyn Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = self.as_operation_mut();
        write!(f, "{:?}", op)
    }
}
impl PartialEq for dyn Op {
    fn eq(&self, other: &Self) -> bool {
        let op = self.as_operation_mut();
        let other = other.as_operation_mut();
        op.eq(&other)
    }
}
impl Eq for dyn Op {}

pub struct Context {
    value_id: Cell<usize>,
}
impl Context {
    pub fn make_value_id(&self) -> usize {
        self.value_id.set(self.value_id.get() + 1);
        self.value_id.get()
    }
}

#[derive(Spanned, Clone, Debug, Eq)]
pub struct Operand {
    owner: Rc<Operation>,
    value: Rc<dyn Value>,
    #[span]
    span: SourceSpan,
    // the index of the operand in the operation
    index: usize,
}
impl Value for Operand {}
impl PartialEq for Operand {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value.clone()
    }
}

#[derive(Spanned, Clone, Debug)]
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

#[derive(Spanned, Clone, Debug)]
pub struct Operation {
    pub operands: Vec<Rc<Operand>>,
    pub owner: Option<Rc<Block>>,
    pub result: RefCell<OpResult>,
    #[span]
    pub span: SourceSpan,
}
impl Operation {
    pub fn new(operands: Vec<Rc<Operand>>, owner: Option<Rc<Block>>, span: SourceSpan) -> Self {
        let mut op = Self {
            operands: vec![],
            owner,
            result: RefCell::new(OpResult {
                owner: None,
                id: 0,
                span,
            }),
            span,
        };
        for operand in operands {
            let operand = Rc::clone(&operand);
            op.append_operand(operand, span);
        }
        op
    }
    pub fn append_operand(&mut self, value: Rc<dyn Value>, span: SourceSpan) -> Rc<Operand> {
        let index = self.operands.len();
        let operand = Rc::new(Operand {
            span,
            index,
            value: Rc::clone(&value),
            owner: Rc::new(self.clone()),
        });
        self.operands.push(Rc::clone(&operand));
        operand
    }
}
impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        let op = self;
        let other = other;
        op == other
    }
}
impl Eq for Operation {}

pub struct Add(RefCell<Operation>);
impl Add {
    pub fn new(
        lhs: Rc<dyn Value>,
        rhs: Rc<dyn Value>,
        owner: Rc<Block>,
        span: SourceSpan,
    ) -> Rc<Add> {
        let operation = Rc::new(Self(RefCell::new(Operation::new(
            vec![],
            Some(owner),
            span,
        ))));
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.operation.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
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
    fn as_operation(&self) -> Ref<'_, Operation> {
        self.0.borrow()
    }
}

// old name was Variable
#[derive(Spanned, Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub ty: Type,
    pub position: usize,
    #[span]
    pub span: SourceSpan,
}

#[derive(Spanned, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub operations: Vec<Operation>,
    pub owner: Option<Rc<Region>>,
    #[span]
    pub span: SourceSpan,
}

#[derive(Spanned, Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub blocks: Vec<Block>,
    pub owner: Option<Rc<Function>>,
    #[span]
    pub span: SourceSpan,
}

#[derive(Spanned, Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: QualifiedIdentifier,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub blocks: Vec<Block>,
    #[span]
    pub span: SourceSpan,
}
impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Spanned, Clone, Debug, PartialEq, Eq)]
pub struct Evaluator {
    pub name: QualifiedIdentifier,
    pub params: Vec<Parameter>,
    pub blocks: Vec<Block>,
    #[span]
    pub span: SourceSpan,
}
impl Hash for Evaluator {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FunctionOrEvaluator {
    Function(Rc<Function>),
    Evaluator(Rc<Evaluator>),
}
