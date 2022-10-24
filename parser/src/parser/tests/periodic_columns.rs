use super::{build_parse_test, Identifier, PeriodicColumn, Source, SourceSection};

#[test]
fn periodic_columns() {
    let source = "
periodic_columns:
    k0: [1, 0, 0, 0]
    k1: [0, 0, 0, 0, 0, 0, 0, 1]";
    let expected = Source(vec![SourceSection::PeriodicColumns(vec![
        PeriodicColumn::new(Identifier("k0".to_string()), vec![1, 0, 0, 0]),
        PeriodicColumn::new(Identifier("k1".to_string()), vec![0, 0, 0, 0, 0, 0, 0, 1]),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn empty_periodic_columns() {
    let source = "
periodic_columns:";
    let expected = Source(vec![SourceSection::PeriodicColumns(vec![])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn error_periodic_columns_length() {
    let source = "
periodic_columns:
    k0: [1, 0, 0]";
    let expected = Source(vec![SourceSection::PeriodicColumns(vec![
        PeriodicColumn::new(Identifier("k0".to_string()), vec![1, 0, 0]),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}
