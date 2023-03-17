use crate::error::Error;
use core::ops::Range;

pub use logos::{Lexer, Logos};

#[cfg(test)]
mod tests;

pub type Span = Range<usize>;

#[derive(Logos, Clone, Eq, Debug, PartialEq)]
pub enum Token {
    // PRIMITIVES
    // --------------------------------------------------------------------------------------------
    /// Identifiers should start with alphabet followed by one or more alpha numeric characters
    /// or an underscore.
    #[regex("[a-zA-Z][a-zA-Z0-9_]*", |tok| tok.slice().to_string())]
    Ident(String),

    /// Integers should only contain numeric characters.
    #[regex(r"[0-9]+", |tok| tok.slice().to_string())]
    Num(String),

    // DECLARATION KEYWORDS
    // --------------------------------------------------------------------------------------------
    /// Used to declare AIR constraints module.
    #[token("def")]
    Def,

    /// Used to declare intermediate variables in the AIR constraints module.
    #[token("let")]
    Let,

    /// Used to declare constants in the AIR constraints module.
    #[token("const")]
    Const,

    /// Used to declare trace columns section in the AIR constraints module.
    #[token("trace_columns")]
    TraceColumns,

    /// Used to declare main trace columns.
    #[token("main")]
    MainDecl,

    /// Used to declare aux trace columns.
    #[token("aux")]
    AuxDecl,

    /// A reserved keyword for accessing main columns by index
    #[token("$main")]
    MainAccess,

    /// A reserved keyword for accessing aux columns by index
    #[token("$aux")]
    AuxAccess,

    /// Keyword to declare the public inputs declaration section for the AIR.
    #[token("public_inputs")]
    PublicInputs,

    /// Keyword to declare the periodic columns declaration section for the AIR.
    #[token("periodic_columns")]
    PeriodicColumns,

    /// Keyword to declare random values section in the AIR constraints module.
    #[token("random_values")]
    RandomValues,

    /// A reserved symbol for accessing random values provided by the verifier.
    #[token("$")]
    Rand,

    /// Keyword to declare the evaluator function section in the AIR constraints module.
    #[token("ev")]
    EvaluatorFunction,

    // BOUNDARY CONSTRAINT KEYWORDS
    // --------------------------------------------------------------------------------------------
    /// Marks the beginning of boundary constraints section in the constraints file.
    #[token("boundary_constraints")]
    BoundaryConstraints,

    /// Used to represent the first row of the column to which a boundary constraint is applied.
    #[token("first")]
    First,

    /// Used to represent the last row of the column to which a boundary constraint is applied.
    #[token("last")]
    Last,

    // INTEGRITY CONSTRAINT KEYWORDS
    // --------------------------------------------------------------------------------------------
    /// Marks the beginning of integrity constraints section in the constraints file.
    #[token("integrity_constraints")]
    IntegrityConstraints,

    /// A modifier for identifiers used to indicate the next row.
    #[token("'")]
    Next,

    // LIST COMPREHENSION KEYWORDS
    // --------------------------------------------------------------------------------------------
    #[token("for")]
    For,

    #[token("in")]
    In,

    /// Used to declare sum list folding operation in the AIR constraints module.
    #[token("sum")]
    Sum,

    /// Used to declare prod list folding operation in the AIR constraints module.
    #[token("prod")]
    Prod,

    // GENERAL KEYWORDS
    // --------------------------------------------------------------------------------------------
    /// Keyword to signify that a constraint needs to be enforced
    #[token("enf")]
    Enf,

    #[token("match")]
    Match,

    // OPERATORS
    // --------------------------------------------------------------------------------------------
    /// Asserts LHS of the expression is equal to RHS of the expression.
    #[token("=")]
    Equal,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Mul,

    #[token("^")]
    Exp,

    // SELECTOR KEYWORDS
    // --------------------------------------------------------------------------------------------
    #[token("&")]
    And,

    #[token("|")]
    Or,

    #[token("!")]
    Not,

    #[token("when")]
    When,

    // DELIMITERS
    // --------------------------------------------------------------------------------------------
    /// Used as a delimiter for section and sub section headings.
    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("[")]
    Lsqb,

    #[token("]")]
    Rsqb,

    #[token(".")]
    Dot,

    #[token("(")]
    Lparen,

    #[token(")")]
    Rparen,

    #[token("..")]
    Range,

    // UNDEFINED TOKENS AND TOKENS TO IGNORE
    // --------------------------------------------------------------------------------------------
    /// Error is returned on encountering unrecognized tokens.
    /// Whitespaces, tabs, newlines and comments are skipped.
    #[error]
    // Skip whitespaces, tabs and newlines
    #[regex(r"[ \t\n\f]+", logos::skip)]
    // Skip comments
    #[regex(r"#.*\n?", logos::skip)]
    Error,
}

impl Token {
    /// Convert logos tokens to tokens accepted by lalrpop.
    pub fn to_spanned((t, r): (Token, Span)) -> Result<(usize, Token, usize), Error> {
        if t == Token::Error {
            Err(Error::ScanError(r))
        } else {
            Ok((r.start, t, r.end))
        }
    }
}
