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
        Token::EvaluatorFunction,
        Token::Ident("ev_fn".to_string()),
        Token::Lparen,
        Token::MainDecl,
        Token::Colon,
        Token::Lsqb,
        Token::Ident("state".to_string()),
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Rsqb,
        Token::Rparen,
        Token::Colon,
        Token::Let,
        Token::Ident("s1".to_string()),
        Token::Equal,
        Token::Lsqb,
        Token::Ident("x".to_string()),
        Token::Exp,
        Token::Num("7".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("state".to_string()),
        Token::Rsqb,
        Token::Let,
        Token::Ident("s2".to_string()),
        Token::Equal,
        Token::Lsqb,
        Token::Ident("x".to_string()),
        Token::Exp,
        Token::Num("7".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("s1".to_string()),
        Token::Rsqb,
        Token::Enf,
        Token::Ident("s1".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Equal,
        Token::Ident("s2".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
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
        Token::EvaluatorFunction,
        Token::Ident("ev_fn".to_string()),
        Token::Lparen,
        Token::MainDecl,
        Token::Colon,
        Token::Lsqb,
        Token::Ident("main_state".to_string()),
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Rsqb,
        Token::Comma,
        Token::AuxDecl,
        Token::Colon,
        Token::Lsqb,
        Token::Ident("aux_state".to_string()),
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Rsqb,
        Token::Rparen,
        Token::Colon,
        Token::Let,
        Token::Ident("ms".to_string()),
        Token::Equal,
        Token::Lsqb,
        Token::Ident("x".to_string()),
        Token::Exp,
        Token::Num("7".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("main_state".to_string()),
        Token::Rsqb,
        Token::Let,
        Token::Ident("ms_sum".to_string()),
        Token::Equal,
        Token::Sum,
        Token::Lparen,
        Token::Lsqb,
        Token::Ident("x".to_string()),
        Token::Exp,
        Token::Num("7".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("main_state".to_string()),
        Token::Rsqb,
        Token::Rparen,
        Token::Let,
        Token::Ident("as".to_string()),
        Token::Equal,
        Token::Lsqb,
        Token::Ident("x".to_string()),
        Token::Exp,
        Token::Num("7".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("aux_state".to_string()),
        Token::Rsqb,
        Token::Enf,
        Token::Ident("main_state".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Equal,
        Token::Ident("ms".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Plus,
        Token::Ident("ms_sum".to_string()),
        Token::Enf,
        Token::Ident("aux_state".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Equal,
        Token::Ident("as".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Mul,
        Token::Rand,
        Token::Ident("rand".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
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
        Token::Lparen,
        Token::Lsqb,
        Token::Ident("state".to_string()),
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Rsqb,
        Token::Rparen,
    ];

    expect_valid_tokenization(source, tokens.to_vec());
}
