use muscript_foundation::source::SourceFileId;
use muscript_syntax::lexis::token::Ident;

#[derive(Debug, Clone)]
pub struct Var {
    pub source_file_id: SourceFileId,
    pub name: Ident,
}
