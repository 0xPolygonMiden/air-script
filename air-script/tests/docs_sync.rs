use std::{path::Path, process::Command};

#[test]
fn docs_sync() {
    let examples_dir = Path::new("../docs/examples");
    // Use CARGO_MANIFEST_DIR to build an absolute path to airc, needed on Windows to correctly use `current_dir`.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let airc_path = Path::new(&manifest_dir).join("../target/release/airc");

    // Build the CLI tool first
    let build_output = Command::new("cargo")
        .args(["build", "--release", "-p", "air-script"])
        .current_dir("../")
        .output()
        .expect("Failed to build airc CLI");

    assert!(
        build_output.status.success(),
        "Failed to build airc CLI: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Find all .air files in the examples directory
    let air_files: Vec<_> = std::fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| {
            let path = entry.expect("Failed to read directory entry").path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("air") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    assert!(!air_files.is_empty(), "No .air files found in docs/examples");

    // Compile each example
    for air_file in air_files {
        let file_name = air_file.file_name().unwrap().to_string_lossy();
        let output_path = air_file.with_extension("rs");

        let output = Command::new(&airc_path)
            .args(["transpile", air_file.to_str().unwrap(), "-o", output_path.to_str().unwrap()])
            .current_dir("../")
            .output()
            .unwrap_or_else(|_| panic!("Failed to transpile {file_name}"));

        assert!(
            output.status.success(),
            "Failed to transpile {}: {}",
            file_name,
            String::from_utf8_lossy(&output.stderr)
        );

        println!("Successfully transpiled: {file_name}");

        // Clean up generated Rust files
        let _ = std::fs::remove_file(output_path);
    }

    println!("All documentation examples compiled successfully!");
}
