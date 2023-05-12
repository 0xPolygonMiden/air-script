use super::{Identifier, ParseTest, Source, SourceSection::*};

// COMMENTS
// ================================================================================================

#[test]
fn simple_comment() {
    let source = "# Simple Comment";
    let expected = Source(vec![]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn inline_comment() {
    let source = "def SystemAir # Simple Comment";
    let expected = Source(vec![AirDef(Identifier("SystemAir".to_string()))]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn multiline_comments() {
    let source = "# Comment line 1
    # Comment line 2
    def SystemAir";
    let expected = Source(vec![AirDef(Identifier("SystemAir".to_string()))]);
    ParseTest::new().expect_ast(source, expected);
}
