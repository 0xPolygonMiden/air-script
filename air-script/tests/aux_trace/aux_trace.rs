use winter_air::{Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions as WinterProofOptions, TransitionConstraintDegree, TraceInfo};
use winter_math::{fields, ExtensionOf, FieldElement};
use winter_utils::{collections, ByteWriter, Serializable};

pub struct PublicInputs {
    stack_inputs: [Felt; 16],
}

impl PublicInputs {
    pub fn new(stack_inputs: [Felt; 16]) -> Self {
        Self { stack_inputs }
    }
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.stack_inputs.as_slice());
    }
}

pub struct AuxiliaryAir {
    context: AirContext<Felt>,
    stack_inputs: [Felt; 16],
}

impl AuxiliaryAir {
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }
}

impl Air for AuxiliaryAir {
    type BaseField = Felt;
    type PublicInputs = PublicInputs;

    fn context(&self) -> &AirContext<Felt> {
        &self.context
    }

    fn new(trace_info: TraceInfo, public_inputs: PublicInputs, options: WinterProofOptions) -> Self {
        let main_degrees = vec![TransitionConstraintDegree::new(1), TransitionConstraintDegree::new(1), TransitionConstraintDegree::new(1)];
        let aux_degrees = Vec::new();
        let num_main_assertions = 2;
        let num_aux_assertions = 0;

        let context = AirContext::new_multi_segment(
            trace_info,
            main_degrees,
            aux_degrees,
            num_main_assertions,
            num_aux_assertions,
            options,
        )
        .set_num_transition_exemptions(2);
        Self { context, stack_inputs: public_inputs.stack_inputs }
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Felt>> {
        vec![]
    }

    fn get_assertions(&self) -> Vec<Assertion<Felt>> {
        let mut result = Vec::new();
        result.push(Assertion::single(0, 0, Felt::new(1)));
        result.push(Assertion::single(1, 0, Felt::new(1)));
        result
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Felt>>(&self, aux_rand_elements: &AuxTraceRandElements<E>) -> Vec<Assertion<E>> {
        let mut result = Vec::new();
        result.push(Assertion::single(0, 0, E::from(1_u64)));
        result.push(Assertion::single(1, 0, aux_rand_elements.get_segment_elements(0)[0]));
        let last_step = self.last_step();
        result.push(Assertion::single(0, last_step, E::from(1_u64)));
        result.push(Assertion::single(1, last_step, E::from(1_u64)));
        result
    }

    fn evaluate_transition<E: FieldElement<BaseField = Felt>>(&self, frame: &EvaluationFrame<E>, periodic_values: &[E], result: &mut [E]) {
        let current = frame.current();
        let next = frame.next();
        result[0] = next[0] - (current[1] + current[2]);
        result[1] = next[1] - (current[2] + next[0]);
        result[2] = current[2] - (current[0] + current[1]);
    }

    fn evaluate_aux_transition<F, E>(&self, main_frame: &EvaluationFrame<F>, aux_frame: &EvaluationFrame<E>, _periodic_values: &[F], aux_rand_elements: &AuxTraceRandElements<E>, result: &mut [E])
    where F: FieldElement<BaseField = Felt>,
          E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
    {
        let current = aux_frame.current();
        let next = aux_frame.next();
        result[0] = next[0] - ((current[0]) * (current[0] + aux_rand_elements.get_segment_elements(0)[0] + current[1] + aux_rand_elements.get_segment_elements(0)[1]));
        result[1] = current[1] - ((next[1]) * (current[2] + aux_rand_elements.get_segment_elements(0)[0]));
    }
}