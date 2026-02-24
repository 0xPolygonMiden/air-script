use std::hash::Hash;

use miden_diagnostics::{SourceSpan, Spanned};

use super::MirType;
use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, OwnerId, Singleton};

/// A MIR operation to represent a `Parameter` in a function or evaluator.
/// Also used in If and For loops to represent declared parameters.
#[derive(Builder, Default, Clone, Eq, Debug, Spanned)]
#[enum_wrapper(Op)]
pub struct Parameter {
    parents: Vec<BackLink<Owner>>,
    /// Stable id of the owner this Parameter references (Function, Evaluator, If, For)
    pub owner_id: OwnerId,
    /// The position of the `Parameter` in the referred node's `Parameter` list
    pub position: usize,
    /// The type of the `Parameter`
    pub ty: MirType,
    /// True if this parameter is a placeholder for a For output element.
    /// This distinguishes synthetic "per-iteration" placeholders from real parameters so we
    /// don't accidentally merge identities during duplication/caching.
    pub is_for_output: bool,
    pub _node: Singleton<Node>,
    #[span]
    pub span: SourceSpan,
}

impl Parameter {
    pub fn create(position: usize, ty: MirType, span: SourceSpan) -> Link<Op> {
        Op::Parameter(Self {
            parents: Vec::default(),
            owner_id: OwnerId::default(),
            position,
            ty,
            is_for_output: false,
            _node: Singleton::none(),
            span,
        })
        .into()
    }

    pub fn set_ref_node(&mut self, ref_node: Link<Owner>) {
        self.owner_id = ref_node.owner_id();
    }

    pub fn set_owner_id(&mut self, owner_id: OwnerId) {
        self.owner_id = owner_id;
    }

    pub fn set_for_output(&mut self, is_for_output: bool) {
        self.is_for_output = is_for_output;
    }
}

impl PartialEq for Parameter {
    /// Parameters compare on position, type, and stable owner id.
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.ty == other.ty
            && self.owner_id == other.owner_id
            // Keep For output placeholders distinct from "regular" parameters.
            && self.is_for_output == other.is_for_output
    }
}

impl Hash for Parameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.ty.hash(state);
        self.owner_id.hash(state);
        // Hash includes is_for_output to avoid collisions across placeholder vs real params.
        self.is_for_output.hash(state);
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
