use super::Scope;

/// Adds the required imports to the provided scope.
pub(super) fn add_imports(scope: &mut Scope) {
    // add winterfell imports
    scope.import(
        "winter_air::TransitionConstraintDegree",
        "TransitionConstraintDegree",
    );
    scope.import("winter_air::TraceInfo", "TraceInfo");
    scope.import("winter_air::ProofOptions", "WinterProofOptions");
    scope.import("winter_air::EvaluationFrame", "EvaluationFrame");
    scope.import("winter_air::Assertion", "Assertion");
    scope.import("winter_air::AirContext", "AirContext");
    scope.import("winter_air::Air", "Air");

    scope.import("winter_utils::collections::Vec", "Vec");
}
