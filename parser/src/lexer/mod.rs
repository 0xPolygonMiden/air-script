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
    TraceColumnns,

    /// Used to declare main trace columns.
    #[token("main")]
    Main,

    /// Used to declare aux trace columns.
    #[token("aux")]
    Aux,

    /// Keyword to declare the public inputs declaration section for the AIR.
    #[token("public_inputs")]
    PublicInputs,

    /// Keyword to declare the periodic columns declaration section for the AIR.
    #[token("periodic_columns")]
    PeriodicColumns,

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

    // TRANSITION CONSTRAINT KEYWORDS
    // --------------------------------------------------------------------------------------------
    /// Marks the beginning of transition constraints section in the constraints file.
    #[token("transition_constraints")]
    TransitionConstraints,

    /// A modifier for identifiers used to indicate the next row.
    #[token("'")]
    Next,

    /// A reserved keyword for accessing random values provided by the verifier.
    #[token("$rand")]
    Rand,

    // GENERAL KEYWORDS
    // --------------------------------------------------------------------------------------------
    /// Keyword to signify that a constraint needs to be enforced
    #[token("enf")]
    Enf,

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
