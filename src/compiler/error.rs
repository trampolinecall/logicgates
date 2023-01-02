// TODO: reorganize this module

#[derive(Debug)]
pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) contents: String,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Span<'file>(&'file File, usize, usize);

impl PartialEq for Span<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0) && self.1 == other.1 && self.2 == other.2
    }
}
impl Eq for Span<'_> {}

#[cfg(test)]
macro_rules! make_spans {
    ($file:expr, [$(($str:literal, $sp:ident => $thing:expr)),* $(,)?], $eof_span:ident => $eof_thing:expr $(,)?) => {
        {
            $file.contents = {
                let mut contents = String::new();
                $(
                    contents += $str;
                )*
                contents
            };


            let mut things = Vec::new();
            let mut cur_idx = 0;

            $(
                {
                    let $sp = $file.span(cur_idx, cur_idx + $str.len());
                    things.push($thing);
                    #[allow(unused_assignments)]
                    (cur_idx += $str.len());
                }
            )*

            let $eof_span = $file.eof_span();
            things.push($eof_thing);

            things
        }
    }
}

impl File {
    #[cfg(test)]
    pub(crate) fn test_file() -> File {
        File { name: "<test file>".into(), contents: "".into() }
    }

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

impl<'file> std::ops::Add<Span<'file>> for Span<'file> {
    type Output = Span<'file>;

    fn add(self, rhs: Span<'file>) -> Self::Output {
        assert!(std::ptr::eq(self.0, rhs.0), "cannot join two spans from different files");
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
