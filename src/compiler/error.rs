#[macro_use]
pub(crate) mod span;

pub(crate) use span::{File, Span};

pub(crate) struct CompileError<'file> {
    span: Span<'file>,
    message: String,
}

impl<'file> CompileError<'file> {
    pub(crate) fn new(span: Span<'file>, message: String) -> Self {
        Self { span, message }
    }
}

pub(crate) trait Report {
    fn report(self);
}

impl<'a, T: Into<CompileError<'a>>> Report for T {
    fn report(self) {
        report(self.into());
    }
}

fn report(e: CompileError) {
    // TODO: better error reporting
    eprintln!("error at {}: {}", e.span, e.message);
}
