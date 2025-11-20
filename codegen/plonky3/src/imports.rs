use super::Scope;

/// Adds the required imports to the provided scope.
pub(super) fn add_imports(scope: &mut Scope) {
    // add plonky3 imports
    scope.import("p3_field", "ExtensionField");
    scope.import("p3_field", "Field");
    scope.import("p3_field", "PrimeCharacteristicRing");
    scope.import("p3_matrix", "Matrix");
    scope.import("p3_miden_air", "MidenAir");
    scope.import("p3_miden_air", "MidenAirBuilder");
}
