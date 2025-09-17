use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to represent the addition of two MIR ops, `lhs` and `rhs`
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Add {
    pub parents: Vec<BackLink<Owner>>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub _bin_ty: BinType,
}

impl ScalarTypeMut for Add {
    fn update_scalar_ty_unchecked(&mut self, new_ty: Option<ScalarType>) {
        self._bin_ty.update_scalar_ty_unchecked(new_ty);
    }
}

impl TypeMut for Add {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self._bin_ty.update_ty_unchecked(new_ty);
    }
}

impl Typing for Add {
    fn ty(&self) -> Option<Type> {
        self._bin_ty.ty()
    }
}

impl BuilderHook for Add {
    fn finalize_hook(&mut self) {
        self._bin_ty = BinType::Add(self.lhs.borrow().ty(), self.rhs.borrow().ty(), None);
        let res = self._bin_ty.infer_bin_ty_add().unwrap();
        *self._bin_ty.result_mut() = res;
    }
}

impl Add {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let mut add = Self {
            lhs,
            rhs,
            span,
            _bin_ty: BinType::default(),
            ..Default::default()
        };
        add.finalize_hook();
        Op::Add(add).into()
    }
}

impl Parent for Add {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Add {
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
