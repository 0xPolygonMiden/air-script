use super::{
    Constant, ConstantType, Identifier, IdentifierType, PeriodicColumn, PublicInput, SymbolTable,
    TraceColumn,
};

// TODO: TEST ERRORS

// --- TEST MUTATORS ------------------------------------------------------------------------------

#[test]
fn test_add_constants() {
    let mut sym_table = SymbolTable::default();

    // the constants to test.
    let constants = vec![
        ("a", ConstantType::Scalar(10)),
        ("b", ConstantType::Vector(vec![1, 2, 3])),
        ("c", ConstantType::Matrix(vec![vec![1, 2], vec![3, 4]])),
    ];

    // insert constants.
    let res = sym_table.insert_constants(&constants.build_ast_type());

    // constants were added successfully.
    assert!(res.is_ok());

    // the symbol table has the expected identifier information.
    constants.check_expected(&sym_table);
}

/// Test inserting columns of the main trace into the symbol table.
#[test]
fn test_add_main_trace() {
    let mut sym_table = SymbolTable::default();

    // the trace to test.
    let cols = vec!["a", "b", "third"];
    let segment = 0;
    let trace = (segment, cols);

    // insert main trace columns.
    let res = sym_table.insert_trace_columns(segment, &trace.build_ast_type());

    // trace columns were added successfully.
    assert!(res.is_ok());

    // the symbol table has the expected identifier information.
    trace.check_expected(&sym_table);
}

/// Test inserting columns of the aux trace into the symbol table.
#[test]
fn test_add_aux_trace() {
    let mut sym_table = SymbolTable::default();

    // the trace to test.
    let cols = vec!["a", "b", "third"];
    let segment = 1;
    let trace = (segment, cols);

    // insert aux trace columns.
    let res = sym_table.insert_trace_columns(segment, &trace.build_ast_type());

    // trace columns were added successfully.
    assert!(res.is_ok());

    // the symbol table has the expected identifier information.
    trace.check_expected(&sym_table);
}

/// Test inserting public inputs into the symbol table.
#[test]
fn test_add_pub_inputs() {
    let mut sym_table = SymbolTable::default();

    // the public input declarations to test.
    let decls = vec![("a", 4_u64), ("b", 8), ("third", 16)];

    // insert public inputs.
    let res = sym_table.insert_public_inputs(&decls.build_ast_type());

    // public inputs were added successfully.
    assert!(res.is_ok());

    // the symbol table has the expected public inputs and identifier information
    decls.check_expected(&sym_table);
}

/// Test inserting periodic columns into the symbol table.
#[test]
fn test_add_periodic_columns() {
    let mut sym_table = SymbolTable::default();

    // the column declarations to test.
    let decls = vec![
        ("a", vec![1_u64, 0]),
        ("b", vec![1, 0, 0, 0]),
        ("third", vec![1, 1, 1, 0]),
    ];

    // insert periodic columns
    let res = sym_table.insert_periodic_columns(&decls.build_ast_type());

    // periodic columns were added successfully.
    assert!(res.is_ok());

    // the symbol table has the expected periodic columns and identifier information
    decls.check_expected(&sym_table);
}

// HELPERS
// ================================================================================================

/// Utility trait for testing symbol table processing of AST types. It takes the AST type as the
/// generic type and is implemented for an input format containing the required data.
trait SymbolTest<T> {
    fn build_ast_type(&self) -> Vec<T>;
    fn check_expected(self, sym_table: &SymbolTable);
}

/// Test symbol table processing of Constants.
impl SymbolTest<Constant> for Vec<(&str, ConstantType)> {
    fn build_ast_type(&self) -> Vec<Constant> {
        self.iter()
            .map(|(name, value)| Constant::new(Identifier(name.to_string()), value.clone()))
            .collect()
    }

    fn check_expected(self, sym_table: &SymbolTable) {
        let expected_const = self
            .into_iter()
            .map(|(name, value)| {
                // check identifiers
                assert_eq!(
                    sym_table.identifiers.get(name),
                    Some(&IdentifierType::Constant(value.clone()))
                );

                Constant::new(Identifier(name.to_string()), value)
            })
            .collect::<Vec<_>>();

        // check symbol table constants
        assert_eq!(sym_table.constants, expected_const);
    }
}

/// Test symbol table processing of a trace segment, which is represented in the AST as a vector of
/// Identifier.
impl SymbolTest<Identifier> for (u8, Vec<&str>) {
    fn build_ast_type(&self) -> Vec<Identifier> {
        self.1
            .iter()
            .map(|name| Identifier(name.to_string()))
            .collect()
    }

    fn check_expected(self, sym_table: &SymbolTable) {
        for (idx, &col_name) in self.1.iter().enumerate() {
            assert_eq!(
                sym_table.identifiers.get(col_name),
                Some(&IdentifierType::TraceColumn(TraceColumn::new(self.0, idx)))
            );
        }
    }
}

/// Test symbol table processing of PublicInputs.
impl SymbolTest<PublicInput> for Vec<(&str, u64)> {
    fn build_ast_type(&self) -> Vec<PublicInput> {
        self.iter()
            .map(|&(name, size)| PublicInput::new(Identifier(name.to_string()), size))
            .collect()
    }

    fn check_expected(self, sym_table: &SymbolTable) {
        let expected_pub_inputs = self
            .into_iter()
            .map(|(name, size)| {
                // check identifiers
                assert_eq!(
                    sym_table.identifiers.get(name),
                    Some(&IdentifierType::PublicInput(size as usize))
                );

                (name.to_string(), size as usize)
            })
            .collect::<Vec<_>>();

        // check symbol table public inputs
        assert_eq!(sym_table.public_inputs, expected_pub_inputs);
    }
}

/// Test symbol table processing of PeriodicColumns
impl SymbolTest<PeriodicColumn> for Vec<(&str, Vec<u64>)> {
    fn build_ast_type(&self) -> Vec<PeriodicColumn> {
        self.iter()
            .map(|(name, values)| {
                PeriodicColumn::new(Identifier(name.to_string()), values.to_vec())
            })
            .collect()
    }

    fn check_expected(self, sym_table: &SymbolTable) {
        let values = self
            .into_iter()
            .enumerate()
            .map(|(idx, (name, values))| {
                // check identifiers
                assert_eq!(
                    sym_table.identifiers.get(name),
                    Some(&IdentifierType::PeriodicColumn(idx, values.len()))
                );

                values
            })
            .collect::<Vec<_>>();

        // check symbol table periodic columns
        assert_eq!(sym_table.periodic_columns, values);
    }
}
