# Parser

This crate contains the parser for AIR DSL.

The purpose of the parser is to parse the constraints written in human friendly AIR DSL language into an AST (Abstract Syntax Tree).

## Generating AST

The parser uses [Logos](https://github.com/maciejhirsz/logos/) to write a custom lexer which is then fed into the parser generated using [LALRPOP](https://github.com/lalrpop/lalrpop/).

To create an AST from a given AIR constraints module, you need to first tokenize it using the lexer and then map the tokens to tokens accepted by the parser which are of type `(usize, Token, usize)` where invalid tokens are stored as ScanError. Then feed the tokens to the parser to generate the corresponding AST.

For example:

```Rust
// tokenize the source string
let lex = Lexer::new(self.source.as_str())
    .spanned()
    .map(Token::to_spanned);
// generate AST
let ast = grammar::SourceParser::new().parse(lex).unwrap();
```