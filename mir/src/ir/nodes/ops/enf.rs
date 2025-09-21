use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to enforce that a given MIR op, `expr` equals zero
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Enf {
    pub parents: Vec<BackLink<Owner>>,
    pub expr: Link<Op>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub _ty: Option<Type>,
}

impl ScalarTypeMut for Enf {
    fn update_scalar_ty_unchecked(&mut self, new_sty: Option<ScalarType>) {
        self._ty.update_scalar_ty_unchecked(new_sty);
    }
}

impl TypeMut for Enf {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self._ty = new_ty;
    }
}

impl Typing for Enf {
    fn ty(&self) -> Option<Type> {
        self._ty.ty()
    }
}

impl BuilderHook for Enf {
    fn finalize_hook(&mut self) {
        self._ty = self.expr.borrow().ty();
    }
}

impl Enf {
    pub fn create(expr: Link<Op>, span: SourceSpan) -> Link<Op> {
        let mut enf = Self { expr, span, ..Default::default() };
        enf.finalize_hook();
        Op::Enf(enf).into()
    }
}

impl Parent for Enf {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.expr.clone()])
    }
}

impl Child for Enf {
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
