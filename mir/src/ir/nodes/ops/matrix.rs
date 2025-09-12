use air_types::{Kind, ScalarTypeMut, TypeMut, Typing};
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to represent a matrix of MIR ops of a given size
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Matrix {
    pub parents: Vec<BackLink<Owner>>,
    pub size: usize,
    // elements are of type Vector
    pub elements: Link<Vec<Link<Op>>>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub _kind: Option<Kind>,
}

impl ScalarTypeMut for Matrix {
    fn scalar_ty_mut(&mut self) -> &mut Option<air_types::ScalarType> {
        self._kind.as_mut().unwrap().scalar_ty_mut()
    }
}

impl TypeMut for Matrix {
    fn ty_mut(&mut self) -> &mut Option<air_types::Type> {
        self._kind.as_mut().unwrap().ty_mut()
    }
}

impl Typing for Matrix {
    fn ty(&self) -> Option<air_types::Type> {
        self._kind.ty()
    }
}

impl BuilderHook for Matrix {
    fn finalize_hook(&mut self) {
        self._kind = self.elements.borrow().kind();
    }
}

impl Matrix {
    pub fn create(elements: Vec<Link<Op>>, span: SourceSpan) -> Link<Op> {
        let size = elements.len();
        let mut mat = Self {
            size,
            elements: Link::new(elements),
            span,
            ..Default::default()
        };
        mat.finalize_hook();
        Op::Matrix(mat).into()
    }
}

impl Parent for Matrix {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.elements.clone()
    }
}

impl Child for Matrix {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.parents.clone()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.push(parent.into());
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.retain(|p| *p != parent.clone().into());
    }
}
