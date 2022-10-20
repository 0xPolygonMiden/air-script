# Parser

This crate contains the parser for AIR DSL.

The purpose of the parser is to parse the constraints written in human friendly AIR DSL language into an AST (Abstract Syntax Tree).

## Generating AST

The parser uses [Logos](https://github.com/maciejhirsz/logos/) to write a custom lexer which is then fed into the parser generated using [LALRPOP](https://github.com/lalrpop/lalrpop/).

To create an AST from a given AIR constraints module, you just need to pass your source to the public `parse` function, which will return the AST or an `Error` of type `ScanError` or `ParseError`. `parse` will first tokenize the source using the lexer, then map the tokens to tokens accepted by the parser which are of type `(usize, Token, usize)`. Invalid tokens will be stored as `ScanError`. Finally, `parse` feeds the tokens to the parser to generate the corresponding AST (or `ParseError`).

For example:

```Rust
// parse the source string to an Result containing the AST or an Error
let ast = parse(source.as_str());
```
