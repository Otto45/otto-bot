use tree_sitter::{Language, Node, Parser, Query, QueryCapture, QueryCaptures, QueryCursor, StreamingIterator, Tree};

use crate::managers::common::tree_sitter::captured_node::CapturedNode;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;

// Runs a stack pack's tree-sitter queries on source files, with the grammars built into otto-bot. Parsing is CPU work
// on files already read: no I/O.
pub struct TreeSitterQueryRunner;

impl TreeSitterQueryRunner {
    /// The grammar a stack pack names.
    pub fn get_language(language_name: &str) -> ApplicationResult<Language> {
        match language_name {
            "rust" => Ok(tree_sitter_rust::LANGUAGE.into()),
            "python" => Ok(tree_sitter_python::LANGUAGE.into()),
            // TypeScript and TSX are two grammars: TSX parses JSX, which plain TypeScript's type assertions rule out.
            "typescript" => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            "tsx" => Ok(tree_sitter_typescript::LANGUAGE_TSX.into()),
            _ => Err(ApplicationBaseException::new(
                ExceptionKind::InvalidStackPack,
                format!("The stack pack names a grammar otto-bot doesn't have: {language_name}"),
                ContextualData::new().with("language", language_name),
            )),
        }
    }

    /// `query_name` says which of the pack's queries it is, if it doesn't compile.
    pub fn compile_query(language: &Language, query_source: &str, query_name: &str) -> ApplicationResult<Query> {
        Query::new(language, query_source).map_err(|exception: tree_sitter::QueryError| {
            ApplicationBaseException::new(
                ExceptionKind::InvalidStackPack,
                format!("The stack pack's {query_name} query doesn't compile"),
                ContextualData::new()
                    .with("query_name", query_name)
                    .with("errors", exception.to_string()),
            )
        })
    }

    /// The source's syntax tree; None only if parsing is cancelled, which nothing here asks for.
    pub fn parse(language: &Language, source: &str) -> ApplicationResult<Option<Tree>> {
        let mut parser: Parser = Parser::new();
        parser
            .set_language(language)
            .map_err(|exception: tree_sitter::LanguageError| {
                ApplicationBaseException::new(
                    ExceptionKind::InvalidStackPack,
                    "The stack pack's grammar can't be used",
                    ContextualData::new().with("errors", exception.to_string()),
                )
            })?;
        Ok(parser.parse(source, None))
    }

    /// Every node the query captures in the tree, in the order they appear, with text predicates (`#eq?`, `#match?`)
    /// applied.
    pub fn capture(query: &Query, tree: &Tree, source: &str) -> Vec<CapturedNode> {
        let capture_names: &[&str] = query.capture_names();
        let mut captured_nodes: Vec<CapturedNode> = Vec::new();
        let mut query_cursor: QueryCursor = QueryCursor::new();
        let mut query_captures: QueryCaptures<'_, '_, '_, &[u8], &[u8]> =
            query_cursor.captures(query, tree.root_node(), source.as_bytes());
        while let Some((query_match, capture_index)) = query_captures.next() {
            let query_capture: &QueryCapture<'_> = &query_match.captures()[*capture_index];
            let node: Node<'_> = query_capture.node;
            captured_nodes.push(CapturedNode {
                capture_name: capture_names[query_capture.index as usize].to_string(),
                byte_range: node.byte_range(),
                line_number: i64::try_from(node.start_position().row + 1).unwrap_or(i64::MAX),
            });
        }
        captured_nodes
    }
}
