use std::ops::Range;

use tree_sitter::{Language, Node, Parser, Query, QueryCursor, QueryMatches, StreamingIterator, Tree};

use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;

const ITEM_CAPTURE: &str = "item";
// What an item leaves out: its body, and anything else the query marks (e.g. comments).
const LEFT_OUT_CAPTURES: [&str; 2] = ["body", "omit"];

// Picks a source file's public items out with a stack pack's tree-sitter query: each @item capture is an item, shown
// on one line, with any @body or @omit capture inside it left out.
pub struct TreeSitterSignatureExtractor;

impl TreeSitterSignatureExtractor {
    /// The grammars built into otto-bot, by the name a stack pack gives them.
    pub fn get_language(language_name: &str) -> ApplicationResult<Language> {
        match language_name {
            "rust" => Ok(tree_sitter_rust::LANGUAGE.into()),
            _ => Err(ApplicationBaseException::new(
                AgentRunEngineException::InvalidProjectMap,
                format!("The stack pack's project map names a grammar otto-bot doesn't have: {language_name}"),
                ContextualData::new().with("language", language_name),
            )),
        }
    }

    pub fn compile_query(language: &Language, query_source: &str) -> ApplicationResult<Query> {
        let query: Query = Query::new(language, query_source).map_err(|exception: tree_sitter::QueryError| {
            ApplicationBaseException::new(
                AgentRunEngineException::InvalidProjectMap,
                "The stack pack's project map query doesn't compile",
                ContextualData::new().with("errors", exception.to_string()),
            )
        })?;
        if query.capture_index_for_name(ITEM_CAPTURE).is_none() {
            return Err(ApplicationBaseException::new(
                AgentRunEngineException::InvalidProjectMap,
                "The stack pack's project map query captures no @item",
                ContextualData::new(),
            ));
        }
        Ok(query)
    }

    /// The file's items, in the order they appear, each on one line.
    pub fn extract(language: &Language, query: &Query, source: &str) -> ApplicationResult<Vec<String>> {
        let mut parser: Parser = Parser::new();
        parser
            .set_language(language)
            .map_err(|exception: tree_sitter::LanguageError| {
                ApplicationBaseException::new(
                    AgentRunEngineException::InvalidProjectMap,
                    "The stack pack's grammar can't be used",
                    ContextualData::new().with("errors", exception.to_string()),
                )
            })?;
        // Only None when parsing is cancelled or times out, which nothing here asks for: the file has no items.
        let Some(tree) = parser.parse(source, None) else {
            return Ok(Vec::new());
        };
        let tree: &Tree = &tree;

        let item_index: u32 = query
            .capture_index_for_name(ITEM_CAPTURE)
            .expect("checked when the query was compiled");
        let left_out_indexes: Vec<u32> = LEFT_OUT_CAPTURES
            .iter()
            .filter_map(|capture_name: &&str| query.capture_index_for_name(capture_name))
            .collect();

        let mut items: Vec<Range<usize>> = Vec::new();
        let mut left_out: Vec<Range<usize>> = Vec::new();
        let mut query_cursor: QueryCursor = QueryCursor::new();
        let mut query_matches: QueryMatches<'_, '_, '_, &[u8], &[u8]> =
            query_cursor.matches(query, tree.root_node(), source.as_bytes());
        while let Some(query_match) = query_matches.next() {
            items.extend(
                query_match
                    .nodes_for_capture_index(item_index)
                    .map(|item: Node<'_>| item.byte_range()),
            );
            for left_out_index in &left_out_indexes {
                left_out.extend(
                    query_match
                        .nodes_for_capture_index(*left_out_index)
                        .map(|node: Node<'_>| node.byte_range()),
                );
            }
        }
        items.sort_by_key(|item: &Range<usize>| (item.start, item.end));
        items.dedup();
        left_out.sort_by_key(|range: &Range<usize>| (range.start, range.end));

        Ok(items
            .into_iter()
            .map(|item: Range<usize>| {
                TreeSitterSignatureExtractor::to_one_line(&TreeSitterSignatureExtractor::without(
                    source, &item, &left_out,
                ))
            })
            .collect())
    }

    // The item's text without the left-out ranges inside it (sorted by start; one inside another is skipped).
    fn without(source: &str, item: &Range<usize>, left_out: &[Range<usize>]) -> String {
        let mut text: String = String::new();
        let mut cursor: usize = item.start;
        for range in left_out {
            if range.start >= cursor && range.end <= item.end {
                text.push_str(&source[cursor..range.start]);
                cursor = range.end;
            }
        }
        text.push_str(&source[cursor..item.end]);
        text
    }

    // Runs of whitespace (a multi-line signature, a struct's fields) become one space.
    fn to_one_line(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }
}
