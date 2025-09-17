use std::hash::Hash;

use air_parser::ast::{Access, AccessType};
use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to represent accessing a given op, `indexable`, in two different ways:
/// - access_type: AccessType, which describes for example how to access a given index for a Vector
///   (e.g. `v[0]`)
/// - offset: usize, which describes the row offset for a trace column access (e.g. `a'`)
#[derive(Hash, Clone, PartialEq, Eq, Debug, Builder, Spanned, Default)]
#[enum_wrapper(Op)]
pub struct Accessor {
    pub parents: Vec<BackLink<Owner>>,
    pub indexable: Link<Op>,
    pub access_type: AccessType,
    pub offset: usize,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub _ty: Option<Type>,
}

impl ScalarTypeMut for Accessor {
    fn update_scalar_ty_unchecked(&mut self, new_ty: Option<ScalarType>) {
        self._ty.update_scalar_ty_unchecked(new_ty);
    }
}

impl TypeMut for Accessor {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self._ty = new_ty;
    }
}

impl Typing for Accessor {
    fn ty(&self) -> Option<Type> {
        self._ty.ty()
    }
}

impl BuilderHook for Accessor {
    fn finalize_hook(&mut self) {
        self._ty = self
            .indexable
            .borrow()
            .ty()
            .map(|ty| ty.access(self.access_type.clone()).unwrap());
    }
}

impl Accessor {
    pub fn create(
        indexable: Link<Op>,
        access_type: AccessType,
        offset: usize,
        span: SourceSpan,
    ) -> Link<Op> {
        let mut accessor = Self {
            indexable,
            access_type,
            offset,
            span,
            ..Default::default()
        };
        accessor.finalize_hook();
        Op::Accessor(accessor).into()
    }
}

impl Parent for Accessor {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.indexable.clone()])
    }
}

impl Child for Accessor {
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
