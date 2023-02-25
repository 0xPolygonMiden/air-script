use super::{expect_valid_tokenization, Token};

// FUNCTIONS VALID TOKENIZATION
// ================================================================================================

#[test]
fn one_arg_function_definition() {
    let source = "
    fn apply_mds(state: vector[12]) -> vector[12]:
    return [
      sum([m * s for (m, s) in (mds_row, state)])
      for mds_row in mds
    ]
    ";
    let tokens = [
        Token::Function,
        Token::Ident("apply_mds".to_string()),
        Token::Lparen,
        Token::Ident("state".to_string()),
        Token::Colon,
        Token::Vector,
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Rparen,
        Token::Rarrow,
        Token::Vector,
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Colon,
        Token::Return,
        Token::Lsqb,
        Token::Sum,
        Token::Lparen,
        Token::Lsqb,
        Token::Ident("m".to_string()),
        Token::Mul,
        Token::Ident("s".to_string()),
        Token::For,
        Token::Lparen,
        Token::Ident("m".to_string()),
        Token::Comma,
        Token::Ident("s".to_string()),
        Token::Rparen,
        Token::In,
        Token::Lparen,
        Token::Ident("mds_row".to_string()),
        Token::Comma,
        Token::Ident("state".to_string()),
        Token::Rparen,
        Token::Rsqb,
        Token::Rparen,
        Token::For,
        Token::Ident("mds_row".to_string()),
        Token::In,
        Token::Ident("mds".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens.to_vec());
}

#[test]
fn multiple_args_function_definition() {
    let source = "
    fn apply_mds(a: scalar, b: vector[12], c: matrix[6][6]) -> (scalar, vector[12], matrix[6][6]):
        return (
            sum([m * a for m in mds_row]),
            [
                sum([m * s for (m, s) in (mds_row, b)])
                for mds_row in mds
            ],
            [
                [
                    sum([m * s for (m, s) in (mds_row, c_row)])
                    for mds_row in mds
                ]
                for c_row in c
            ]
        )
    ";
    let tokens = [
        Token::Function,
        Token::Ident("apply_mds".to_string()),
        Token::Lparen,
        Token::Ident("a".to_string()),
        Token::Colon,
        Token::Scalar,
        Token::Comma,
        Token::Ident("b".to_string()),
        Token::Colon,
        Token::Vector,
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Ident("c".to_string()),
        Token::Colon,
        Token::Matrix,
        Token::Lsqb,
        Token::Num("6".to_string()),
        Token::Rsqb,
        Token::Lsqb,
        Token::Num("6".to_string()),
        Token::Rsqb,
        Token::Rparen,
        Token::Rarrow,
        Token::Lparen,
        Token::Scalar,
        Token::Comma,
        Token::Vector,
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Matrix,
        Token::Lsqb,
        Token::Num("6".to_string()),
        Token::Rsqb,
        Token::Lsqb,
        Token::Num("6".to_string()),
        Token::Rsqb,
        Token::Rparen,
        Token::Colon,
        Token::Return,
        Token::Lparen,
        Token::Sum,
        Token::Lparen,
        Token::Lsqb,
        Token::Ident("m".to_string()),
        Token::Mul,
        Token::Ident("a".to_string()),
        Token::For,
        Token::Ident("m".to_string()),
        Token::In,
        Token::Ident("mds_row".to_string()),
        Token::Rsqb,
        Token::Rparen,
        Token::Comma,
        Token::Lsqb,
        Token::Sum,
        Token::Lparen,
        Token::Lsqb,
        Token::Ident("m".to_string()),
        Token::Mul,
        Token::Ident("s".to_string()),
        Token::For,
        Token::Lparen,
        Token::Ident("m".to_string()),
        Token::Comma,
        Token::Ident("s".to_string()),
        Token::Rparen,
        Token::In,
        Token::Lparen,
        Token::Ident("mds_row".to_string()),
        Token::Comma,
        Token::Ident("b".to_string()),
        Token::Rparen,
        Token::Rsqb,
        Token::Rparen,
        Token::For,
        Token::Ident("mds_row".to_string()),
        Token::In,
        Token::Ident("mds".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Lsqb,
        Token::Lsqb,
        Token::Sum,
        Token::Lparen,
        Token::Lsqb,
        Token::Ident("m".to_string()),
        Token::Mul,
        Token::Ident("s".to_string()),
        Token::For,
        Token::Lparen,
        Token::Ident("m".to_string()),
        Token::Comma,
        Token::Ident("s".to_string()),
        Token::Rparen,
        Token::In,
        Token::Lparen,
        Token::Ident("mds_row".to_string()),
        Token::Comma,
        Token::Ident("c_row".to_string()),
        Token::Rparen,
        Token::Rsqb,
        Token::Rparen,
        Token::For,
        Token::Ident("mds_row".to_string()),
        Token::In,
        Token::Ident("mds".to_string()),
        Token::Rsqb,
        Token::For,
        Token::Ident("c_row".to_string()),
        Token::In,
        Token::Ident("c".to_string()),
        Token::Rsqb,
        Token::Rparen,
    ];
    expect_valid_tokenization(source, tokens.to_vec());
}
