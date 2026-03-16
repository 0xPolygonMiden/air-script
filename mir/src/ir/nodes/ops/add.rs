use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent, Singleton};

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
}

impl Add {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        Op::Add(Self { lhs, rhs, span, ..Default::default() }).into()
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
        self.parents.retain(|p| match p.to_link() {
            Some(link) => link != parent,
            None => true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove_parent_removes_only_target() {
        let span = SourceSpan::default();

        let lhs: Link<Op> = Op::None(span).into();
        let rhs: Link<Op> = Op::None(span).into();

        let mut add = Add {
            parents: Vec::new(),
            lhs,
            rhs,
            _node: Singleton::default(),
            _owner: Singleton::default(),
            span,
        };

        let owner1: Link<Owner> = Owner::None(span).into();
        let owner2: Link<Owner> = Owner::None(span).into();

        add.add_parent(owner1.clone());
        add.add_parent(owner2.clone());
        assert_eq!(add.get_parents().len(), 2);

        add.remove_parent(owner1.clone());

        let parents = add.get_parents();
        assert_eq!(parents.len(), 1);
        let remaining =
            parents[0].to_link().expect("BackLink<Owner> should upgrade to Link<Owner>");
        assert_eq!(remaining, owner2);
    }
}
