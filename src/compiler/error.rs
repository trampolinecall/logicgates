pub(crate) struct CompileError {
    pub(crate) message: String,
}

pub(crate) trait Report {
    fn report(self);
}

impl<T: Into<CompileError>> Report for T {
    fn report(self) {
        report(self.into())
    }
}

fn report(e: CompileError) {
    eprintln!("error: {}", e.message);
}
