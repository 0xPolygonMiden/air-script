use air_ir::{ConstraintDomain, ConstraintRoot};

/// Utility to order contraints roots w.r.t. their natural order.
pub fn contraint_root_domain(boundary: &&ConstraintRoot) -> u8 {
    // TODO: Sort by the column index. Issue #315
    match boundary.domain() {
        ConstraintDomain::FirstRow => 0,
        ConstraintDomain::LastRow => 1,
        ConstraintDomain::EveryRow => panic!("EveryRow is not supported"),
        ConstraintDomain::EveryFrame(_) => panic!("EveryFrame is not supported"),
    }
}
