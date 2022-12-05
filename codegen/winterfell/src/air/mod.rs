use super::{AirIR, Impl, Scope};
use ir::TransitionConstraintDegree;

mod constants;
use constants::add_constants;

mod public_inputs;
use public_inputs::add_public_inputs_struct;

mod periodic_columns;
use periodic_columns::add_fn_get_periodic_column_values;

mod boundary_constraints;
use boundary_constraints::{add_fn_get_assertions, add_fn_get_aux_assertions};

mod transition_constraints;
use transition_constraints::{add_fn_evaluate_aux_transition, add_fn_evaluate_transition};

// HELPERS TO GENERATE AN IMPLEMENTATION OF THE WINTERFELL AIR TRAIT
// ================================================================================================

/// Updates the provided scope with a new Air struct and Winterfell Air trait implementation
/// which are equivalent the provided AirIR.
pub(super) fn add_air(scope: &mut Scope, ir: &AirIR) {
    // add constant declarations
    if !ir.constants().is_empty() {
        add_constants(scope, ir);
    }

    // add the Public Inputs struct and its base implementation.
    add_public_inputs_struct(scope, ir);

    let name = ir.air_name();

    // add the Air struct and its base implementation.
    add_air_struct(scope, ir, name);

    // add Winterfell Air trait implementation for the provided AirIR.
    add_air_trait(scope, ir, name);
}

/// Updates the provided scope with a custom Air struct.
fn add_air_struct(scope: &mut Scope, ir: &AirIR, name: &str) {
    // define the custom Air struct.
    let air_struct = scope
        .new_struct(name)
        .vis("pub")
        .field("context", "AirContext<Felt>");

    // add public inputs
    for (pub_input, pub_input_size) in ir.public_inputs() {
        air_struct.field(pub_input, format!("[Felt; {}]", pub_input_size));
    }

    // add the custom Air implementation block
    let base_impl = scope.new_impl(name);
    // add a simple method to get the last step.
    base_impl
        .new_fn("last_step")
        .arg_ref_self()
        .vis("pub")
        .ret("usize")
        .line("self.trace_length() - self.context().num_transition_exemptions()");
}

/// Updates the provided scope with the custom Air struct and an Air trait implementation based on
/// the provided AirIR.
fn add_air_trait(scope: &mut Scope, ir: &AirIR, name: &str) {
    // add the implementation block for the Air trait.
    let air_impl = scope
        .new_impl(name)
        .impl_trait("Air")
        .associate_type("BaseField", "Felt")
        .associate_type("PublicInputs", "PublicInputs");

    // add default function "context".
    let fn_context = air_impl
        .new_fn("context")
        .arg_ref_self()
        .ret("&AirContext<Felt>");
    fn_context.line("&self.context");

    // add the method implementations required by the AIR trait.
    add_fn_new(air_impl, ir);

    add_fn_get_periodic_column_values(air_impl, ir);

    add_fn_get_assertions(air_impl, ir);

    add_fn_get_aux_assertions(air_impl, ir);

    add_fn_evaluate_transition(air_impl, ir);

    add_fn_evaluate_aux_transition(air_impl, ir);
}

/// Adds an implementation of the "new" method to the referenced Air implementation based on the
/// data in the provided AirIR.
fn add_fn_new(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function.
    let new = impl_ref
        .new_fn("new")
        .arg("trace_info", "TraceInfo")
        .arg("public_inputs", "PublicInputs")
        .arg("options", "WinterProofOptions")
        .ret("Self");

    // define the transition constraint degrees of the main trace `main_degrees`.
    let main_degrees = ir
        .constraint_degrees(0)
        .iter()
        .map(|degree| degree.to_string(ir, false))
        .collect::<Vec<_>>();
    new.line(format!(
        "let main_degrees = vec![{}];",
        main_degrees.join(", ")
    ));

    // define the transition constraint degrees of the aux trace `aux_degrees`.
    let aux_degrees = ir
        .constraint_degrees(1)
        .iter()
        .map(|degree| degree.to_string(ir, true))
        .collect::<Vec<_>>();
    new.line(format!(
        "let aux_degrees = vec![{}];",
        aux_degrees.join(", ")
    ));

    // define the number of main trace boundary constraints `num_main_assertions`.
    new.line(format!(
        "let num_main_assertions = {};",
        ir.num_main_assertions()
    ));

    // define the number of aux trace boundary constraints `num_aux_assertions`.
    new.line(format!(
        "let num_aux_assertions = {};",
        ir.num_aux_assertions()
    ));

    // define the context.
    let context = "
let context = AirContext::new_multi_segment(
    trace_info,
    main_degrees,
    aux_degrees,
    num_main_assertions,
    num_aux_assertions,
    options,
)
.set_num_transition_exemptions(2);";

    new.line(context);

    // get public inputs
    let mut pub_inputs = Vec::new();
    for (pub_input, _) in ir.public_inputs() {
        pub_inputs.push(format!("{}: public_inputs.{}", pub_input, pub_input));
    }
    // return initialized Self.
    new.line(format!("Self {{ context, {} }}", pub_inputs.join(", ")));
}

// RUST STRING GENERATION
// ================================================================================================

/// Code generation trait for generating Rust code strings from boundary constraint expressions.
pub trait Codegen {
    fn to_string(&self, ir: &AirIR, is_aux_constraint: bool) -> String;
}

impl Codegen for TransitionConstraintDegree {
    fn to_string(&self, _ir: &AirIR, _is_aux_constraint: bool) -> String {
        if self.cycles().is_empty() {
            format!("TransitionConstraintDegree::new({})", self.base())
        } else {
            let cycles = self
                .cycles()
                .iter()
                .map(|cycle_len| cycle_len.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            format!(
                "TransitionConstraintDegree::with_cycles({}, vec![{}])",
                self.base(),
                cycles
            )
        }
    }
}
