use super::{expect_valid_tokenization, Token};

// LIST COMPREHENSION VALID TOKENIZATION
// ================================================================================================

#[test]
fn one_iterable_comprehension() {
    let source = "let y = [x for x in x]";
    let tokens = vec![
        Token::Let,
        Token::Ident("y".to_string()),
        Token::Equal,
        Token::LBracket,
        Token::Ident("x".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("x".to_string()),
        Token::RBracket,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn multiple_iterables_comprehension() {
    let source = "let a = [w + x - y - z for (w, x, y, z) in (0..3, x, y[0..3], z[0..3])]";
    let tokens = vec![
        Token::Let,
        Token::Ident("a".to_string()),
        Token::Equal,
        Token::LBracket,
        Token::Ident("w".to_string()),
        Token::Plus,
        Token::Ident("x".to_string()),
        Token::Minus,
        Token::Ident("y".to_string()),
        Token::Minus,
        Token::Ident("z".to_string()),
        Token::For,
        Token::LParen,
        Token::Ident("w".to_string()),
        Token::Comma,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::Comma,
        Token::Ident("z".to_string()),
        Token::RParen,
        Token::In,
        Token::LParen,
        Token::Num(0),
        Token::DotDot,
        Token::Num(3),
        Token::Comma,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::DotDot,
        Token::Num(3),
        Token::RBracket,
        Token::Comma,
        Token::Ident("z".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::DotDot,
        Token::Num(3),
        Token::RBracket,
        Token::RParen,
        Token::RBracket,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn one_iterable_folding() {
    let source = "let y = sum([x for x in x])";
    let tokens = vec![
        Token::Let,
        Token::Ident("y".to_string()),
        Token::Equal,
        Token::Sum,
        Token::LParen,
        Token::LBracket,
        Token::Ident("x".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("x".to_string()),
        Token::RBracket,
        Token::RParen,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn multiple_iterables_list_folding() {
    let source = "let a = sum([w + x - y - z for (w, x, y, z) in (0..3, x, y[0..3], z[0..3])])";
    let tokens = vec![
        Token::Let,
        Token::Ident("a".to_string()),
        Token::Equal,
        Token::Sum,
        Token::LParen,
        Token::LBracket,
        Token::Ident("w".to_string()),
        Token::Plus,
        Token::Ident("x".to_string()),
        Token::Minus,
        Token::Ident("y".to_string()),
        Token::Minus,
        Token::Ident("z".to_string()),
        Token::For,
        Token::LParen,
        Token::Ident("w".to_string()),
        Token::Comma,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::Comma,
        Token::Ident("z".to_string()),
        Token::RParen,
        Token::In,
        Token::LParen,
        Token::Num(0),
        Token::DotDot,
        Token::Num(3),
        Token::Comma,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::DotDot,
        Token::Num(3),
        Token::RBracket,
        Token::Comma,
        Token::Ident("z".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::DotDot,
        Token::Num(3),
        Token::RBracket,
        Token::RParen,
        Token::RBracket,
        Token::RParen,
    ];
    expect_valid_tokenization(source, tokens);
}
