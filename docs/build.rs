use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=docs/");
    
    // Check if we're in docs test mode
    if env::var("DOCS_TEST").unwrap_or_else(|_| String::from("0")) == "1" {
        // Run mdbook test if available
        match Command::new("mdbook").arg("test").current_dir("docs").output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("mdbook test failed:");
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                    std::process::exit(1);
                }
            },
            Err(_) => {
                eprintln!("Warning: mdbook not found, skipping documentation build test");
            }
        }
    }
}