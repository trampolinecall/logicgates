#[macro_use]
pub(crate) mod span;

pub(crate) use span::{File, Span};

pub(crate) struct CompileError<'file> {
    span: Span<'file>,
    message: String,
    notes: Vec<(Span<'file>, String)>,
}

impl<'file> CompileError<'file> {
    pub(crate) fn new(span: Span<'file>, message: String) -> Self {
        Self { span, message, notes: Vec::new() }
    }

    pub(crate) fn note(mut self, sp: Span<'file>, note: String) -> Self {
        self.notes.push((sp, note));
        self
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
    print_message(&format!("error at {}: {}", e.span, e.message), e.span);

    for (sp, note) in e.notes {
        print_message(&format!("  - {}", note), sp);
    }
}

fn print_message(message: &str, span: Span) {
    let (start_line_nr, start_col) = span::get_lc(span.0, span.1);
    let (end_line_nr, end_col) = span::get_lc(span.0, span.2);
    let line_quote = span.0.contents.lines().nth(start_line_nr - 1).unwrap_or("");

    eprintln!("{}", message);
    eprintln!("{}", line_quote);
    eprint!("{}{}", " ".repeat(start_col - 1), "^".repeat(end_col - start_col));
    if start_line_nr != end_line_nr {
        eprint!("...")
    }
    eprintln!()
}
