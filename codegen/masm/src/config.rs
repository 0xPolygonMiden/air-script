use crate::constants;

pub struct CodegenConfig {
    // Memory location of the trace length using the following format:
    //
    //      [trace_len_address] => [trace_len, 0, 0, 0]
    pub trace_len_address: u32,

    // Memory location of the out-of-domain value using the following format:
    //
    //      [z_address] => [z8_0, z8_1, z_0, z_1]
    pub z_address: u32,

    // Memory range for the OOD main frame values, starting at this value going up to
    // `ood_frame_address + main_frame_width`. Each memory location contains the values of the
    // current and next rows using the following format:
    //
    //      [ood_frame_address+0] => [ood_curr_0, ood_curr_1, ood_next_0, ood_next_1]
    pub ood_frame_address: u32,

    // Memory range for the OOD auxiliary frame values, starting at this value going up to
    // `ood_aux_frame_address + aux_frame_width`. Each memory location contains the values of the
    // current and next rows using the following format:
    //
    //      [ood_aux_frame_address+0] => [ood_aux_curr_0, ood_aux_curr_1, ood_aux_next_0, ood_aux_next_1]
    pub ood_aux_frame_address: u32,

    // Memory range for the composition coefficients.
    //
    // The coefficients are organized as follows:
    //
    // 1. Transition constraint coefficients for the main trace
    // 2. Transition constraint coefficients for the aux trace
    // 3. Boundary constraint coefficients for the main trace
    // 4. Boundary constraint coefficients for the aux trace
    //
    pub composition_coef_address: u32,

    // Memory range for the public inputs.
    pub public_inputs_address: u32,

    pub aux_rand_address: u32,
    pub periodic_values_address: u32,

    /// Memory range used to store exponentiations of Z, each address contains one point to be used
    /// on the evaluation of each periodic polynimal.
    pub z_exp_address: u32,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        CodegenConfig {
            trace_len_address: constants::TRACE_LEN_ADDRESS,
            z_address: constants::Z_ADDRESS,
            ood_frame_address: constants::OOD_FRAME_ADDRESS,
            ood_aux_frame_address: constants::OOD_AUX_FRAME_ADDRESS,
            composition_coef_address: constants::COMPOSITION_COEF_ADDRESS,
            public_inputs_address: constants::PUBLIC_INPUTS_ADDRESS,
            aux_rand_address: constants::AUX_RAND_ELEM_PTR,
            periodic_values_address: constants::PERIODIC_VALUES_ADDRESS,
            z_exp_address: constants::Z_EXP_ADDRESS,
        }
    }
}
