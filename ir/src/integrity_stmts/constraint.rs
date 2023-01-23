use super::NodeIndex;

/// A [ConstraintRoot] represents the entry node of a subgraph representing an integrity constraint
/// within the [AlgebraicGraph]. It also contains the row offset for the constraint which is the
/// maximum of all row offsets accessed by the constraint. For example, if a constraint only
/// accesses the trace in the current row then the row offset will be 0, but if it accesses the
/// trace in both the current and the next rows then the row offset will be 1.
#[derive(Debug, Clone)]
pub struct ConstraintRoot {
    index: NodeIndex,
    domain: ConstraintDomain,
}

impl ConstraintRoot {
    /// Creates a new [ConstraintRoot] with the specified entry index and row offset.
    pub fn new(index: NodeIndex, domain: ConstraintDomain) -> Self {
        Self { index, domain }
    }

    /// Returns the index of the entry node of the subgraph representing the constraint.
    pub fn node_index(&self) -> &NodeIndex {
        &self.index
    }

    /// Returns the [ConstraintDomain] for this constraint, which specifies the rows against which
    /// the constraint should be applied.
    pub fn domain(&self) -> ConstraintDomain {
        self.domain
    }
}

/// The domain to which the constraint is applied, which is either the first or last row (for
/// boundary constraints), every row (for validity constraints), or every frame (for transition
/// constraints). When the constraint is applied to a frame the inner value specifies the size of
/// the frame. For example, for a transition constraint that is applied against the current and next
/// rows, the frame size will be 2.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ConstraintDomain {
    FirstRow,          // for boundary constraints against the first row
    LastRow,           // for boundary constraints against the last row
    EveryRow,          // for validity constraints
    EveryFrame(usize), // for transition constraints
}

impl ConstraintDomain {
    /// Combines the two [ConstraintDomain]s into a single [ConstraintDomain] that represents the
    /// maximum constraint domain. For example, if one domain is [ConstraintDomain::EveryFrame(2)]
    /// and the other is [ConstraintDomain::EveryFrame(3)] then the result will be
    /// [ConstraintDomain::EveryFrame(3)].
    pub fn merge(&self, other: &ConstraintDomain) -> ConstraintDomain {
        if self == other {
            return *other;
        }

        match (self, other) {
            (ConstraintDomain::EveryFrame(a), ConstraintDomain::EveryFrame(b)) => {
                ConstraintDomain::EveryFrame(*a.max(b))
            }
            (ConstraintDomain::EveryFrame(a), _) => ConstraintDomain::EveryFrame(*a),
            (_, ConstraintDomain::EveryFrame(b)) => ConstraintDomain::EveryFrame(*b),
            // for any other pair of constraints which are not equal, the result of combining the
            // domains is to apply the constraint at every row.
            _ => ConstraintDomain::EveryRow,
        }
    }
}

impl From<usize> for ConstraintDomain {
    /// Creates a [ConstraintDomain] from the specified row offset.
    fn from(row_offset: usize) -> Self {
        if row_offset == 0 {
            ConstraintDomain::EveryRow
        } else {
            ConstraintDomain::EveryFrame(row_offset + 1)
        }
    }
}
