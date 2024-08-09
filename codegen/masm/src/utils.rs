use crate::constants::AUX_TRACE;
use crate::writer::Writer;
use crate::{constants::MAIN_TRACE, error::CodegenError};
use air_ir::{ConstraintDomain, ConstraintRoot, TraceSegmentId};

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

/// Given a periodic column group position, returns a memory offset.
///
/// Periodic columns are grouped based on their length, this is done so that only a single z value
/// needs to be cached per group. The grouping is based on unique lengths, sorted from highest to
/// lowest. Given a periodic group, this function will return a memory offset, which can be used to
/// load the corresponding z value.
pub fn periodic_group_to_memory_offset(group: u32) -> u32 {
    // Each memory address contains a quadratic field extension element, this makes the code to
    // store/load the data more efficient, since it is easier to push/pop the high values of a
    // word. So below we have to multiply the group by 2, to account for the zero padding, and add
    // 1, to account for the data being at the low and not high part of the word.
    group * 2 + 1
}

/// Loads the `element` from a memory range starting at `base_addr`.
///
/// This function is used to load a qudratic element from memory, and discard the other value. Even
/// values are store in higher half of the word, while odd values are stored in the lower half.
pub fn load_quadratic_element(
    writer: &mut Writer,
    base_addr: u32,
    element: u32,
) -> Result<(), CodegenError> {
    let target_word: u32 = element / 2;
    let address = base_addr + target_word;

    // Load data from memory
    writer.padw();
    writer.mem_loadw(address);

    // Discard the other value
    match element % 2 {
        0 => {
            writer.movdn(3);
            writer.movdn(3);
            writer.drop();
            writer.drop();
        }
        1 => {
            writer.drop();
            writer.drop();
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub fn boundary_group_to_procedure_name(
    trace: TraceSegmentId,
    domain: ConstraintDomain,
) -> &'static str {
    match (trace, domain) {
        (MAIN_TRACE, ConstraintDomain::FirstRow) => "compute_boundary_constraints_main_first",
        (MAIN_TRACE, ConstraintDomain::LastRow) => "compute_boundary_constraints_main_last",
        (AUX_TRACE, ConstraintDomain::FirstRow) => "compute_boundary_constraints_aux_first",
        (AUX_TRACE, ConstraintDomain::LastRow) => "compute_boundary_constraints_aux_last",
        _ => panic!("Invalid boundary constraint"),
    }
}
