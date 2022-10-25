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
fn error_no_public_input() {
    let source = "
    public_inputs:
    ";
    assert!(build_parse_test!(source).parse().is_err());
}
