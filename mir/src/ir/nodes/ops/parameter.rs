use std::hash::{Hash, Hasher};

use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Singleton};

/// A MIR operation to represent a `Parameter` in a function or evaluator.
/// Also used in If and For loops to represent declared parameters.
#[derive(Builder, Default, Clone, Eq, Debug, Spanned)]
#[enum_wrapper(Op)]
pub struct Parameter {
    parents: Vec<BackLink<Owner>>,
    /// The node that this `Parameter` is referencing (Function, Evaluator, If, For)
    pub ref_node: BackLink<Owner>,
    /// The position of the `Parameter` in the referred node's `Parameter` list
    pub position: usize,
    pub _node: Singleton<Node>,
    #[span]
    pub span: SourceSpan,
    /// The type of the `Parameter`
    pub ty: Option<Type>,
}

impl ScalarTypeMut for Parameter {
    fn update_scalar_ty_unchecked(&mut self, new_sty: Option<ScalarType>) {
        self.ty.update_scalar_ty_unchecked(new_sty);
    }
}

impl TypeMut for Parameter {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self.ty = new_ty;
    }
}

impl Typing for Parameter {
    fn ty(&self) -> Option<Type> {
        self.ty.ty()
    }
}

impl BuilderHook for Parameter {}

impl Parameter {
    pub fn create(position: usize, ty: Type, span: SourceSpan) -> Link<Op> {
        Op::Parameter(Self {
            parents: Vec::default(),
            ref_node: BackLink::none(),
            position,
            ty: Some(ty),
            _node: Singleton::none(),
            span,
        })
        .into()
    }

    pub fn set_ref_node(&mut self, ref_node: Link<Owner>) {
        self.ref_node = ref_node.into();
    }
}

fn get_hash<T: Hash>(t: &T) -> u64 {
    let mut s = std::hash::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl PartialEq for Parameter {
    /// PartialEq uses the ref_node's hash to compare the nodes to allow comparing multiple
    /// instances of of the same graph (memory locations may differ)
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.ty == other.ty
            // TODO: This always returns true.
            // fix this by inserting unique ids in the nodes in a
            // linearized order in place of the hash.
            // See the relationship between [crate::ir::Bus] and [crate::ir::BusOp]
            // and their use in [crate::ir::Graph::insert_bus] for an example.
            && get_hash(&self.ref_node) == get_hash(&other.ref_node)
    }
}

impl Hash for Parameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.ty.hash(state);
        // TODO: This always returns true.
        // fix this by inserting unique ids in the nodes in a
        // linearized order in place of the hash.
        // See the relationship between [crate::ir::Bus] and [crate::ir::BusOp]
        // and their use in [crate::ir::Graph::insert_bus] for an example.
        self.ref_node.hash(state);
    }
}

impl Child for Parameter {
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
