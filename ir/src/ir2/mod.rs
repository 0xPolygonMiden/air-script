#![allow(unused)]
use std::{
    cell::{Cell, RefCell, RefMut},
    rc::Rc,
};

use air_parser::ast::Type;
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

pub struct Operation {
    pub operands: Vec<Rc<Operand>>,
    pub owner: Option<Rc<Block>>,
    pub result: RefCell<OpResult>,
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
    pub params: Vec<Type>,
    pub result: Type,
    pub blocks: Vec<Block>,
    #[span]
    pub span: SourceSpan,
}
