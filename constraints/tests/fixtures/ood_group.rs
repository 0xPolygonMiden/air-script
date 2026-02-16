//! Constraint group manifest for System+Range OOD parity testing.

use super::ood_expected::{expected_ood_evals, CONSTRAINT_NAMES};
use crate::helpers::ood::GroupManifest;

pub fn system_range_group() -> GroupManifest<'static> {
    GroupManifest::new("miden_vm.air", expected_ood_evals(), &CONSTRAINT_NAMES)
}

/// Entry point for the active tagged group in parity tests.
pub fn active_group() -> GroupManifest<'static> {
    system_range_group()
}
