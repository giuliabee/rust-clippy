use clippy_utils::diagnostics::span_lint;
use clippy_utils::source::SpanRangeExt;
use itertools::Itertools;
use rustc_ast::ast::*;
use rustc_lexer::{TokenKind, tokenize};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::declare_lint_pass;
use rustc_span::{BytePos, Pos, Span, SyntaxContext};
use std::ops::Not;

declare_clippy_lint! {
    /// ### What it does
    /// Checks comments for leftover TODOs and FIXMEs.
    ///
    /// ### Why is this bad?
    /// If any comments are still marked as TODO or FIXME, code might not be ready for production yet.
    ///
    /// ### Example
    /// ```no_run
    /// // TODO write a better error for this function
    /// fn do_something(){
    /// panic!("Generic error")}
    /// ```
    #[clippy::version = "1.83.0"]
    pub TODO_IN_COMMENTS,
    pedantic    ,
    "detects TODO and FIXME comments"
}

declare_lint_pass!(TodoInComments => [TODO_IN_COMMENTS]);
const TODO: &str = "TODO";
const FIXME: &str = "FIXME";
impl EarlyLintPass for TodoInComments {
    fn check_crate(&mut self, ctx: &EarlyContext<'_>, crt: &Crate) {
        let Some(source_range) = &crt.spans.inner_span.get_source_range(ctx) else {
            return;
        };

        let source_path = source_range.sf.name.clone();

        let Some(source_file) = ctx.sess().source_map().get_source_file(&source_path) else {
            return;
        };

        let Some(source_code) = &source_file.src else {
            return;
        };

        let tokens = tokenize(source_code.as_str());

        let mut cur_pos: usize = 0;

        for token in tokens {
            match token.kind {
                TokenKind::LineComment { .. } | TokenKind::BlockComment { .. } => {
                    let comment = &source_code[cur_pos..cur_pos + token.len as usize];

                    let indices = comment
                        .to_ascii_uppercase()
                        .match_indices(TODO)
                        .map(|(index, _)| index)
                        .collect_vec();

                    if indices.is_empty().not() {
                        let mut spans = vec![Span::new(
                            source_file.start_pos + BytePos::from_usize(cur_pos),
                            source_file.start_pos + BytePos::from_usize(cur_pos + token.len as usize),
                            SyntaxContext::root(),
                            None,
                        )];

                        for index in indices {
                            spans.push(Span::new(
                                source_file.start_pos + BytePos::from_usize(cur_pos + index),
                                source_file.start_pos + BytePos::from_usize(cur_pos + index + TODO.len()),
                                SyntaxContext::root(),
                                None,
                            ))
                        }

                        span_lint(ctx, TODO_IN_COMMENTS, spans, "TODO found in comment");
                    }
                    if comment.to_ascii_uppercase().contains(FIXME) {
                        span_lint(
                            ctx,
                            TODO_IN_COMMENTS,
                            vec![Span::new(
                                source_file.start_pos + BytePos::from_usize(cur_pos),
                                source_file.start_pos + BytePos::from_usize(cur_pos + token.len as usize),
                                SyntaxContext::root(),
                                None,
                            )],
                            "FIXME found in comment",
                        );
                    }
                },
                _ => {},
            }

            cur_pos += token.len as usize;
        }
    }
}
