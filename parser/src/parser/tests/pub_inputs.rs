use super::{Identifier, ParseTest, PublicInput, Source, SourceSection::*};

// PUBLIC INPUTS
// ================================================================================================

#[test]
fn public_inputs() {
    let source = "
    public_inputs:
        program_hash: [4]
        stack_inputs: [16]";
    let expected = Source(vec![PublicInputs(vec![
        PublicInput::new(Identifier("program_hash".to_string()), 4),
        PublicInput::new(Identifier("stack_inputs".to_string()), 16),
    ])]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn error_no_public_input() {
    let source = "
    public_inputs:
    ";
    assert!(ParseTest::new().parse(source).is_err());
}
