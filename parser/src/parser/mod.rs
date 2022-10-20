lalrpop_mod!(grammar, "/parser/grammar.rs");

pub use grammar::SourceParser;

#[cfg(test)]
mod tests;
