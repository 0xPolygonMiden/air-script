use super::Scope;

/// Adds the required imports to the provided scope.
pub(super) fn add_imports(scope: &mut Scope) {
    // add plonky3 imports
    scope.import("p3_air", "Air");
    scope.import("p3_air", "AirBuilder");
    scope.import("p3_air", "AirBuilderWithPublicValues");
    scope.import("p3_air", "BaseAir");
    scope.import("p3_air", "BaseAirWithPublicValues");
    scope.import("p3_matrix", "Matrix");
    scope.import("p3_field", "PrimeCharacteristicRing");
}
