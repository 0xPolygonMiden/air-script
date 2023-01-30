use super::{
    build_parse_test, Error, Identifier, ParseError, RandBinding, RandomValues, Source,
    SourceSection::*,
};

// RANDOM VALUES
// ================================================================================================

#[test]
fn random_values_fixed_list_default_name() {
    let source = "
    random_values:
        rand: [15]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        Identifier("rand".to_string()),
        15,
        vec![],
    ))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn random_values_ident_vector() {
    let source = "
    random_values:
        rand: [a, b[12], c]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        Identifier("rand".to_string()),
        14,
        vec![
            RandBinding::new(Identifier("a".to_string()), 1),
            RandBinding::new(Identifier("b".to_string()), 12),
            RandBinding::new(Identifier("c".to_string()), 1),
        ],
    ))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn random_values_ident_vector_custom_name() {
    let source = "
    random_values:
        alphas: [a, b[12], c]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        Identifier("alphas".to_string()),
        14,
        vec![
            RandBinding::new(Identifier("a".to_string()), 1),
            RandBinding::new(Identifier("b".to_string()), 12),
            RandBinding::new(Identifier("c".to_string()), 1),
        ],
    ))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn random_values_empty_list_error() {
    let source = "
    random_values:
        rand: []";
    let error = Error::ParseError(ParseError::InvalidRandomValues(
        "Random Values section cannot be empty".to_string(),
    ));
    build_parse_test!(source).expect_error(error)
}

#[test]
fn random_values_multiple_declaration_error() {
    let source = "
    random_values:
        rand: [12]
        alphas: [a, b[2]]
        betas: [c[12], d]";
    let error = Error::ParseError(ParseError::InvalidRandomValues(
        "No more than one set of random values can be declared".to_string(),
    ));
    build_parse_test!(source).expect_error(error)
}
