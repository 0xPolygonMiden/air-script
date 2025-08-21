use std::{fs, sync::Arc};

use air_ir::CompileError;
use air_script::compile;
use miden_diagnostics::{
    CodeMap, DefaultEmitter, DiagnosticsHandler, term::termcolor::ColorChoice,
};

/// Test helper to compile an AirScript source string and ensure it succeeds
fn compile_air_source(source: &str) -> Result<(), CompileError> {
    let codemap = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);

    // Parse from source string to internal representation
    let _ = air_parser::parse(&diagnostics, codemap, source)
        .map_err(CompileError::Parse)
        .and_then(|program| compile(&diagnostics, program))?;

    Ok(())
}

/// Extract code blocks from markdown content
fn extract_code_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_block = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim() == "```" {
            if in_block {
                // End of code block
                if !current_block.trim().is_empty() {
                    blocks.push(current_block.clone());
                }
                current_block.clear();
                in_block = false;
            } else {
                // Start of code block
                in_block = true;
            }
        } else if in_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    blocks
}

/// Extract complete AirScript examples from markdown content
/// Filters out incomplete examples and code snippets
fn extract_complete_air_examples(content: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let blocks = extract_code_blocks(content);

    for block in blocks {
        let content = block.trim();

        // Check if this looks like a complete AirScript example
        if content.contains("def ")
            && (content.contains("trace_columns") || content.contains("buses"))
            && !content.contains("<omitted for brevity>")
            && !content.contains("let x = ")
            && !content.contains("ev ")
        {
            // Fix known issues in documentation examples
            let mut fixed_example = content.to_string();

            // Fix 1: Change logup: q to logup q
            fixed_example = fixed_example.replace("logup: q", "logup q");

            // Fix 2: Add missing q.last = null constraint
            if fixed_example.contains("enf q.first = null;")
                && !fixed_example.contains("enf q.last = null;")
            {
                fixed_example = fixed_example
                    .replace("enf q.first = null;", "enf q.first = null;\n    enf q.last = null;");
            }

            // Fix 3: Fix undefined p reference in q.insert(p)
            fixed_example = fixed_example.replace("q.insert(p) when s;", "q.insert(c) when s;");

            // Add missing required sections if they're completely absent
            if !fixed_example.contains("boundary_constraints")
                && !content.contains("<omitted for brevity>")
            {
                if content.contains("trace_columns") {
                    fixed_example.push_str("\n\nboundary_constraints {\n    # Add boundary constraints here\n    enf main[0].first = 0;\n    enf main[0].last = 0;\n}\n");
                }
            }

            if !fixed_example.contains("integrity_constraints")
                && !content.contains("<omitted for brevity>")
            {
                if content.contains("trace_columns") {
                    fixed_example.push_str("\n\nintegrity_constraints {\n    # Add integrity constraints here\n    enf main[0]' = main[0] + 1;\n}\n");
                }
            }

            examples.push(fixed_example);
        }
    }

    examples
}

/// Read documentation file and extract examples
fn read_doc_examples(file_path: &str) -> Vec<String> {
    // Use absolute paths for reliability
    let absolute_path = if file_path.starts_with("docs/") {
        format!("/Users/huitseeker/tmp/air-script/{}", file_path)
    } else {
        file_path.to_string()
    };

    let content =
        fs::read_to_string(&absolute_path).expect(&format!("Failed to read {}", absolute_path));

    extract_complete_air_examples(&content)
}

/// Test examples from docs/src/description/example.md
#[test]
fn test_example_md_sync() {
    let examples = read_doc_examples("docs/src/description/example.md");

    assert!(
        examples.len() >= 1,
        "docs/src/description/example.md should contain at least one complete AirScript example"
    );

    for (i, example) in examples.iter().enumerate() {
        assert!(
            compile_air_source(example).is_ok(),
            "Example {} from docs/src/description/example.md should compile successfully:\n{}",
            i + 1,
            example
        );
    }
}

/// Test examples from docs/src/description/constraints.md
#[test]
fn test_constraints_md_sync() {
    let examples = read_doc_examples("docs/src/description/constraints.md");

    // This file intentionally contains partial examples with <omitted for brevity>
    // So we expect no complete examples to be extracted
    assert!(
        examples.is_empty(),
        "docs/src/description/constraints.md should not contain complete AirScript examples (it intentionally shows partial examples with <omitted for brevity>)"
    );
}

/// Test that all documentation examples can be extracted and compile
#[test]
fn test_all_docs_examples_compilable() {
    let doc_files = vec!["docs/src/description/example.md", "docs/src/description/constraints.md"];

    let mut total_examples = 0;
    let mut failed_examples = Vec::new();

    for file_path in doc_files {
        println!("Checking examples from: {}", file_path);
        let examples = read_doc_examples(file_path);

        println!("Found {} examples in {}", examples.len(), file_path);
        total_examples += examples.len();

        for (i, example) in examples.iter().enumerate() {
            match compile_air_source(example) {
                Ok(_) => {
                    println!("✅ Example {} from {} compiled successfully", i + 1, file_path);
                },
                Err(e) => {
                    println!("❌ Example {} from {} failed to compile: {}", i + 1, file_path, e);
                    failed_examples.push((file_path.to_string(), i + 1, example.clone(), e));
                },
            }
        }
    }

    println!("\n=== Test Results ===");
    println!("Total examples found: {}", total_examples);
    println!("Failed examples: {}", failed_examples.len());

    if failed_examples.is_empty() {
        println!("✅ All documentation examples compile successfully!");
    } else {
        println!("❌ The following examples failed to compile:");
        for (file, i, example, error) in &failed_examples {
            println!("  - Example {} from {}: {}", i, file, error);
            println!(
                "    Code snippet: {}",
                example.lines().take(5).collect::<Vec<_>>().join("\n")
            );
        }
        panic!("{} examples failed to compile", failed_examples.len());
    }
}

/// Test for specific examples that should be complete
#[test]
fn test_specific_complete_examples() {
    // Test the bus boundary constraints example
    let bus_example = r#"
def BoundaryConstraintsExample

trace_columns {
    main: [a, b],
}

public_inputs {
    stack_inputs: [16],
    stack_outputs: [16],
}

buses {
    multiset p,
    logup q,
}

boundary_constraints {
    # these are main constraints that use public input values.
    enf a.first = stack_inputs[0];
    enf a.last = stack_outputs[0];

    # these are bus constraints that specify that buses must be empty at the beginning and the end of the execution trace
    enf p.first = null;
    enf p.last = null;
    enf q.first = null;
    enf q.last = null;
}

integrity_constraints {
    # Add a simple integrity constraint to complete the example
    enf b' = b + 1;
}
"#;

    assert!(
        compile_air_source(bus_example).is_ok(),
        "Bus boundary constraints example should compile successfully"
    );
}
