#[macro_use]
pub(crate) mod span;

pub(crate) use span::{File, Span};

pub(crate) struct CompileError<'file> {
    span: Span<'file>,
    message: String,
    note: Option<String>,
    more: Vec<(Span<'file>, String, Option<String>)>,
}

impl<'file> CompileError<'file> {
    pub(crate) fn new(span: Span<'file>, message: String) -> Self {
        Self { span, message, note: None, more: Vec::new() }
    }

    pub(crate) fn new_with_note(span: Span<'file>, message: String, note: String) -> Self {
        Self { span, message, note: Some(note), more: Vec::new() }
    }

    pub(crate) fn note(mut self, sp: Span<'file>, message: String) -> Self {
        self.more.push((sp, message, None));
        self
    }

    pub(crate) fn note_and(mut self, sp: Span<'file>, message: String, note: String) -> Self {
        self.more.push((sp, message, Some(note)));
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
    use nu_ansi_term::Color::*;
    use nu_ansi_term::Style;
    print_message(&format!("{} at {}: {}", LightRed.bold().paint("error"), LightCyan.paint(e.span.to_string()), Style::new().bold().paint(e.message)), e.span, e.note.as_deref());

    for (sp, message, note) in e.more {
        print_message(&format!("   {} {}", LightCyan.paint(sp.to_string()), Style::new().bold().paint(message)), sp, note.as_deref());
    }
}

fn print_message(message: &str, span: Span, note: Option<&str>) {
    use nu_ansi_term::Color::*;
    use nu_ansi_term::Style;

    let (start_line_nr, start_col) = span::get_lc(span.0, span.1);
    let (end_line_nr, end_col) = span::get_lc(span.0, span.2);
    let line_quote = span.0.contents.lines().nth(start_line_nr - 1).unwrap_or("");

    eprintln!("{}", message);
    eprintln!("{}{}{}", &line_quote[..start_col - 1], LightGreen.bold().paint(&line_quote[start_col - 1..end_col - 1]), &line_quote[end_col - 1..]);
    eprint!("{}{}", " ".repeat(start_col - 1), LightGreen.bold().paint("^".repeat(std::cmp::max(1, end_col - start_col))));
    if start_line_nr != end_line_nr {
        eprint!("...")
    }
    if let Some(note) = note {
        eprint!("{} {}", LightGreen.bold().paint("--"), Style::new().bold().paint(note));
    }
    eprintln!()
}
