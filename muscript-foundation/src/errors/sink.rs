use tracing::warn;

use crate::errors::Diagnostic;

/// Diagnostic sink - anything that can collect diagnostics for later display.
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

pub fn pipe_all_diagnostics_into<I>(sink: &mut dyn DiagnosticSink, source: I)
where
    I: IntoIterator<Item = Diagnostic>,
{
    source
        .into_iter()
        .for_each(|diagnostic| sink.emit(diagnostic))
}
