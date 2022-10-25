use super::{expect_valid_tokenization, Token};

// BOUNDARY CONSTRAINTS VALID TOKENIZATION
// ================================================================================================

#[test]
fn first_boundary_constant() {
    let source = "enf clk.first = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Num("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn last_boundary_constant() {
    let source = "enf clk.last = 15";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::Last,
        Token::Equal,
        Token::Num("15".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn boundary_with_pub_input() {
    let source = "enf clk.first = stack_inputs[0]";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Ident("stack_inputs".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn boundary_expression() {
    let source = "enf clk.first = 5 + stack_inputs[3] + 6";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Num("5".to_string()),
        Token::Plus,
        Token::Ident("stack_inputs".to_string()),
        Token::Lsqb,
        Token::Num("3".to_string()),
        Token::Rsqb,
        Token::Plus,
        Token::Num("6".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}
