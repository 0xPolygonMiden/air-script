#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar);

pub mod ast;
pub mod error;
pub mod lexer;

#[cfg(test)]
pub mod tests;
