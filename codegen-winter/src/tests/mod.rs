mod boundary_constraints;
mod transition_constraints;

// TEST HANDLER
// ================================================================================================
#[macro_export]
macro_rules! build_codegen_test {
    ($source:expr) => {{
        $crate::tests::CodegenTest::new($source)
    }};
}

pub const INDENT_SPACES: usize = 4;

pub struct CodegenTest {
    source_lines: Vec<String>,
    start_line_number: usize,
    end_line_number: usize,
    indent_level: usize,
}

impl CodegenTest {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Creates a new test, from the source string.
    pub fn new(source: &str) -> Self {
        CodegenTest {
            source_lines: source.lines().map(|l| l.to_string()).collect(),
            start_line_number: 0,
            end_line_number: 0,
            indent_level: 0,
        }
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------
    pub fn expect_lines(&mut self, expected_lines: &[String]) {
        self.end_line_number = self.start_line_number + expected_lines.len();
        for (expected_line, source_line) in expected_lines.iter().zip(&self.source_lines[self.start_line_number..self.end_line_number]) {
            assert_eq!(expected_line, source_line);
        }
        self.start_line_number = self.end_line_number + 1;
    }

    pub fn expect_imports(&mut self) {
        let mut expected_imports = get_expected_imports();
        for (expected_line, source_line) in expected_imports.iter().zip(&self.source_lines[self.start_line_number..expected_imports.len()]) {
            assert_eq!(expected_line, source_line);
        }
        self.start_line_number = expected_imports.len() + 1;
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------
    pub fn add_new_line(&mut self) {
        self.start_line_number += 1;
    }

    pub fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn decrease_indent(&mut self) {
        self.indent_level -= 1;
    }

}

// HELPERS
// ================================================================================================
fn get_expected_imports() -> Vec<String> {
    vec![
        "use winter_air::TransitionConstraintDegree::TransitionConstraintDegree;".to_string(),
        "use winter_air::TraceInfo::TraceInfo;".to_string(),
        "use winter_air::ProofOptions::WinterProofOptions;".to_string(),
        "use winter_air::EvaluationFrame::EvaluationFrame;".to_string(),
        "use winter_air::Assertion::Assertion;".to_string(),
        "use winter_air::AirContext::AirContext;".to_string(),
        "use winter_air::Air::Air;".to_string(),
        "use winter_utils::collections::Vec::Vec;".to_string(),
    ]
}

fn indent(lines: &mut Vec<String>, indent_level: usize) {
    let mut indent = (0..INDENT_SPACES * indent_level).map(|_| " ").collect::<String>();
    for mut line in lines {
        *line = format!("{}{}", indent, line);
    }
}

fn dedent(lines: &mut Vec<String>, indent_level: usize) {
    for mut line in lines {
        *line = line[(INDENT_SPACES * indent_level)..].to_string();
    }
}

// TODO:
// - check valid file name