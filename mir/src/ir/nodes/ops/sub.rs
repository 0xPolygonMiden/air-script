use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to represent the subtraction of two MIR ops, `lhs` and `rhs`
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Sub {
    pub parents: Vec<BackLink<Owner>>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub _bin_ty: BinType,
}

impl ScalarTypeMut for Sub {
    fn update_scalar_ty_unchecked(&mut self, new_sty: Option<ScalarType>) {
        self._bin_ty.update_scalar_ty_unchecked(new_sty);
    }
}

impl TypeMut for Sub {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self._bin_ty.update_ty_unchecked(new_ty);
    }
}

impl Typing for Sub {
    fn ty(&self) -> Option<Type> {
        self._bin_ty.ty()
    }
}

impl BuilderHook for Sub {
    fn finalize_hook(&mut self) {
        self._bin_ty = BinType::Sub(self.lhs.borrow().ty(), self.rhs.borrow().ty(), None);
        let res = self._bin_ty.infer_bin_ty_sub().unwrap();
        *self._bin_ty.result_mut() = res;
    }
}

impl Sub {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let mut sub = Self {
            lhs,
            rhs,
            span,
            _bin_ty: BinType::default(),
            ..Default::default()
        };
        sub.finalize_hook();
        Op::Sub(sub).into()
    }
}

impl Parent for Sub {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Sub {
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
