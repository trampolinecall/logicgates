#[derive(Debug)]
pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) contents: String,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Span<'file>(pub(super) &'file File, pub(super) usize, pub(super) usize);

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

            let mut cur_idx = 0;
            let things = [
                $(
                    {
                        let $sp = $file.span(cur_idx, cur_idx + $str.len());
                        #[allow(unused_assignments)]
                        (cur_idx += $str.len());
                        $thing
                    },
                )*
                {
                    let $eof_span = $file.eof_span();
                    $eof_thing
                },
            ];

            things
        }
    }
}

pub(super) fn get_lc(file: &File, ind: usize) -> (usize, usize) {
    // TODO: handle grapheme clusters correctly, especially in col
    let line = file.contents[..ind].chars().filter(|&c| c == '\n').count() + 1;
    let col = file.contents[..ind].chars().rev().take_while(|&c| c != '\n').count() + 1;

    (line, col)
}

impl<'file> std::fmt::Display for Span<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start_l, start_c) = get_lc(self.0, self.1);
        let (end_l, end_c) = get_lc(self.0, self.2);
        write!(f, "{}:({}:{} - {}:{})", self.0.name, start_l, start_c, end_l, end_c)
    }
}

impl<'file> std::ops::Add<Span<'file>> for Span<'file> {
    type Output = Span<'file>;

    fn add(self, rhs: Span<'file>) -> Self::Output {
        assert!(std::ptr::eq(self.0, rhs.0), "cannot join two spans from different files");
        Span(self.0, std::cmp::min(self.1, rhs.1), std::cmp::max(self.2, rhs.2))
    }
}

