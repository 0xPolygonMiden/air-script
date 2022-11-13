use super::{expect_valid_tokenization, Token};

#[test]
fn constants_scalar() {
    let source = "
constants:
    a: 1
    b: 2";

    let tokens = vec![
        Token::Constants,
        Token::Colon,
        Token::Ident("a".to_string()),
        Token::Colon,
        Token::Num("1".to_string()),
        Token::Ident("b".to_string()),
        Token::Colon,
        Token::Num("2".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn constants_vector() {
    let source = "
constants:
    a: [1, 2, 3, 4]
    b: [5, 6, 7, 8]";

    let tokens = vec![
        Token::Constants,
        Token::Colon,
        Token::Ident("a".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Comma,
        Token::Num("2".to_string()),
        Token::Comma,
        Token::Num("3".to_string()),
        Token::Comma,
        Token::Num("4".to_string()),
        Token::Rsqb,
        Token::Ident("b".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Num("5".to_string()),
        Token::Comma,
        Token::Num("6".to_string()),
        Token::Comma,
        Token::Num("7".to_string()),
        Token::Comma,
        Token::Num("8".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn constants_matrix() {
    let source = "
    constants:
        a: [[1, 2], [3, 4]]
        b: [[5, 6], [7, 8]]";

    let tokens = vec![
        Token::Constants,
        Token::Colon,
        Token::Ident("a".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Comma,
        Token::Num("2".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Lsqb,
        Token::Num("3".to_string()),
        Token::Comma,
        Token::Num("4".to_string()),
        Token::Rsqb,
        Token::Rsqb,
        Token::Ident("b".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Lsqb,
        Token::Num("5".to_string()),
        Token::Comma,
        Token::Num("6".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Lsqb,
        Token::Num("7".to_string()),
        Token::Comma,
        Token::Num("8".to_string()),
        Token::Rsqb,
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn constants_access_inside_boundary_expr() {
    let source = "
    boundary_constraints:
        enf clk.first = a + b[0]
        enf clk.last = c[0][1]
    ";

    let tokens = vec![
        Token::BoundaryConstraints,
        Token::Colon,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Ident("a".to_string()),
        Token::Plus,
        Token::Ident("b".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::Last,
        Token::Equal,
        Token::Ident("c".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn constants_access_inside_transition_expr() {
    let source = "
        transition_constraints:
            enf clk * 2^a = b[0] + c[0][1]
    ";
    let tokens = vec![
        Token::TransitionConstraints,
        Token::Colon,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Mul,
        Token::Num("2".to_string()),
        Token::Exp,
        Token::Ident("a".to_string()),
        Token::Equal,
        Token::Ident("b".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Plus,
        Token::Ident("c".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}
