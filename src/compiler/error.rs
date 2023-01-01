#[derive(Debug)]
pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) contents: String,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Span<'file>(&'file File, usize, usize);

impl PartialEq for Span<'_> {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}
impl Eq for Span<'_> {}

impl File {
    pub(crate) fn eof_span(&self) -> Span {
        self.span_to_end(self.contents.len())
    }
    pub(crate) fn span(&self, start: usize, end: usize) -> Span {
        Span(self, start, end)
    }
    pub(crate) fn span_to_end(&self, start: usize) -> Span {
        self.span(start, self.contents.len())
    }

    pub(crate) fn load(name: &str) -> Result<File, Box<dyn std::error::Error>> {
        Ok(File { name: name.to_string(), contents: std::fs::read_to_string(name)? })
    }
}

impl<'file> Span<'file> {
    pub(crate) fn slice(&self) -> &'file str {
        &self.0.contents[self.1..self.2]
    }
}

impl<'file> std::ops::Add<Span<'file>> for Span<'file> {
    type Output = Span<'file>;

    fn add(self, rhs: Span<'file>) -> Self::Output {
        assert!(self.0 as *const _ == rhs.0 as *const _, "cannot join two spans from different files");
        Span(self.0, std::cmp::min(self.1, rhs.1), std::cmp::max(self.2, rhs.2))
    }
}

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
        report(self.into())
    }
}

fn report(e: CompileError) {
    // TODO: better error reporting
    eprintln!("error: {}", e.message);
}
