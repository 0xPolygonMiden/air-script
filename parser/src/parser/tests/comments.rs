use super::{build_parse_test, Identifier, Source, SourceSection::*};

// COMMENTS
// ================================================================================================

#[test]
fn simple_comment() {
    let source = "# Simple Comment";
    let expected = Source(vec![]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn inline_comment() {
    let source = "def SystemAir # Simple Comment";
    let expected = Source(vec![AirDef(Identifier("SystemAir".to_string()))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multiline_comments() {
    let source = "# Comment line 1
    # Comment line 2
    def SystemAir";
    let expected = Source(vec![AirDef(Identifier("SystemAir".to_string()))]);
    build_parse_test!(source).expect_ast(expected);
}
