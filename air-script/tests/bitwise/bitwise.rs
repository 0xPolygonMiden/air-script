use winter_air::{Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions as WinterProofOptions, TransitionConstraintDegree, TraceInfo};
use winter_math::fields::f64::BaseElement as Felt;
use winter_math::{ExtensionOf, FieldElement};
use winter_utils::collections::Vec;
use winter_utils::{ByteWriter, Serializable};

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

pub struct BitwiseAir {
    context: AirContext<Felt>,
    stack_inputs: [Felt; 16],
}

impl BitwiseAir {
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }
}

impl Air for BitwiseAir {
    type BaseField = Felt;
    type PublicInputs = PublicInputs;

    fn context(&self) -> &AirContext<Felt> {
        &self.context
    }

    fn new(trace_info: TraceInfo, public_inputs: PublicInputs, options: WinterProofOptions) -> Self {
        let main_degrees = vec![TransitionConstraintDegree::new(2), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(2), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::new(3)];
        let aux_degrees = vec![];
        let num_main_assertions = 1;
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
        vec![vec![Felt::new(1), Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(0)], vec![Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(0)]]
    }

    fn get_assertions(&self) -> Vec<Assertion<Felt>> {
        let mut result = Vec::new();
        result.push(Assertion::single(13, 0, Felt::new(0)));
        result
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Felt>>(&self, aux_rand_elements: &AuxTraceRandElements<E>) -> Vec<Assertion<E>> {
        let mut result = Vec::new();
        result
    }

    fn evaluate_transition<E: FieldElement<BaseField = Felt>>(&self, frame: &EvaluationFrame<E>, periodic_values: &[E], result: &mut [E]) {
        let current = frame.current();
        let next = frame.next();
        result[0] = (current[0]).exp(E::PositiveInteger::from(2_u64)) - (current[0]) - (E::from(0_u64));
        result[1] = (periodic_values[1]) * (next[0] - (current[0])) - (E::from(0_u64));
        result[2] = (current[3]).exp(E::PositiveInteger::from(2_u64)) - (current[3]) - (E::from(0_u64));
        result[3] = (current[4]).exp(E::PositiveInteger::from(2_u64)) - (current[4]) - (E::from(0_u64));
        result[4] = (current[5]).exp(E::PositiveInteger::from(2_u64)) - (current[5]) - (E::from(0_u64));
        result[5] = (current[6]).exp(E::PositiveInteger::from(2_u64)) - (current[6]) - (E::from(0_u64));
        result[6] = (current[7]).exp(E::PositiveInteger::from(2_u64)) - (current[7]) - (E::from(0_u64));
        result[7] = (current[8]).exp(E::PositiveInteger::from(2_u64)) - (current[8]) - (E::from(0_u64));
        result[8] = (current[9]).exp(E::PositiveInteger::from(2_u64)) - (current[9]) - (E::from(0_u64));
        result[9] = (current[10]).exp(E::PositiveInteger::from(2_u64)) - (current[10]) - (E::from(0_u64));
        result[10] = (periodic_values[0]) * (current[1] - (((E::from(2_u64)).exp(E::PositiveInteger::from(0_u64))) * (current[3]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(1_u64))) * (current[4]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(2_u64))) * (current[5]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(3_u64))) * (current[6]))) - (E::from(0_u64));
        result[11] = (periodic_values[0]) * (current[2] - (((E::from(2_u64)).exp(E::PositiveInteger::from(0_u64))) * (current[7]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(1_u64))) * (current[8]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(2_u64))) * (current[9]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(3_u64))) * (current[10]))) - (E::from(0_u64));
        result[12] = (periodic_values[1]) * (next[1] - ((current[1]) * (E::from(16_u64)) + ((E::from(2_u64)).exp(E::PositiveInteger::from(0_u64))) * (current[3]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(1_u64))) * (current[4]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(2_u64))) * (current[5]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(3_u64))) * (current[6]))) - (E::from(0_u64));
        result[13] = (periodic_values[1]) * (next[2] - ((current[2]) * (E::from(16_u64)) + ((E::from(2_u64)).exp(E::PositiveInteger::from(0_u64))) * (current[7]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(1_u64))) * (current[8]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(2_u64))) * (current[9]) + ((E::from(2_u64)).exp(E::PositiveInteger::from(3_u64))) * (current[10]))) - (E::from(0_u64));
        result[14] = (periodic_values[0]) * (current[11]) - (E::from(0_u64));
        result[15] = (periodic_values[1]) * (current[12] - (next[11])) - (E::from(0_u64));
        result[16] = (E::from(1_u64) - (current[0])) * (current[12] - ((current[11]) * (E::from(16_u64)) + (((E::from(2_u64)).exp(E::PositiveInteger::from(0_u64))) * (current[3])) * (current[7]) + (((E::from(2_u64)).exp(E::PositiveInteger::from(1_u64))) * (current[4])) * (current[8]) + (((E::from(2_u64)).exp(E::PositiveInteger::from(2_u64))) * (current[5])) * (current[9]) + (((E::from(2_u64)).exp(E::PositiveInteger::from(3_u64))) * (current[6])) * (current[10]))) + (current[0]) * (current[12] - ((current[11]) * (E::from(16_u64)) + ((E::from(2_u64)).exp(E::PositiveInteger::from(0_u64))) * (current[3] + current[7] - (((E::from(2_u64)) * (current[3])) * (current[7]))) + ((E::from(2_u64)).exp(E::PositiveInteger::from(1_u64))) * (current[4] + current[8] - (((E::from(2_u64)) * (current[4])) * (current[8]))) + ((E::from(2_u64)).exp(E::PositiveInteger::from(2_u64))) * (current[5] + current[9] - (((E::from(2_u64)) * (current[5])) * (current[9]))) + ((E::from(2_u64)).exp(E::PositiveInteger::from(3_u64))) * (current[6] + current[10] - (((E::from(2_u64)) * (current[6])) * (current[10]))))) - (E::from(0_u64));
    }

    fn evaluate_aux_transition<F, E>(&self, main_frame: &EvaluationFrame<F>, aux_frame: &EvaluationFrame<E>, _periodic_values: &[F], aux_rand_elements: &AuxTraceRandElements<E>, result: &mut [E])
    where F: FieldElement<BaseField = Felt>,
          E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
    {
        let current = aux_frame.current();
        let next = aux_frame.next();
    }
}