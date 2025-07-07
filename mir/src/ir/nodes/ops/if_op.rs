use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to represent conditional constraints
///
/// Notes:
/// - the If operation will be unrolled into a Vector during the Unrolling pass, combining the then
///   and else branches. For example, If(s, vec![a, b], vec![c]) will be unrolled into: vec![s * a,
///   s * b, (1 - s) * c]
/// - After the Unrolling pass, no If ops should be present in the graph
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct If {
    pub parents: Vec<BackLink<Owner>>,
    pub match_arms: Link<Vec<MatchArm>>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct MatchArm {
    pub condition: Link<Op>,
    pub expr: Link<Op>,
}

impl MatchArm {
    pub fn new(expr: Link<Op>, condition: Link<Op>) -> Self {
        Self { condition, expr }
    }
}

impl If {
    pub fn create(match_arms: Vec<MatchArm>, span: SourceSpan) -> Link<Op> {
        Op::If(Self {
            match_arms: match_arms.into(),
            span,
            ..Default::default()
        })
        .into()
    }
}

impl Parent for If {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(
            self.match_arms
                .borrow()
                .iter()
                .flat_map(|arm| vec![arm.condition.clone(), arm.expr.clone()])
                .collect(),
        )
    }
}

impl Child for If {
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
