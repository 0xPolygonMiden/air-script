use super::{Identifier, ParseTest, PeriodicColumn, Source, SourceSection::*};

#[test]
fn periodic_columns() {
    let source = "
periodic_columns:
    k0: [1, 0, 0, 0]
    k1: [0, 0, 0, 0, 0, 0, 0, 1]";
    let expected = Source(vec![PeriodicColumns(vec![
        PeriodicColumn::new(Identifier("k0".to_string()), vec![1, 0, 0, 0]),
        PeriodicColumn::new(Identifier("k1".to_string()), vec![0, 0, 0, 0, 0, 0, 0, 1]),
    ])]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn empty_periodic_columns() {
    let source = "
periodic_columns:";
    let expected = Source(vec![PeriodicColumns(vec![])]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn error_periodic_columns_length() {
    let source = "
periodic_columns:
    k0: [1, 0, 0]";
    let expected = Source(vec![PeriodicColumns(vec![PeriodicColumn::new(
        Identifier("k0".to_string()),
        vec![1, 0, 0],
    )])]);
    ParseTest::new().expect_ast(source, expected);
}
