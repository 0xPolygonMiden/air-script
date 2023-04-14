use super::{
    BTreeMap, ConstraintDomain, IntegrityConstraintDegree, SemanticError, TraceSegment, Value,
    AUX_SEGMENT, DEFAULT_SEGMENT,
};

// ALGEBRAIC GRAPH
// ================================================================================================

/// The AlgebraicGraph is a directed acyclic graph used to represent integrity constraints. To
/// store it compactly, it is represented as a vector of nodes where each node references other
/// nodes by their index in the vector.
///
/// Within the graph, constraint expressions can overlap and share subgraphs, since new expressions
/// reuse matching existing nodes when they are added, rather than creating new nodes.
///
/// - Leaf nodes (with no outgoing edges) are constants or references to trace cells (i.e. column 0
///   in the current row or column 5 in the next row).
/// - Tip nodes with no incoming edges (no parent nodes) always represent constraints, although they
///   do not necessarily represent all constraints. There could be constraints which are also
///   subgraphs of other constraints.
#[derive(Default, Debug, Clone)]
pub struct AlgebraicGraph {
    /// All nodes in the graph.
    nodes: Vec<Node>,
}

impl AlgebraicGraph {
    /// Creates a new graph from a list of nodes.
    fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns the node with the specified index.
    pub fn node(&self, index: &NodeIndex) -> &Node {
        &self.nodes[index.0]
    }

    /// Returns the number of nodes in the graph.
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the degree of the subgraph which has the specified node as its tip.
    pub fn degree(&self, index: &NodeIndex) -> IntegrityConstraintDegree {
        let mut cycles: BTreeMap<usize, usize> = BTreeMap::new();
        let base = self.accumulate_degree(&mut cycles, index);

        if cycles.is_empty() {
            IntegrityConstraintDegree::new(base)
        } else {
            IntegrityConstraintDegree::with_cycles(base, cycles.values().cloned().collect())
        }
    }

    /// TODO: docs
    pub fn node_details(
        &self,
        index: &NodeIndex,
        default_domain: ConstraintDomain,
    ) -> Result<(TraceSegment, ConstraintDomain), SemanticError> {
        // recursively walk the subgraph and infer the trace segment and domain
        match self.node(index).op() {
            Operation::Value(value) => match value {
                Value::InlineConstant(_) | Value::BoundConstant(_) => {
                    Ok((DEFAULT_SEGMENT, default_domain))
                }
                Value::PeriodicColumn(_, _) => {
                    if default_domain.is_boundary() {
                        return Err(SemanticError::invalid_periodic_column_access_in_bc());
                    }
                    // the default domain for [IntegrityConstraints] is `EveryRow`
                    Ok((DEFAULT_SEGMENT, ConstraintDomain::EveryRow))
                }
                Value::PublicInput(_, _) => {
                    if default_domain.is_integrity() {
                        return Err(SemanticError::invalid_public_input_access_in_ic());
                    }
                    Ok((DEFAULT_SEGMENT, default_domain))
                }
                Value::RandomValue(_) => Ok((AUX_SEGMENT, default_domain)),
                Value::TraceElement(trace_access) => {
                    let domain = if default_domain.is_boundary() {
                        if trace_access.row_offset() == 0 {
                            default_domain
                        } else {
                            return Err(SemanticError::invalid_trace_offset_in_bc(trace_access));
                        }
                    } else {
                        trace_access.row_offset().into()
                    };

                    Ok((trace_access.trace_segment(), domain))
                }
            },
            Operation::Add(lhs, rhs) | Operation::Sub(lhs, rhs) | Operation::Mul(lhs, rhs) => {
                let (lhs_segment, lhs_domain) = self.node_details(lhs, default_domain)?;
                let (rhs_segment, rhs_domain) = self.node_details(rhs, default_domain)?;

                let trace_segment = lhs_segment.max(rhs_segment);
                let domain = lhs_domain.merge(&rhs_domain)?;

                Ok((trace_segment, domain))
            }
            Operation::Exp(lhs, _) => self.node_details(lhs, default_domain),
        }
    }

    // --- PUBLIC MUTATORS ------------------------------------------------------------------------
    /// Insert the operation and return its node index. If an identical node already exists, return
    /// that index instead.
    pub(crate) fn insert_node(&mut self, op: Operation) -> NodeIndex {
        self.nodes.iter().position(|n| *n.op() == op).map_or_else(
            || {
                // create a new node.
                let index = self.nodes.len();
                self.nodes.push(Node { op });
                NodeIndex(index)
            },
            |index| {
                // return the existing node's index.
                NodeIndex(index)
            },
        )
    }

    /// Adds the nodes from the provided graph into self.
    pub(crate) fn extend(&mut self, graph: Self) {
        self.nodes.extend(graph.nodes);
    }

    /// Clones the graph, updates all nodes that reference the trace according to the provided set
    /// of offsets, and returns the new graph.
    pub(crate) fn clone_with_offsets(&self, offsets: &[Vec<usize>]) -> Self {
        let nodes = self
            .nodes
            .iter()
            .map(|node| match node.op() {
                Operation::Value(Value::TraceElement(trace_access)) => {
                    let value = Value::TraceElement(trace_access.clone_with_offsets(offsets));
                    Node {
                        op: Operation::Value(value),
                    }
                }
                _ => node.clone(),
            })
            .collect::<Vec<_>>();

        Self::new(nodes)
    }

    // --- HELPERS --------------------------------------------------------------------------------

    /// Recursively accumulates the base degree and the cycle lengths of the periodic columns.
    fn accumulate_degree(&self, cycles: &mut BTreeMap<usize, usize>, index: &NodeIndex) -> usize {
        // recursively walk the subgraph and compute the degree from the operation and child nodes
        match self.node(index).op() {
            Operation::Value(value) => match value {
                Value::InlineConstant(_)
                | Value::BoundConstant(_)
                | Value::RandomValue(_)
                | Value::PublicInput(_, _) => 0,
                Value::TraceElement(_) => 1,
                Value::PeriodicColumn(index, cycle_len) => {
                    cycles.insert(*index, *cycle_len);
                    0
                }
            },
            Operation::Add(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                let rhs_base = self.accumulate_degree(cycles, rhs);
                lhs_base.max(rhs_base)
            }
            Operation::Sub(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                let rhs_base = self.accumulate_degree(cycles, rhs);
                lhs_base.max(rhs_base)
            }
            Operation::Mul(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                let rhs_base = self.accumulate_degree(cycles, rhs);
                lhs_base + rhs_base
            }
            Operation::Exp(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                lhs_base * rhs
            }
        }
    }
}

/// Reference to a node in a graph by its index in the nodes vector of the graph struct.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct NodeIndex(usize);

impl NodeIndex {
    /// Clones the [NodeIndex], updates its value by adding the provided offset, and returns the new
    /// [NodeIndex].
    pub fn clone_with_offset(&self, offset: usize) -> NodeIndex {
        Self(self.0 + offset)
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    /// The operation represented by this node
    op: Operation,
}

impl Node {
    pub fn op(&self) -> &Operation {
        &self.op
    }
}

/// An integrity constraint operation or value reference.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Operation {
    /// TODO: docs
    Value(Value),
    /// Addition operation applied to the nodes with the specified indices.
    Add(NodeIndex, NodeIndex),
    /// Subtraction operation applied to the nodes with the specified indices.
    Sub(NodeIndex, NodeIndex),
    /// Multiplication operation applied to the nodes with the specified indices.
    Mul(NodeIndex, NodeIndex),
    /// Exponentiation operation applied to the node with the specified index, using the provided
    /// value as the power.
    /// TODO: Support non const exponents.
    Exp(NodeIndex, usize),
}

impl Operation {
    pub fn precedence(&self) -> usize {
        match self {
            Operation::Add(_, _) => 1,
            Operation::Sub(_, _) => 2,
            Operation::Mul(_, _) => 3,
            _ => 4,
        }
    }
}
