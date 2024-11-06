mod constant_propagation;
mod inlining;
mod translate;
mod value_numbering;
mod visitor;

pub use self::constant_propagation::ConstantPropagation;
pub use self::inlining::Inlining;
pub use self::translate::AstToMir;
pub use self::value_numbering::ValueNumbering;
pub use self::visitor::{Visit, VisitContext};

use air_pass::Pass;

pub struct DumpAst;
impl Pass for DumpAst {
    type Input<'a> = air_parser::ast::Program;
    type Output<'a> = air_parser::ast::Program;
    type Error = air_parser::SemanticAnalysisError;

    fn run<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        println!("{}", &input);
        Ok(input)
    }
}
