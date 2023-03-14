use muscript_foundation::errors::Diagnostic;
use tracing::warn;

pub trait DiagnosticSink {
    fn emit(&mut self, diagnostic: Diagnostic);
}

impl DiagnosticSink for () {
    fn emit(&mut self, _: Diagnostic) {}
}

impl DiagnosticSink for Vec<Diagnostic> {
    fn emit(&mut self, diagnostic: Diagnostic) {
        self.push(diagnostic);
    }
}

impl DiagnosticSink for Option<Diagnostic> {
    #[track_caller]
    fn emit(&mut self, new: Diagnostic) {
        *self = self.take().map(|old| {
            if new.severity > old.severity {
                new
            } else {
                warn!("new diagnostic dropped from Option<Diagnostic>");
                old
            }
        });
    }
}
