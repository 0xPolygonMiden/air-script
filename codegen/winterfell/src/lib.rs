use codegen::{Impl, Scope};
use ir::AirIR;

mod imports;
use imports::add_imports;

mod air;
use air::add_air;

// GENERATE RUST CODE FOR WINTERFELL AIR
// ================================================================================================

/// CodeGenerator is used to generate a Rust implementation of the Winterfell STARK prover library's
/// Air trait. The generated Air expresses the constraints specified by the AirIR used to build the
/// CodeGenerator.
pub struct CodeGenerator {
    scope: Scope,
}

impl CodeGenerator {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    /// Builds a new Rust scope that represents a Winterfell Air trait implementation for the
    /// provided AirIR.
    pub fn new(ir: &AirIR) -> Self {
        let mut scope = Scope::new();

        // add winterfell imports.
        add_imports(&mut scope);

        // add an Air struct and Winterfell Air trait implementation for the provided AirIR.
        add_air(&mut scope, ir);

        Self { scope }
    }

    /// Returns a string of Rust code containing a Winterfell Air implementation for the AirIR with
    /// which this [CodeGenerator] was instantiated.
    pub fn generate(&self) -> String {
        self.scope.to_string()
    }
}
