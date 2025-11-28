use std::collections::BTreeMap;

use miden_air::{Felt, FieldElement, trace::main_trace::MainTrace};
use miden_processor::{
    ColMatrix, Kernel, PrecompileTranscriptState, QuadExtension,
    chiplets::{AceHints, AuxTraceBuilder as ChipletsAuxTraceBuilder},
    decoder::AuxTraceBuilder as DecoderAuxTraceBuilder,
    range::AuxTraceBuilder as RangeAuxTraceBuilder,
    stack::AuxTraceBuilder as StackAuxTraceBuilder,
};
use p3_field::{ExtensionField, Field, PrimeField64};
use p3_matrix::Matrix;
use p3_miden_air::RowMajorMatrix;

pub enum MidenModule {
    Chiplets,
    Decoder,
    Stack,
    Range,
}

impl MidenModule {
    fn build_aux_columns<EF: FieldElement<BaseField = Felt>>(
        &self,
        main_trace: &MainTrace,
        rand_elements: &[EF],
    ) -> Vec<Vec<EF>> {
        match self {
            MidenModule::Chiplets => {
                let kernel = Kernel::new(&[]).unwrap();
                let ace_hints = AceHints::new(0, vec![]);
                let final_transcript_state = PrecompileTranscriptState::default();
                let aux_trace_builder =
                    ChipletsAuxTraceBuilder::new(kernel, ace_hints, final_transcript_state);
                aux_trace_builder.build_aux_columns(main_trace, rand_elements).to_vec()
            },
            MidenModule::Decoder => {
                let aux_trace_builder = DecoderAuxTraceBuilder {};
                aux_trace_builder.build_aux_columns(main_trace, rand_elements)
            },
            MidenModule::Stack => {
                let aux_trace_builder = StackAuxTraceBuilder {};
                aux_trace_builder.build_aux_columns(main_trace, rand_elements)
            },
            MidenModule::Range => {
                let lookup_values = vec![];
                let cycle_lookups = BTreeMap::new();
                let values_start = 0;
                let aux_trace_builder =
                    RangeAuxTraceBuilder::new(lookup_values, cycle_lookups, values_start);
                aux_trace_builder.build_aux_columns(main_trace, rand_elements)
            },
        }
    }
}

/// Builds the Miden VM auxiliary trace using the provided main trace and challenges.
pub fn build_aux_trace_with_miden_vm<F, EF>(
    main: &RowMajorMatrix<F>,
    challenges: &[EF],
    module: MidenModule,
) -> RowMajorMatrix<EF>
where
    F: Field + PrimeField64,
    EF: ExtensionField<F>,
{
    // Convert main trace to Miden format
    let mut main_trace_vec_vec = Vec::new();
    for row_index in 0..main.height() {
        let row_f: Vec<F> = main.row_slice(row_index).unwrap().to_vec();
        let row_felt =
            row_f.into_iter().map(|x| Felt::new(x.as_canonical_u64())).collect::<Vec<_>>();
        main_trace_vec_vec.push(row_felt);
    }
    let transposed_main_trace_vec_vec: Vec<Vec<_>> = (0..main_trace_vec_vec[0].len())
        .map(|i| main_trace_vec_vec.iter().map(|row| row[i]).collect())
        .collect();
    let col_matrix = ColMatrix::new(transposed_main_trace_vec_vec);
    let last_program_row = main.height().into();
    let main_trace = MainTrace::new(col_matrix, last_program_row);

    // Convert challenges to Miden format
    let mut rand_elements = Vec::new();
    for r in challenges {
        let coeffs: Vec<Felt> = r
            .as_basis_coefficients_slice()
            .iter()
            .map(|x| Felt::new(x.as_canonical_u64()))
            .collect();
        let r_fe: &[QuadExtension<Felt>] = FieldElement::slice_from_base_elements(&coeffs);
        rand_elements.push(r_fe[0]);
    }

    // Build aux trace using Miden VM AuxTraceBuilder
    let aux_trace_miden = module.build_aux_columns(&main_trace, &rand_elements);
    let aux_width = aux_trace_miden.len();

    // Convert aux trace back to RowMajorMatrix<EF>
    let num_rows = main.height();
    let trace_length = num_rows * aux_width;
    let long_trace = EF::zero_vec(trace_length);
    let mut aux_trace = RowMajorMatrix::new(long_trace, aux_width);

    for j in 0..aux_width {
        let col = aux_trace_miden.get(j).unwrap();
        for i in 0..num_rows {
            let value_felt = col[i];
            let coeffs_f: Vec<F> = value_felt
                .to_base_elements()
                .iter()
                .map(|x| F::from_canonical_checked(x.as_int()).unwrap())
                .collect();
            let value_ef = EF::from_basis_coefficients_iter(coeffs_f.iter().cloned()).unwrap();
            aux_trace.row_mut(i)[j] = value_ef;
        }
    }

    aux_trace
}
