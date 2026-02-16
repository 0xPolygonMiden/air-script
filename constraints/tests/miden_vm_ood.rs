//! OOD evaluation parity test for the System+Range group of MidenVM AIR.
//!
//! We mirror the miden-vm evaluation flow and compare against fixed outputs captured from miden-vm.

mod fixtures;
mod helpers;

use fixtures::ood_group::active_group;
use helpers::ood::run_group_parity_test;

#[test]
fn test_miden_vm_system_range_ood_evals_match() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| run_group_parity_test(active_group()))
        .unwrap()
        .join()
        .unwrap();
}
