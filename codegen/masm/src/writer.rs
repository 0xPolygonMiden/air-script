use miden_processor::math::{Felt, StarkField};
use std::borrow::{Borrow, Cow};

#[derive(Debug, Clone, Copy)]
enum ControlFlow {
    While,
}

impl std::fmt::Display for ControlFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFlow::While => write!(f, "while"),
        }
    }
}

#[derive(Default)]
enum LineState {
    #[default]
    Start,
    Idented,
    Instructions,
    Comment,
}

/// A MASM assembly writer.
///
/// This struct helps to detect errors by tracking state of the code generation, and also helps to
/// produce more readable code with automatic alignining.
#[derive(Default)]
pub struct Writer {
    code: String,
    procedure: Option<Cow<'static, str>>,
    stack: Vec<ControlFlow>,
    state: LineState,
}

macro_rules! simple_ins {
    ($ins: ident) => {
        pub fn $ins(&mut self) {
            self.ins(stringify!($ins));
        }
    };
}

impl Writer {
    const INDENT: usize = 4;

    pub fn new() -> Self {
        Self::default()
    }

    /// Consume the [Writer] and returns the generated code.
    ///
    /// Panics:
    /// - If there are unclosed control flows.
    pub fn into_code(self) -> String {
        assert!(self.stack.is_empty(), "Unclosed control flows");
        assert!(self.procedure.is_none(), "Unclosed procedure");
        self.code
    }

    pub fn new_line(&mut self) {
        self.code.push('\n');
        self.state = LineState::Start;
    }

    fn maybe_new_line_and_indent(&mut self) {
        match self.state {
            LineState::Start => self.indent(),
            LineState::Idented => {}
            LineState::Comment | LineState::Instructions => {
                self.new_line();
                self.indent();
            }
        }
    }

    fn spaces(&mut self, count: usize) {
        self.code.push_str(&" ".repeat(count));
    }

    fn indent(&mut self) {
        let count = self.stack.len() + (self.procedure.is_some() as usize);
        self.spaces(count * Self::INDENT);
        self.state = LineState::Idented;
    }

    /// Ensures the comment is on a new line.
    pub fn header(&mut self, comment: impl Borrow<str>) {
        self.maybe_new_line_and_indent();
        self.comment(comment.borrow());
        self.new_line();
    }

    /// Add a comment at the end of the current line and break it.
    pub fn comment(&mut self, comment: impl Borrow<str>) {
        match self.state {
            LineState::Start => self.indent(),
            LineState::Idented => {}
            LineState::Instructions => self.spaces(1),
            LineState::Comment => {
                self.new_line();
                self.indent();
            }
        }
        self.code.push_str("# ");
        self.code.push_str(comment.borrow());
        self.state = LineState::Comment;
    }

    // CONTROL FLOW
    // -------------------------------------------------------------------------------------------

    /// Starts the codegen for a procedure.
    pub fn proc(&mut self, name: impl Into<Cow<'static, str>>) {
        assert!(
            self.procedure.is_none(),
            "Can not nest procedures, currently writing {:?}",
            self.procedure
        );

        let name = name.into();
        self.code.push_str(&format!("proc.{}", name));
        self.procedure = Some(name);
        self.new_line();
    }

    pub fn exec(&mut self, name: impl Into<Cow<'static, str>>) {
        self.maybe_new_line_and_indent();
        self.code.push_str(&format!("exec.{}", name.into()));
        self.new_line();
    }

    /// Ends a control block.
    ///
    /// The blocks can be a while/if/else or proc.
    pub fn end(&mut self) {
        assert!(
            self.procedure.is_some(),
            "Can not end outside of a procedure"
        );

        if let Some(flow) = self.stack.pop() {
            self.maybe_new_line_and_indent();
            self.code.push_str("end");
            self.state = LineState::Instructions;
            self.comment(format!("END {}", flow));
            self.new_line();
        } else if let Some(proc) = &self.procedure.take() {
            self.maybe_new_line_and_indent();
            self.code.push_str("end");
            self.state = LineState::Instructions;
            self.comment(format!("END PROC {}", proc));
            self.new_line();
            self.new_line();
        }
    }

    // INSTRUCTIONS
    // -------------------------------------------------------------------------------------------

    /// Adds an instruciton to the code.
    fn ins(&mut self, ins: impl Into<Cow<'static, str>>) {
        assert!(
            self.procedure.is_some(),
            "Can not write instructions outside of a procedure"
        );
        match self.state {
            LineState::Start => self.indent(),
            LineState::Idented => {}
            LineState::Instructions => self.spaces(1),
            LineState::Comment => {
                self.new_line();
                self.indent();
            }
        }

        self.state = LineState::Instructions;
        let ins = ins.into();
        self.code.push_str(&ins);
    }

    simple_ins!(drop);
    simple_ins!(dropw);
    simple_ins!(padw);
    simple_ins!(ext2mul);
    simple_ins!(ext2add);
    simple_ins!(ext2sub);
    simple_ins!(neg);
    simple_ins!(swap);
    simple_ins!(div);
    simple_ins!(mul);

    pub(crate) fn add(&mut self, arg: u64) {
        self.ins(format!("add.{}", arg));
    }

    pub fn dup(&mut self, arg: u64) {
        assert!(
            arg <= 15,
            "dup.15 is the highest supported value, got {}",
            arg
        );

        self.ins(format!("dup.{}", arg));
    }

    pub fn mem_load(&mut self, address: u32) {
        self.ins(format!("mem_load.{}", address));
    }

    pub fn mem_loadw(&mut self, address: u32) {
        self.ins(format!("mem_loadw.{}", address));
    }

    pub fn mem_storew(&mut self, address: u32) {
        self.ins(format!("mem_storew.{}", address));
    }

    pub fn movup(&mut self, arg: i32) {
        assert!(arg != 0, "movdn.0 is a noop");
        assert!(arg != 1, "use swap instead of movdn.1");
        assert!(
            arg < 16,
            "movup.15 is the highest supported value, got {}",
            arg
        );

        self.ins(format!("movup.{}", arg));
    }

    pub fn movdn(&mut self, arg: u64) {
        assert!(arg != 0, "movdn.0 is a noop");
        assert!(arg != 1, "use swap instead of movdn.1");
        assert!(
            arg < 16,
            "movdn.15 is the highest supported value, got {}",
            arg
        );

        self.ins(format!("movdn.{}", arg));
    }

    pub fn push(&mut self, arg: u64) {
        assert!(
            arg < Felt::MODULUS,
            "Value is larger than modulus, likely a bug"
        );
        self.ins(format!("push.{}", arg));
    }

    pub fn neq(&mut self, arg: u64) {
        self.ins(format!("neq.{}", arg));
    }

    pub fn r#while(&mut self) {
        assert!(
            self.procedure.is_some(),
            "Can not open a while outside of a procedure"
        );
        self.new_line();
        self.indent();
        self.code.push_str("while.true");
        self.new_line();
        self.stack.push(ControlFlow::While);
    }
}
