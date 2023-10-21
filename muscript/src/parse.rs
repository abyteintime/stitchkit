use std::rc::Rc;

use muscript_foundation::{errors::DiagnosticSink, source::SourceFileId};
use muscript_lexer::{
    sliced_tokens::SlicedTokens, sources::OwnedSources, token::Token,
    token_stream::TokenSpanCursor, Lexer,
};
use muscript_preprocessor::{Definitions, Preprocessor};
use muscript_syntax::{Parse, Parser};
use tracing::info_span;

pub fn parse_source<T>(
    sources: &mut OwnedSources<'_>,
    definitions: &mut Definitions,
    id: SourceFileId,
    diagnostics: &mut dyn DiagnosticSink<Token>,
) -> Result<T, muscript_syntax::ParseError>
where
    T: Parse,
{
    let source_file = sources.source_file_set.get(id);
    let _span = info_span!("parse_source", source_file.filename).entered();

    let token_span = {
        let _span = info_span!("lex").entered();
        let lexer = Lexer::new(
            sources.token_arena.build_source_file(id),
            id,
            Rc::clone(&source_file.source),
            &mut sources.lexer_errors,
        );
        lexer.lex()
    };

    let preprocessed = {
        let _span = info_span!("preprocess").entered();
        let mut preprocessed = SlicedTokens::new();
        let mut preprocessor = Preprocessor::new(
            definitions,
            sources.as_borrowed(),
            TokenSpanCursor::new(&sources.token_arena, token_span)
                .expect("token span emitted by lexer must not be empty"),
            &mut preprocessed,
            diagnostics,
        );
        preprocessor.preprocess();
        preprocessed
    };

    let result = {
        let _span = info_span!("parse").entered();
        let tokens = preprocessed
            .stream(&sources.token_arena)
            .expect("token slices emitted by preprocessor must not be empty");
        let mut parser = Parser::new(sources.as_borrowed(), tokens, diagnostics);
        parser.parse::<T>()
    };

    result
}
