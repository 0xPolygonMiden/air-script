use super::{expect_valid_tokenization, Token};

// EVALUATOR FUNCTION VALID TOKENIZATION
// ================================================================================================

#[test]
fn ev_fn_with_main_cols() {
    let source = "
    ev ev_fn(main: [state[12]]):
        let s1 = [x^7 for x in state]
        let s2 = [x^7 for x in s1]
  
        enf s1[0] = s2[0]";

    let tokens = [
        Token::Ev,
        Token::Ident("ev_fn".to_string()),
        Token::LParen,
        Token::Main,
        Token::Colon,
        Token::LBracket,
        Token::Ident("state".to_string()),
        Token::LBracket,
        Token::Num(12),
        Token::RBracket,
        Token::RBracket,
        Token::RParen,
        Token::Colon,
        Token::Let,
        Token::Ident("s1".to_string()),
        Token::Equal,
        Token::LBracket,
        Token::Ident("x".to_string()),
        Token::Caret,
        Token::Num(7),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("state".to_string()),
        Token::RBracket,
        Token::Let,
        Token::Ident("s2".to_string()),
        Token::Equal,
        Token::LBracket,
        Token::Ident("x".to_string()),
        Token::Caret,
        Token::Num(7),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("s1".to_string()),
        Token::RBracket,
        Token::Enf,
        Token::Ident("s1".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Equal,
        Token::Ident("s2".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
    ];

    expect_valid_tokenization(source, tokens.to_vec());
}

#[test]
fn ev_fn_with_main_and_aux_cols() {
    let source = "
    ev ev_fn(main: [main_state[12]], aux: [aux_state[12]]):
        let ms = [x^7 for x in main_state]
        let ms_sum = sum([x^7 for x in main_state])
        let as = [x^7 for x in aux_state]
        
        enf main_state[0] = ms[0] + ms_sum
        enf aux_state[0] = as[0] * $rand[0]";

    let tokens = [
        Token::Ev,
        Token::Ident("ev_fn".to_string()),
        Token::LParen,
        Token::Main,
        Token::Colon,
        Token::LBracket,
        Token::Ident("main_state".to_string()),
        Token::LBracket,
        Token::Num(12),
        Token::RBracket,
        Token::RBracket,
        Token::Comma,
        Token::Aux,
        Token::Colon,
        Token::LBracket,
        Token::Ident("aux_state".to_string()),
        Token::LBracket,
        Token::Num(12),
        Token::RBracket,
        Token::RBracket,
        Token::RParen,
        Token::Colon,
        Token::Let,
        Token::Ident("ms".to_string()),
        Token::Equal,
        Token::LBracket,
        Token::Ident("x".to_string()),
        Token::Caret,
        Token::Num(7),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("main_state".to_string()),
        Token::RBracket,
        Token::Let,
        Token::Ident("ms_sum".to_string()),
        Token::Equal,
        Token::Sum,
        Token::LParen,
        Token::LBracket,
        Token::Ident("x".to_string()),
        Token::Caret,
        Token::Num(7),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("main_state".to_string()),
        Token::RBracket,
        Token::RParen,
        Token::Let,
        Token::Ident("as".to_string()),
        Token::Equal,
        Token::LBracket,
        Token::Ident("x".to_string()),
        Token::Caret,
        Token::Num(7),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("aux_state".to_string()),
        Token::RBracket,
        Token::Enf,
        Token::Ident("main_state".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Equal,
        Token::Ident("ms".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Plus,
        Token::Ident("ms_sum".to_string()),
        Token::Enf,
        Token::Ident("aux_state".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Equal,
        Token::Ident("as".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Star,
        Token::DeclIdentRef("$rand".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
    ];

    expect_valid_tokenization(source, tokens.to_vec());
}

#[test]
fn ev_fn_call() {
    let source = "
        integrity_constraints:
            enf ev_fn([state[12]])";

    let tokens = [
        Token::IntegrityConstraints,
        Token::Colon,
        Token::Enf,
        Token::Ident("ev_fn".to_string()),
        Token::LParen,
        Token::LBracket,
        Token::Ident("state".to_string()),
        Token::LBracket,
        Token::Num(12),
        Token::RBracket,
        Token::RBracket,
        Token::RParen,
    ];

    expect_valid_tokenization(source, tokens.to_vec());
}
