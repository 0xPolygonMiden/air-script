use super::*;

/// [Block] defines the various node types represented
/// in the [MIR].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    arguments: Vec<Value>,
    instructions: Vec<Operation>,
}

impl Block {}
