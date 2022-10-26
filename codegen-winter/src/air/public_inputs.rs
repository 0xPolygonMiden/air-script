use super::Scope;
use ir::AirIR;

/// Updates the provided scope with a public inputs.
pub(super) fn add_public_inputs_struct(scope: &mut Scope, ir: &AirIR) {
    let name = "PublicInputs";
    // define the PublicInputs struct.
    let pub_inputs_struct = scope.new_struct(name).vis("pub");

    for (pub_input, pub_input_size) in ir.public_inputs() {
        pub_inputs_struct.field(pub_input, format!("[Felt; {}]", pub_input_size));
    }

    // add the public inputs implementation block
    let base_impl = scope.new_impl(name);

    let pub_inputs_values: Vec<String> = ir
        .public_inputs()
        .iter()
        .map(|input| input.0.clone())
        .collect();

    // add a constructor for public inputs
    let new_fn = base_impl
        .new_fn("new")
        .vis("pub")
        .ret("Self")
        .line(format!("Self {{ {} }}", pub_inputs_values.join(", ")));
    for (pub_input, pub_input_size) in ir.public_inputs() {
        new_fn.arg(pub_input, format!("[Felt; {}]", pub_input_size));
    }
}
