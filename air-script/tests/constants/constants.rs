use winter_air::{Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions as WinterProofOptions, TransitionConstraintDegree, TraceInfo};
use winter_math::fields::f64::BaseElement as Felt;
use winter_math::{ExtensionOf, FieldElement};
use winter_utils::collections::Vec;
use winter_utils::{ByteWriter, Serializable};

const A: Felt = Felt::new(1);
const B: [Felt; 2] = [Felt::new(0), Felt::new(1)];
const C: [[Felt; 2]; 2] = [[Felt::new(1), Felt::new(2)], [Felt::new(2), Felt::new(0)]];

pub struct PublicInputs {
    program_hash: [Felt; 4],
    stack_inputs: [Felt; 4],
    stack_outputs: [Felt; 20],
    overflow_addrs: [Felt; 4],
}

impl PublicInputs {
    pub fn new(program_hash: [Felt; 4], stack_inputs: [Felt; 4], stack_outputs: [Felt; 20], overflow_addrs: [Felt; 4]) -> Self {
        Self { program_hash, stack_inputs, stack_outputs, overflow_addrs }
    }
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.program_hash.as_slice());
        target.write(self.stack_inputs.as_slice());
        target.write(self.stack_outputs.as_slice());
        target.write(self.overflow_addrs.as_slice());
    }
}

pub struct ConstantsAir {
    context: AirContext<Felt>,
    program_hash: [Felt; 4],
    stack_inputs: [Felt; 4],
    stack_outputs: [Felt; 20],
    overflow_addrs: [Felt; 4],
}

impl ConstantsAir {
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }
}

impl Air for ConstantsAir {
    type BaseField = Felt;
    type PublicInputs = PublicInputs;

    fn context(&self) -> &AirContext<Felt> {
        &self.context
    }

    fn new(trace_info: TraceInfo, public_inputs: PublicInputs, options: WinterProofOptions) -> Self {
        let main_degrees = vec![TransitionConstraintDegree::new(1), TransitionConstraintDegree::new(1), TransitionConstraintDegree::new(1)];
        let aux_degrees = vec![TransitionConstraintDegree::new(1), TransitionConstraintDegree::new(1)];
        let num_main_assertions = 4;
        let num_aux_assertions = 2;

        let context = AirContext::new_multi_segment(
            trace_info,
            main_degrees,
            aux_degrees,
            num_main_assertions,
            num_aux_assertions,
            options,
        )
        .set_num_transition_exemptions(2);
        Self { context, program_hash: public_inputs.program_hash, stack_inputs: public_inputs.stack_inputs, stack_outputs: public_inputs.stack_outputs, overflow_addrs: public_inputs.overflow_addrs }
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Felt>> {
        vec![]
    }

    fn get_assertions(&self) -> Vec<Assertion<Felt>> {
        let mut result = Vec::new();
        result.push(Assertion::single(0, 0, A));
        result.push(Assertion::single(1, 0, A + B[0] * C[0][1]));
        result.push(Assertion::single(2, 0, (B[0] - C[1][1]) * A));
        result.push(Assertion::single(3, 0, A + B[0] - B[1] + C[0][0] - C[0][1] + C[1][0] - C[1][1]));
        result
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Felt>>(&self, aux_rand_elements: &AuxTraceRandElements<E>) -> Vec<Assertion<E>> {
        let mut result = Vec::new();
        result.push(Assertion::single(0, 0, E::from(A) + E::from(B[0]) * E::from(C[0][1])));
        result.push(Assertion::single(0, self.last_step(), E::from(A) - E::from(B[1]) * E::from(C[0][0])));
        result
    }

    fn evaluate_transition<E: FieldElement<BaseField = Felt>>(&self, frame: &EvaluationFrame<E>, periodic_values: &[E], result: &mut [E]) {
        let main_current = frame.current();
        let main_next = frame.next();
        result[0] = main_next[0] - (main_current[0] + E::from(A));
        result[1] = main_next[1] - E::from(B[0]) * main_current[1];
        result[2] = main_next[2] - (E::from(C[0][0]) + E::from(B[0])) * main_current[2];
    }

    fn evaluate_aux_transition<F, E>(&self, main_frame: &EvaluationFrame<F>, aux_frame: &EvaluationFrame<E>, _periodic_values: &[F], aux_rand_elements: &AuxTraceRandElements<E>, result: &mut [E])
    where F: FieldElement<BaseField = Felt>,
          E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
    {
        let main_current = main_frame.current();
        let main_next = main_frame.next();
        let aux_current = aux_frame.current();
        let aux_next = aux_frame.next();
        result[0] = aux_next[0] - (aux_current[0] + E::from(A) + E::from(B[0]) * E::from(C[0][1]));
        result[1] = aux_current[0] - (E::from(A) + E::from(B[1]) * E::from(C[1][1]));
    }
}