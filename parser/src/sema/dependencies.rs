use crate::ast::{ModuleId, QualifiedIdentifier};

/// Represents the graph of dependencies between module items and items they
/// reference, either in the same module, or via imports.
///
/// The dependency graph is used to construct the final [Program] representation,
/// containing only those parts of the program which are referenced from the root
/// module.
pub type DependencyGraph = petgraph::graphmap::DiGraphMap<QualifiedIdentifier, DependencyType>;

/// Represents the graph of dependencies between modules, with no regard to what
/// items in those modules are actually used. In other words, this graph tells us
/// what modules depend on what other modules in the program.
pub type ModuleGraph = petgraph::graphmap::DiGraphMap<ModuleId, ()>;

/// Represents the type of edges in the dependency graph
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DependencyType {
    /// Depends on an imported constant
    Constant,
    /// Depends on an imported evaluator function
    Evaluator,
    /// Depends on an imported function
    Function,
    /// Depends on a periodic_columns declaration (not visible as an export)
    PeriodicColumn,
}
