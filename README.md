# AirScript

A domain specific language to write AIR constraints for the [Miden VM](https://github.com/maticnetwork/miden/).

NOTE: This project is in the initial stages of development.

## Overview

AirScript is a domain specific language for writing AIR constraints for the STARK proving system. The primary goals of AirScript is to enable writing and auditing AIR constraints without the need to learn a specific programming language (e.g., Rust). The secondary goal is to perform automated optimizations of constraints and to output constraint evaluator code to multiple backends (e.g., Rust, Miden assembly, Solidity etc.).

## Project Structure

The project is organized into several crates like so:
| Crate                  | Description |
| ---------------------- | ----------- |
| [Parser](parser) | Contains parser for the AirScript. The parser is used to parse the constraints written in AirScript into an AST. |

## References
1. [Lalrpop](https://github.com/lalrpop/lalrpop/): Rust parser generator framework.
2. [Logos](https://github.com/maciejhirsz/logos/): Library to help in creating a lexer.