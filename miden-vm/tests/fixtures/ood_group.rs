//! Constraint group manifest for the current OOD parity testing group.

use super::ood_expected::{expected_ood_evals, CONSTRAINT_NAMES};
use crate::helpers::ood::GroupManifest;

/// Entry point for the group of active tags in parity tests.
pub fn active_group() -> GroupManifest<'static> {
    GroupManifest::new("constraints/miden_vm.air", expected_ood_evals(), &CONSTRAINT_NAMES)
}
