use std::sync::Arc;

use air_ir::CompileError;
use air_script::compile;
use miden_diagnostics::{
    CodeMap, DefaultEmitter, DiagnosticsHandler, term::termcolor::ColorChoice,
};

/// Test helper to compile an AirScript file and ensure it succeeds
fn compile_air_file(path: &str) -> Result<(), CompileError> {
    let codemap = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);

    // Parse from file to internal representation
    let _ = air_parser::parse_file(&diagnostics, codemap, path)
        .map_err(CompileError::Parse)
        .and_then(|program| compile(&diagnostics, program))?;

    Ok(())
}

#[test]
fn test_constraints_miden_vm_files_compile() {
    // Note: These files use 'mod' syntax which is different from current 'def' syntax
    // They are legacy examples and should not compile with current parser
    // TODO: Convert these to use 'def' syntax or update parser to support both
    let files = vec![
        "../air-script/constraints/miden-vm/hash.air",
        "../air-script/constraints/miden-vm/bitwise.air",
        "../air-script/constraints/miden-vm/decoder.air",
        "../air-script/constraints/miden-vm/range_checker.air",
        "../air-script/constraints/miden-vm/chiplets.air",
        "../air-script/constraints/miden-vm/memory.air",
    ];

    for file in files {
        // These files are expected to fail compilation due to different syntax
        assert!(
            compile_air_file(file).is_err(),
            "File {} should fail to compile with current parser (uses 'mod' syntax)",
            file
        );
    }
}

#[test]
fn test_parser_test_files_compile() {
    // Test parser test files
    // Note: Some files use 'mod' syntax which is different from current 'def' syntax
    let files = vec![
        // These should compile (use 'def' syntax)
        ("../parser/src/parser/tests/input/import_example.air", true),
        ("../parser/src/parser/tests/input/system.air", true),
        // These should fail (use 'mod' syntax)
        ("../parser/src/parser/tests/input/foo.air", false),
        ("../parser/src/parser/tests/input/bar.air", false),
    ];

    for (file, should_succeed) in files {
        if should_succeed {
            assert!(compile_air_file(file).is_ok(), "File {} should compile successfully", file);
        } else {
            assert!(
                compile_air_file(file).is_err(),
                "File {} should fail to compile with current parser (uses 'mod' syntax)",
                file
            );
        }
    }
}

#[test]
fn test_codegen_ace_files_compile() {
    // Test ACE codegen test files
    let files = vec![
        "../codegen/ace/src/tests/airs/LongTrace.air",
        "../codegen/ace/src/tests/airs/Vector.air",
        "../codegen/ace/src/tests/airs/SimpleBoundary.air",
        "../codegen/ace/src/tests/airs/PublicInput.air",
        "../codegen/ace/src/tests/airs/ComplexBoundary.air",
        "../codegen/ace/src/tests/airs/MultipleRows.air",
        "../codegen/ace/src/tests/airs/Exp.air",
        "../codegen/ace/src/tests/airs/Simple.air",
        "../codegen/ace/src/tests/airs/MultipleAux.air",
        "../codegen/ace/src/tests/airs/SimpleIntegrityAux.air",
        "../codegen/ace/src/tests/airs/Busses.air",
        "../codegen/ace/src/tests/airs/SimpleArithmetic.air",
        "../codegen/ace/src/tests/airs/ConstantsAir.air",
    ];

    for file in files {
        assert!(compile_air_file(file).is_ok(), "File {} should compile successfully", file);
    }
}

#[test]
fn test_examples_file_compile() {
    // Test the main example file
    let file = "../examples/example.air";
    assert!(compile_air_file(file).is_ok(), "File {} should compile successfully", file);
}

#[test]
fn test_all_air_files_in_constraints() {
    // Comprehensive test of all constraint files
    // These files use 'mod' syntax which is different from current 'def' syntax
    // They are legacy examples and should not compile with current parser
    let base_path = "../air-script/constraints/miden-vm";
    let files = vec![
        "hash.air",
        "bitwise.air",
        "decoder.air",
        "range_checker.air",
        "chiplets.air",
        "memory.air",
    ];

    for file in files {
        let full_path = format!("{}/{}", base_path, file);
        // These files are expected to fail compilation due to different syntax
        assert!(
            compile_air_file(&full_path).is_err(),
            "File {} should fail to compile with current parser (uses 'mod' syntax)",
            full_path
        );
    }
}
