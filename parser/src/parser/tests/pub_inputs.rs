use super::{build_parse_test, Identifier, PublicInput, Source, SourceSection};

// PUBLIC INPUTS
// ================================================================================================

#[test]
fn public_inputs() {
    let source = "
    public_inputs:
        program_hash: [4]
        stack_inputs: [16]";
    let expected = Source(vec![SourceSection::PublicInputs(vec![
        PublicInput::new(Identifier("program_hash".to_string()), 4),
        PublicInput::new(Identifier("stack_inputs".to_string()), 16),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn empty_public_inputs() {
    let source = "
    public_inputs:";
    let expected = Source(vec![SourceSection::PublicInputs(vec![])]);
    build_parse_test!(source).expect_ast(expected);
}
