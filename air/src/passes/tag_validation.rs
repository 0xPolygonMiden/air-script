//! Final tag validation pass for AIR constraints.
//!
//! Tagging methodology (summary):
//! - Use `@tag(<id>)` on simple `enf` constraints and `@tag_transition(<id>)` on bus declarations.
//! - Use `@tag_range(..)` or `@tag([..])` on comprehension constraints to map each expanded
//!   constraint to a stable id.
//! - Tags propagate AST -> MIR -> AIR, and are validated against `CURRENT_MAX_ID`.
//! - Validation runs after bus expansion/CSE so tags reflect the final constraint set.
use air_parser::ast::TraceSegmentId;
use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, Severity};

use crate::{Air, CompileError};

/// Validates constraint tags against CURRENT_MAX_ID after all constraints are expanded.
pub struct TagValidation<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> TagValidation<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

impl Pass for TagValidation<'_> {
    type Input<'a> = Air;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let Some(max_id) = ir.expected_max_constraint_id else {
            return Ok(ir);
        };

        let expected = max_id as usize + 1;
        let mut seen = vec![false; expected];
        let mut count = 0usize;

        for segment in [TraceSegmentId::Main, TraceSegmentId::Aux] {
            if usize::from(segment) >= ir.trace_segment_widths.len() {
                continue;
            }

            for (index, constraint) in
                ir.constraints.boundary_constraints(segment).iter().enumerate()
            {
                let tag = constraint.tag().ok_or_else(|| {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("constraint missing tag")
                        .with_note(format!("boundary constraint {index} in segment {segment}"))
                        .emit();
                    CompileError::Failed
                })?;
                if tag > max_id {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("constraint tag exceeds CURRENT_MAX_ID")
                        .with_note(format!("tag {tag} is out of range (max {max_id})"))
                        .emit();
                    return Err(CompileError::Failed);
                }
                let idx = tag as usize;
                if seen[idx] {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("duplicate constraint tag")
                        .with_note(format!("tag {tag} reused"))
                        .emit();
                    return Err(CompileError::Failed);
                }
                seen[idx] = true;
                count += 1;
            }

            for (index, constraint) in
                ir.constraints.integrity_constraints(segment).iter().enumerate()
            {
                let tag = constraint.tag().ok_or_else(|| {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("constraint missing tag")
                        .with_note(format!("integrity constraint {index} in segment {segment}"))
                        .emit();
                    CompileError::Failed
                })?;
                if tag > max_id {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("constraint tag exceeds CURRENT_MAX_ID")
                        .with_note(format!("tag {tag} is out of range (max {max_id})"))
                        .emit();
                    return Err(CompileError::Failed);
                }
                let idx = tag as usize;
                if seen[idx] {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("duplicate constraint tag")
                        .with_note(format!("tag {tag} reused"))
                        .emit();
                    return Err(CompileError::Failed);
                }
                seen[idx] = true;
                count += 1;
            }
        }

        if count != expected {
            self.diagnostics
                .diagnostic(Severity::Error)
                .with_message("constraint tag count does not match CURRENT_MAX_ID")
                .with_note(format!("expected {expected} constraints, found {count}"))
                .emit();
            return Err(CompileError::Failed);
        }

        Ok(ir)
    }
}
