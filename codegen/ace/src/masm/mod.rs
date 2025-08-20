use std::collections::{BTreeMap, HashMap};

use air_ir::{Air, BusType};
use anyhow::Ok;
use ood_frames::generate_ood_frames_module;
use public_inputs::generate_public_inputs;

use crate::{
    AceCircuit,
    masm::{
        constraints_eval::generate_constraints_eval_module,
        deep_queries::generate_deep_queries_module, verifier::generate_verifier_module,
    },
};

mod constraints_eval;
mod deep_queries;
mod ood_frames;
mod public_inputs;
mod verifier;

// CONSTANTS
// ================================================================================================

const FIELD_EXTENSION_DEGREE: usize = 2;
const NUM_CONSTRAINTS_COMPOSITION_POLYS: usize = 8;
const DOUBLE_WORD_SIZE: usize = 8;

// MASM CODE GENERATOR
// ================================================================================================

/// Generates the modules of a Miden assembly (MASM) STARK verifier using the core verifier in
/// the Miden standard library `stdlib`.
///
/// The following assumptions are made:
///
/// 1. Number of constraints composition polynomials is set to 8,
/// 2. FRI folding factor is set to 4,
/// 3. Extension degree of the cryptographic field is 2.
pub fn generate_masm_verifier(air: &Air, circuit: &AceCircuit) -> anyhow::Result<MasmVerifier> {
    // generate the parameters needed during code generation
    let masm_verifier_parameters = MasmVerifierParameters::from_air(air);
    // get the encoded circuit for the ACE chiplet
    let encoded_circuit = circuit.to_ace();

    // generate the different modules
    let deep_queries: String = generate_deep_queries_module(&masm_verifier_parameters);
    let ood_frames: String = generate_ood_frames_module(&masm_verifier_parameters);
    let public_inputs: String = generate_public_inputs(&masm_verifier_parameters);
    let constraints_eval: String =
        generate_constraints_eval_module(&masm_verifier_parameters, &encoded_circuit);
    let verifier: String = generate_verifier_module(&masm_verifier_parameters);

    Ok(MasmVerifier {
        deep_queries,
        ood_frames,
        public_inputs,
        constraints_eval,
        verifier,
    })
}

// HELPER STRUCTS
// ================================================================================================

/// Collects the modules making up the MASM STARK verifier.
#[derive(Debug, Default)]
pub struct MasmVerifier {
    constraints_eval: String,
    deep_queries: String,
    ood_frames: String,
    public_inputs: String,
    verifier: String,
}

impl MasmVerifier {
    pub fn deep_queries(&self) -> &str {
        &self.deep_queries
    }

    pub fn constants(&self) -> &str {
        &self.constraints_eval
    }

    pub fn ood_frames(&self) -> &str {
        &self.ood_frames
    }

    pub fn public_inputs(&self) -> &str {
        &self.public_inputs
    }

    pub fn verifier(&self) -> &str {
        &self.verifier
    }

    pub fn constraints_eval(&self) -> &str {
        &self.constraints_eval
    }
}

/// Parameters derived from [Air] and used in building the MASM verifier.
struct MasmVerifierParameters {
    num_auxiliary_randomness: u16,
    max_cycle_len_log: u32,

    main_trace_width: u16,
    aux_trace_width: Option<u16>,
    constraints_composition_trace_width: usize,

    variable_len_pub_inputs_sizes: BTreeMap<air_ir::Identifier, (usize, BusType)>,
    fixed_len_pub_inputs_total_size: usize,
    num_constraints: usize,
}

impl MasmVerifierParameters {
    fn from_air(air: &Air) -> Self {
        let main_trace_width = air.trace_segment_widths[0].next_multiple_of(8);
        let aux_trace_width =
            air.trace_segment_widths.get(1).map(|width| width.next_multiple_of(8));
        let num_auxiliary_randomness = air.num_random_values;

        let max_cycle_length = air.periodic_columns().map(|col| col.period()).max();
        let max_cycle_len_log = max_cycle_length.unwrap_or(1).ilog2();

        // iterate over the public inputs and build a map from the table identifier to
        // its width and its bus type
        let mut variable_len_pub_inputs_sizes = BTreeMap::new();
        for bus in air.buses.iter() {
            for pi in air.public_inputs() {
                if let air_ir::BusBoundary::PublicInputTable(public_input_table_access) =
                    bus.1.first
                {
                    if public_input_table_access.table_name == pi.name() {
                        let _ = variable_len_pub_inputs_sizes
                            .insert(pi.name(), (pi.size(), public_input_table_access.bus_type));
                    }
                }
                if let air_ir::BusBoundary::PublicInputTable(public_input_table_access) = bus.1.last
                {
                    if public_input_table_access.table_name == pi.name() {
                        let _ = variable_len_pub_inputs_sizes
                            .insert(pi.name(), (pi.size(), public_input_table_access.bus_type));
                    }
                }
            }
        }

        // compute the total number of fixed length public inputs
        let mut fixed_len_pub_inputs_total_size = 0;
        for pi in air.public_inputs() {
            if let air_ir::PublicInput::Vector { size, .. } = pi {
                fixed_len_pub_inputs_total_size += size
            }
        }

        // compute the number of constraints
        let num_constraints: usize = [0, 1]
            .iter()
            .map(|trace_id| {
                air.num_boundary_constraints(*trace_id) + air.num_integrity_constraints(*trace_id)
            })
            .sum();

        Self {
            num_auxiliary_randomness,
            max_cycle_len_log,
            main_trace_width,
            aux_trace_width,
            num_constraints,
            fixed_len_pub_inputs_total_size,
            variable_len_pub_inputs_sizes,
            constraints_composition_trace_width: NUM_CONSTRAINTS_COMPOSITION_POLYS,
        }
    }

    fn num_constraints(&self) -> usize {
        self.num_constraints
    }

    fn num_auxiliary_randomness(&self) -> u16 {
        self.num_auxiliary_randomness
    }

    fn max_cycle_len_log(&self) -> u32 {
        self.max_cycle_len_log
    }

    fn main_trace_width(&self) -> u16 {
        self.main_trace_width
    }

    fn aux_trace_width(&self) -> Option<u16> {
        self.aux_trace_width
    }

    fn constraints_composition_trace_width(&self) -> usize {
        self.constraints_composition_trace_width
    }

    fn variable_len_pub_inputs_sizes(&self) -> &BTreeMap<air_ir::Identifier, (usize, BusType)> {
        &self.variable_len_pub_inputs_sizes
    }

    fn fixed_len_pub_inputs_total_size(&self) -> usize {
        self.fixed_len_pub_inputs_total_size
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Given a map with keys section labels placeholders and values the `String` to assign to
/// the placholders, returns the resulting filled `String`.
fn generate_with_map_sections(file: &mut String, sections_map: HashMap<&'static str, String>) {
    for (section_name, code) in sections_map {
        let begin_marker = format!("# BEGIN_SECTION:{section_name}");
        let end_marker = format!("# END_SECTION:{section_name}");

        if let Some(begin_index) = file.find(&begin_marker) {
            if let Some(end_index) = file.find(&end_marker) {
                let end_position = end_index + end_marker.len();

                // find the line start to preserve indentation
                let line_start = file[..begin_index].rfind('\n').map(|i| i + 1).unwrap_or(0);

                // extract indentation from the line containing the end marker
                let indentation = &file[line_start..begin_index];
                let indent_str =
                    indentation.chars().take_while(|&c| c == ' ' || c == '\t').collect::<String>();

                let before = &file[..line_start];
                let after_line_end = file[end_position..]
                    .find('\n')
                    .map(|i| end_position + i + 1)
                    .unwrap_or(file.len());
                let after = &file[after_line_end..];

                // indent each line of the code section to be inserted
                let indented_code = if code.trim().is_empty() {
                    String::new()
                } else {
                    code.lines()
                        .map(|line| {
                            if line.trim().is_empty() {
                                String::new()
                            } else {
                                format!("{indent_str}{line}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                // assemble the resulting file
                *file = if indented_code.is_empty() {
                    format!("{before}{after}")
                } else {
                    format!("{before}{indented_code}\n{after}")
                };
            }
        }
    }
}

/// Given a section placeholder identifier and a `String` value to assign to the placholder,
/// returns the resulting filled `String`.
fn add_section(file: &mut String, section_placeholder_id: &'static str, section: &str) {
    let mut sections_map = HashMap::new();
    sections_map.insert(section_placeholder_id, section.to_string());
    generate_with_map_sections(file, sections_map)
}
