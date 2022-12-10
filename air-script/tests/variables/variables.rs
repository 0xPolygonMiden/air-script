use winter_air::{Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions as WinterProofOptions, TransitionConstraintDegree, TraceInfo};
use winter_math::fields::f64::BaseElement as Felt;
use winter_math::{ExtensionOf, FieldElement};
use winter_utils::collections::Vec;
use winter_utils::{ByteWriter, Serializable};

pub struct PublicInputs {
    stack_inputs: [Felt; 16],
    stack_outputs: [Felt; 16],
}

impl PublicInputs {
    pub fn new(stack_inputs: [Felt; 16], stack_outputs: [Felt; 16]) -> Self {
        Self { stack_inputs, stack_outputs }
    }
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.stack_inputs.as_slice());
        target.write(self.stack_outputs.as_slice());
    }
}

pub struct VariablesAir {
    context: AirContext<Felt>,
    stack_inputs: [Felt; 16],
    stack_outputs: [Felt; 16],
}

impl VariablesAir {
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }
}

impl Air for VariablesAir {
    type BaseField = Felt;
    type PublicInputs = PublicInputs;

    fn context(&self) -> &AirContext<Felt> {
        &self.context
    }

    fn new(trace_info: TraceInfo, public_inputs: PublicInputs, options: WinterProofOptions) -> Self {
        let main_degrees = vec![TransitionConstraintDegree::new(2), TransitionConstraintDegree::with_cycles(1, vec![8]), TransitionConstraintDegree::new(2), TransitionConstraintDegree::new(3)];
        let aux_degrees = vec![TransitionConstraintDegree::new(2)];
        let num_main_assertions = 6;
        let num_aux_assertions = 1;

        let context = AirContext::new_multi_segment(
            trace_info,
            main_degrees,
            aux_degrees,
            num_main_assertions,
            num_aux_assertions,
            options,
        )
        .set_num_transition_exemptions(2);
        Self { context, stack_inputs: public_inputs.stack_inputs, stack_outputs: public_inputs.stack_outputs }
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Felt>> {
        vec![vec![Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(1), Felt::new(0)]]
    }

    fn get_assertions(&self) -> Vec<Assertion<Felt>> {
        let x = Felt::new(1);
        let y = [x, (Felt::new(4)) - (Felt::new(2))];
        let z = [[x, Felt::new(3)], [(Felt::new(4)) - (Felt::new(2)), (Felt::new(8)) + (Felt::new(8))]];
        let mut result = Vec::new();
        result.push(Assertion::single(1, 0, self.stack_inputs[0]));
        result.push(Assertion::single(2, 0, x));
        result.push(Assertion::single(3, 0, y[0]));
        let last_step = self.last_step();
        result.push(Assertion::single(1, last_step, self.stack_outputs[0]));
        result.push(Assertion::single(2, last_step, z[0][0]));
        result.push(Assertion::single(3, last_step, self.stack_outputs[2]));
        result
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Felt>>(&self, aux_rand_elements: &AuxTraceRandElements<E>) -> Vec<Assertion<E>> {
        let x = E::from(1_u64);
        let y = [E::from(x), (E::from(4_u64)) - (E::from(2_u64))];
        let z = [[E::from(x), E::from(3_u64)], [(E::from(4_u64)) - (E::from(2_u64)), (E::from(8_u64)) + (E::from(8_u64))]];
        let mut result = Vec::new();
        result.push(Assertion::single(0, 0, E::from(1_u64)));
        result
    }

    fn evaluate_transition<E: FieldElement<BaseField = Felt>>(&self, frame: &EvaluationFrame<E>, periodic_values: &[E], result: &mut [E]) {
        let m = E::from(0_u64);
        let n = [(E::from(2_u64)) * (E::from(3_u64)), current[0]];
        let o = [[next[0], E::from(3_u64)], [E::from(4_u64) - (E::from(2_u64)), E::from(8_u64) + E::from(8_u64)]];
        let current = frame.current();
        let next = frame.next();
        result[0] = (current[0]).exp(E::PositiveInteger::from(2_u64)) - (current[0]);
        result[1] = (periodic_values[0]) * (next[0] - (current[0])) - (m);
        result[2] = (E::from(1_u64) - (current[0])) * (current[3] - (current[1]) + current[2]) - (n[0] - (n[1]));
        result[3] = (current[0]) * (current[3] - ((current[1]) * (current[2]))) - (o[0][0] - (o[0][1]) - (o[1][0]));
    }

    fn evaluate_aux_transition<F, E>(&self, main_frame: &EvaluationFrame<F>, aux_frame: &EvaluationFrame<E>, _periodic_values: &[F], aux_rand_elements: &AuxTraceRandElements<E>, result: &mut [E])
    where F: FieldElement<BaseField = Felt>,
          E: FieldElement<BaseField = Felt> + ExtensionOf<F>,
    {
        let m = E::from(0_u64);
        let n = [(E::from(2_u64)) * (E::from(3_u64)), current[0]];
        let o = [[next[0], E::from(3_u64)], [E::from(4_u64) - (E::from(2_u64)), E::from(8_u64) + E::from(8_u64)]];
        let current = aux_frame.current();
        let next = aux_frame.next();
        result[0] = next[0] - ((current[0]) * (current[3] + aux_rand_elements.get_segment_elements(0)[0]));
    }
}