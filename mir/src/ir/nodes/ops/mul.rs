use air_types::{BinType, ScalarTypeMut, TypeMut, Typing};
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to represent the multiplication of two MIR ops, `lhs` and `rhs`
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Mul {
    pub parents: Vec<BackLink<Owner>>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub _bin_ty: BinType,
}

impl ScalarTypeMut for Mul {
    fn scalar_ty_mut(&mut self) -> &mut Option<air_types::ScalarType> {
        self._bin_ty.scalar_ty_mut()
    }
}

impl TypeMut for Mul {
    fn ty_mut(&mut self) -> &mut Option<air_types::Type> {
        self._bin_ty.ty_mut()
    }
}

impl Typing for Mul {
    fn ty(&self) -> Option<air_types::Type> {
        self._bin_ty.ty()
    }
}

impl BuilderHook for Mul {
    fn finalize_hook(&mut self) {
        self._bin_ty = BinType::Mul(self.lhs.borrow().ty(), self.rhs.borrow().ty(), None);
        let res = self._bin_ty.infer_bin_ty_add().unwrap();
        *self._bin_ty.result_mut() = res;
    }
}

impl Mul {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let mut mul = Self {
            lhs,
            rhs,
            span,
            _bin_ty: BinType::default(),
            ..Default::default()
        };
        mul.finalize_hook();
        Op::Mul(mul).into()
    }
}

impl Parent for Mul {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Mul {
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
