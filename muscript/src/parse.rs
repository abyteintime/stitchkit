use std::rc::Rc;

use muscript_analysis::OwnedSources;
use muscript_foundation::{
    errors::{pipe_all_diagnostics_into, DiagnosticSink},
    source::{SourceFileId, SourceFileSet},
    source_arena::SourceArena,
};
use muscript_lexer::{sources::LexedSources, token::Token, token_stream::TokenSpanCursor, Lexer};
use muscript_syntax::{Parse, Parser};
use tracing::info_span;

pub fn parse_source<T>(
    sources: &mut OwnedSources<'_>,
    id: SourceFileId,
    diagnostics: &mut dyn DiagnosticSink<Token>,
) -> Result<T, muscript_syntax::ParseError>
where
    T: Parse,
{
    let _span = info_span!("parse_source").entered();

    let source_file = sources.source_file_set.get(id);

    let (token_span, lexer_errors) = {
        let _span = info_span!("lex", source_file.filename).entered();
        let lexer = Lexer::new(
            sources.token_arena.build_source_file(id),
            id,
            Rc::clone(&source_file.source),
        );
        lexer.lex()
    };

    let result = {
        let _span = info_span!("parse", source_file.filename).entered();
        let tokens = TokenSpanCursor::new(&sources.token_arena, token_span)
            .expect("token span emitted by lexer must not be empty");
        let mut parser = Parser::new(sources.as_borrowed(), &lexer_errors, tokens, diagnostics);
        parser.parse::<T>()
    };

    result
}
