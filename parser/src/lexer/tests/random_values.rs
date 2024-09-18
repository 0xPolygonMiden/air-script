use super::{expect_valid_tokenization, Symbol, Token};

#[test]
fn random_values_empty_list() {
    let source = "
random_values {
    rand: []
}";

    let tokens = vec![
        Token::RandomValues,
        Token::LBrace,
        Token::Ident(Symbol::intern("rand")),
        Token::Colon,
        Token::LBracket,
        Token::RBracket,
        Token::RBrace,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn random_values_fixed_list() {
    let source = "
random_values {
    rand: [15]
}";

    let tokens = vec![
        Token::RandomValues,
        Token::LBrace,
        Token::Ident(Symbol::intern("rand")),
        Token::Colon,
        Token::LBracket,
        Token::Num(15),
        Token::RBracket,
        Token::RBrace,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn random_values_ident_vector() {
    let source = "
random_values {
    rand: [a, b[12], c]
}";

    let tokens = vec![
        Token::RandomValues,
        Token::LBrace,
        Token::Ident(Symbol::intern("rand")),
        Token::Colon,
        Token::LBracket,
        Token::Ident(Symbol::intern("a")),
        Token::Comma,
        Token::Ident(Symbol::intern("b")),
        Token::LBracket,
        Token::Num(12),
        Token::RBracket,
        Token::Comma,
        Token::Ident(Symbol::intern("c")),
        Token::RBracket,
        Token::RBrace,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn random_values_index_access() {
    let source = "
    integrity_constraints {
        enf a + $alphas[1] = 0
    }";

    let tokens = vec![
        Token::IntegrityConstraints,
        Token::LBrace,
        Token::Enf,
        Token::Ident(Symbol::intern("a")),
        Token::Plus,
        Token::DeclIdentRef(Symbol::intern("$alphas")),
        Token::LBracket,
        Token::Num(1),
        Token::RBracket,
        Token::Equal,
        Token::Num(0),
        Token::RBrace,
    ];
    expect_valid_tokenization(source, tokens);
}
