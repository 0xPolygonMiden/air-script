# Changelog

## 0.4.0 (TBD)

## 0.3.0 (2023-07-12)

- Added support for library modules.
- Added support for evaluator functions.
- Added support for constraint comprehension.
- Added support for conditional constraints.
- Refactored parser to make it more robust and provide better error reporting.
- Fixed grammar ambiguities.
- Added initial implementation of Miden assembly backend.

## 0.2.0 (2023-02-23)

- Added support for named constants (scalars, vectors, and matrices).
- Added support for intermediate variables in `boundary_constraints` and `integrity_constraints` sections (scalars, vectors, and matrices).
- Added support for binding identifier names to groups of trace columns (in addition to single columns, which were already supported).
- Added the `$main` and `$aux` built-ins for accessing columns in the execution trace by index.
- [BREAKING] Replaced the `$rand` built-in for accessing random values with an explicit `random_values` declaration section, enabling use of a custom identifier for the random values array.
- Added support for binding identifiers to specific random values or groups of random values.
- Made significant changes to the IR, including:
  - Moved the boundary constraints into the algebraic constraint graph.
  - Made the trace reference more general to support additional trace segments in the future.
  - Added analysis to differentiate between validity and transition constraints.
  - Added the `Sub` operation and removed the `Neg` operation.
- [FIX] Fixed a bug in the Winterfell codegen output for auxiliary transition constraints.
- Improved the Winterfell codegen by consolidating code generation for boundary and transition constraints and removing redundant parentheses.

## 0.1.0 (2022-11-10)

- Initial release of AirScript, including a minimal but complete implementation of the AirScript compiler for an initial basic set of language features.
