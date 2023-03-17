use rome_diagnostics::adapters::IoError;
use rome_diagnostics::{Advices, Diagnostic, DiagnosticExt, Error, Visit};
use rome_text_size::TextRange;
use std::io;

#[derive(Debug, Default, Diagnostic)]
#[diagnostic(category = "project")]
pub struct ProjectDiagnostic {
    #[location(span)]
    range: Option<TextRange>,
    #[source]
    source: Option<Error>,
    #[verbose_advice]
    verbose_advice: VerboseAdvice,
}

#[derive(Debug, Default)]
pub struct VerboseAdvice {
    diagnostics: Vec<Error>,
}

impl Advices for VerboseAdvice {
    fn record(&self, visitor: &mut dyn Visit) -> io::Result<()> {
        for diagnostics in &self.diagnostics {
            visitor.record_frame(diagnostics.location())?;
        }
        Ok(())
    }
}

impl ProjectDiagnostic {
    pub fn new_failed_deserialization(manifest_path: &str, diagnostics: Vec<Error>) -> Self {
        let mut diagnostic = Self::default();
        diagnostic.verbose_advice = VerboseAdvice {
            diagnostics: diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.with_file_path(manifest_path))
                .collect(),
        };
        diagnostic
    }

    pub fn with_source(mut self, source: Error) -> Self {
        self.source = Some(source);
        self
    }
}

impl From<io::Error> for ProjectDiagnostic {
    fn from(error: io::Error) -> Self {
        Self::default().with_source(Error::from(IoError::from(error)))
    }
}
