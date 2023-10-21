use tracing::warn;

use crate::errors::Diagnostic;

/// Diagnostic sink - anything that can collect diagnostics for later display.
pub trait DiagnosticSink<T> {
    fn emit(&mut self, diagnostic: Diagnostic<T>);
}

impl<T> DiagnosticSink<T> for () {
    fn emit(&mut self, _: Diagnostic<T>) {}
}

impl<T> DiagnosticSink<T> for Vec<Diagnostic<T>> {
    fn emit(&mut self, diagnostic: Diagnostic<T>) {
        self.push(diagnostic);
    }
}

impl<T> DiagnosticSink<T> for Option<Diagnostic<T>> {
    #[track_caller]
    fn emit(&mut self, new: Diagnostic<T>) {
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

pub fn pipe_all_diagnostics_into<T, I>(sink: &mut dyn DiagnosticSink<T>, source: I)
where
    I: IntoIterator<Item = Diagnostic<T>>,
{
    source
        .into_iter()
        .for_each(|diagnostic| sink.emit(diagnostic))
}
