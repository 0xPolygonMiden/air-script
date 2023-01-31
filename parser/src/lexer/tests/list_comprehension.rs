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
        Token::Lsqb,
        Token::Ident("x".to_string()),
        Token::For,
        Token::Ident("x".to_string()),
        Token::In,
        Token::Ident("x".to_string()),
        Token::Rsqb,
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
        Token::Lsqb,
        Token::Ident("w".to_string()),
        Token::Plus,
        Token::Ident("x".to_string()),
        Token::Minus,
        Token::Ident("y".to_string()),
        Token::Minus,
        Token::Ident("z".to_string()),
        Token::For,
        Token::Lparen,
        Token::Ident("w".to_string()),
        Token::Comma,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::Comma,
        Token::Ident("z".to_string()),
        Token::Rparen,
        Token::In,
        Token::Lparen,
        Token::Num("0".to_string()),
        Token::Range,
        Token::Num("3".to_string()),
        Token::Comma,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Range,
        Token::Num("3".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Ident("z".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Range,
        Token::Num("3".to_string()),
        Token::Rsqb,
        Token::Rparen,
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}
