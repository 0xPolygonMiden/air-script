//! MIR operation interner.
//!
//! The aim is to deduplicate identical MIR subtrees to cut memory use and downstream work. We do
//! this by building a structural key from operands, accessors, and span, then reusing a single
//! node. The tradeoff is conservative keys (e.g. include spans) to preserve diagnostics and
//! safety; that reduces sharing and only works when ops are immutable and self-describing.
//!
//! Interning means deduplicating identical structures by storing a single canonical instance
//! and reusing it everywhere. In this module we intern MIR ops so structurally identical
//! subtrees share nodes, reducing memory usage and speeding up later passes.

use std::collections::HashMap;

use miden_diagnostics::SourceSpan;

use crate::ir::{
    Accessor, Add, Exp, Link, Matrix, MirAccessType, Mul, Op, SpannedMirValue, Sub, Value, Vector,
};

#[derive(Hash, Eq, PartialEq, Clone)]
enum AccessKey {
    Default,
    Index { index: usize },
    Matrix { row: usize, col: usize },
}

impl AccessKey {
    fn from_access_type(access_type: &MirAccessType) -> Self {
        match access_type {
            MirAccessType::Default => AccessKey::Default,
            MirAccessType::Index(index) => AccessKey::Index { index: index.get_ptr() },
            MirAccessType::Matrix(row, col) => {
                AccessKey::Matrix { row: row.get_ptr(), col: col.get_ptr() }
            },
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
enum OpKey {
    Add {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Sub {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Mul {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Exp {
        lhs: usize,
        rhs: usize,
        span: SourceSpan,
    },
    Vector {
        elements: Vec<usize>,
        span: SourceSpan,
    },
    Matrix {
        rows: Vec<usize>,
        span: SourceSpan,
    },
    Accessor {
        indexable: usize,
        access: AccessKey,
        offset: usize,
        span: SourceSpan,
    },
    Value(SpannedMirValue),
}

/// A small, conservative interner for MIR ops.
///
/// Interning is safe only when:
/// - The op is immutable after creation (no later in-place rewrites).
/// - All semantics are encoded in the op fields (no hidden context).
/// - The op is not identity-sensitive (or identity is part of the key).
/// - Diagnostics are acceptable with shared spans (we include span in the key).
///
/// Interning is keyed by op structure (operand pointers + span) so repeated
/// subtrees can share nodes without changing semantics.
pub struct OpInterner {
    map: HashMap<OpKey, Link<Op>>,
}

impl OpInterner {
    /// Create a new empty interner.
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Intern an `Add` node.
    /// Safe to intern because it is a pure structural op (operands + span define semantics).
    pub fn intern_add(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Add {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Add::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    /// Intern a `Sub` node.
    /// Safe to intern because it is a pure structural op (operands + span define semantics).
    pub fn intern_sub(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Sub {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Sub::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    /// Intern a `Mul` node.
    /// Safe to intern because it is a pure structural op (operands + span define semantics).
    pub fn intern_mul(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Mul {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Mul::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    /// Intern an `Exp` node.
    /// Safe to intern because it is a pure structural op (operands + span define semantics).
    pub fn intern_exp(&mut self, lhs: Link<Op>, rhs: Link<Op>, span: SourceSpan) -> Link<Op> {
        let key = OpKey::Exp {
            lhs: lhs.get_ptr(),
            rhs: rhs.get_ptr(),
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Exp::create(lhs, rhs, span);
        self.map.insert(key, node.clone());
        node
    }

    /// Intern a `Vector` node.
    /// Safe to intern because the elements + span fully define the value.
    pub fn intern_vector(&mut self, elements: Vec<Link<Op>>, span: SourceSpan) -> Link<Op> {
        let elem_ptrs = elements.iter().map(|elem| elem.get_ptr()).collect();
        let key = OpKey::Vector { elements: elem_ptrs, span };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Vector::create(elements, span);
        self.map.insert(key, node.clone());
        node
    }

    /// Intern a `Matrix` node.
    /// Safe to intern because the rows + span fully define the value.
    pub fn intern_matrix(&mut self, rows: Vec<Link<Op>>, span: SourceSpan) -> Link<Op> {
        let row_ptrs = rows.iter().map(|row| row.get_ptr()).collect();
        let key = OpKey::Matrix { rows: row_ptrs, span };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Matrix::create(rows, span);
        self.map.insert(key, node.clone());
        node
    }

    /// Intern an `Accessor` node.
    /// Safe to intern because access type, offset, indexable, and span capture semantics.
    pub fn intern_accessor(
        &mut self,
        indexable: Link<Op>,
        access_type: MirAccessType,
        offset: usize,
        span: SourceSpan,
    ) -> Link<Op> {
        let access = AccessKey::from_access_type(&access_type);
        let key = OpKey::Accessor {
            indexable: indexable.get_ptr(),
            access,
            offset,
            span,
        };
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Accessor::create(indexable, access_type, offset, span);
        self.map.insert(key, node.clone());
        node
    }

    /// Intern a `Value` node.
    /// Safe to intern because the literal value is self-contained (no external context).
    pub fn intern_value(&mut self, value: SpannedMirValue) -> Link<Op> {
        let key = OpKey::Value(value.clone());
        if let Some(existing) = self.map.get(&key) {
            return existing.clone();
        }
        let node = Value::create(value);
        self.map.insert(key, node.clone());
        node
    }
}

impl Default for OpInterner {
    fn default() -> Self {
        Self::new()
    }
}
