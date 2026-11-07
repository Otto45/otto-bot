use crate::managers::engines::agent_run::processors::patch::diff_appliers::v4a_chunk::V4aChunk;
use crate::managers::engines::agent_run::processors::patch::diff_appliers::v4a_section::V4aSection;
use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;

const END_PATCH: &str = "*** End Patch";
const END_FILE: &str = "*** End of File";
const SECTION_TERMINATORS: [&str; 4] = [END_PATCH, "*** Update File:", "*** Delete File:", "*** Add File:"];
const END_SECTION_MARKERS: [&str; 5] = [
    END_PATCH,
    "*** Update File:",
    "*** Delete File:",
    "*** Add File:",
    END_FILE,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineMode {
    Keep,
    Add,
    Delete,
}

// Applies a V4A diff, the format of the apply_patch tool's operations. A port of the reference applier of OpenAI's
// Agents SDK (`apply_diff`): the same parsing, the same three levels of context matching (exact, then ignoring
// trailing whitespace, then ignoring surrounding whitespace), and the same error messages, which the model knows how
// to act on. The one difference: a created file ends with a newline, which the reference leaves off.
pub struct V4aDiffApplier;

impl V4aDiffApplier {
    /// A new file's content, from a create_file diff: every line starts with "+".
    pub fn apply_create(diff: &str) -> ApplicationResult<String> {
        let newline: &str = if diff.contains("\r\n") { "\r\n" } else { "\n" };
        let mut lines: Vec<String> = V4aDiffApplier::normalize_diff_lines(diff);
        lines.push(END_PATCH.to_string());

        let mut output: Vec<String> = Vec::new();
        let mut index: usize = 0;
        while !V4aDiffApplier::is_done(&lines, index, &SECTION_TERMINATORS) {
            let line: &str = &lines[index];
            index += 1;
            match line.strip_prefix('+') {
                Some(content) => output.push(content.to_string()),
                None => return Err(V4aDiffApplier::to_exception(format!("Invalid Add File Line: {line}"))),
            }
        }

        let mut content: String = output.join(newline);
        if !content.is_empty() {
            content.push_str(newline);
        }
        Ok(content)
    }

    /// The file's new content, from its current content and an update_file diff.
    pub fn apply_update(input: &str, diff: &str) -> ApplicationResult<String> {
        let newline: &str = if input.contains('\n') {
            if input.contains("\r\n") { "\r\n" } else { "\n" }
        } else if diff.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let diff_lines: Vec<String> = V4aDiffApplier::normalize_diff_lines(diff);
        let normalized_input: String = input.replace("\r\n", "\n");
        let chunks: Vec<V4aChunk> = V4aDiffApplier::parse_update_diff(diff_lines, &normalized_input)?;
        V4aDiffApplier::apply_chunks(&normalized_input, &chunks, newline)
    }

    fn normalize_diff_lines(diff: &str) -> Vec<String> {
        let mut lines: Vec<String> = diff
            .split('\n')
            .map(|line: &str| line.strip_suffix('\r').unwrap_or(line).to_string())
            .collect();
        if lines.last().is_some_and(|line: &String| line.is_empty()) {
            lines.pop();
        }
        lines
    }

    fn is_done(lines: &[String], index: usize, prefixes: &[&str]) -> bool {
        match lines.get(index) {
            None => true,
            Some(line) => prefixes.iter().any(|prefix: &&str| line.starts_with(prefix)),
        }
    }

    fn parse_update_diff(mut lines: Vec<String>, input_text: &str) -> ApplicationResult<Vec<V4aChunk>> {
        lines.push(END_PATCH.to_string());
        let input_lines: Vec<String> = input_text.split('\n').map(str::to_string).collect();
        let mut chunks: Vec<V4aChunk> = Vec::new();
        let mut cursor: usize = 0;
        let mut index: usize = 0;

        while !V4aDiffApplier::is_done(&lines, index, &END_SECTION_MARKERS) {
            let (anchors, anchor_count): (Vec<String>, usize) = V4aDiffApplier::read_anchors(&lines, &mut index);
            if anchor_count == 0 && cursor != 0 {
                let current: &str = lines.get(index).map_or("", String::as_str);
                return Err(V4aDiffApplier::to_exception(format!("Invalid Line:\n{current}")));
            }

            let require_anchor_match: bool = anchor_count > 1;
            for (anchor_index, anchor) in anchors.iter().enumerate() {
                cursor = V4aDiffApplier::advance_cursor_to_anchor(
                    anchor,
                    &input_lines,
                    cursor,
                    require_anchor_match,
                    anchor_index > 0,
                )?;
            }

            let section: V4aSection = V4aDiffApplier::read_section(&lines, index)?;
            let Some(new_index) =
                V4aDiffApplier::find_context(&input_lines, &section.next_context, cursor, section.eof)
            else {
                let context: String = section.next_context.join("\n");
                let message: String = if section.eof {
                    format!("Invalid EOF Context {cursor}:\n{context}")
                } else {
                    format!("Invalid Context {cursor}:\n{context}")
                };
                return Err(V4aDiffApplier::to_exception(message));
            };

            cursor = new_index + section.next_context.len();
            index = section.end_index;
            for chunk in section.section_chunks {
                chunks.push(V4aChunk {
                    orig_index: chunk.orig_index + new_index,
                    del_lines: chunk.del_lines,
                    ins_lines: chunk.ins_lines,
                });
            }
        }
        Ok(chunks)
    }

    // The "@@" headers before a section: "@@ <anchor text>", or a bare "@@".
    fn read_anchors(lines: &[String], index: &mut usize) -> (Vec<String>, usize) {
        let mut anchors: Vec<String> = Vec::new();
        let mut anchor_count: usize = 0;
        while let Some(line) = lines.get(*index) {
            let anchor: String = if let Some(anchor) = line.strip_prefix("@@ ") {
                anchor.to_string()
            } else if line == "@@" {
                String::new()
            } else {
                break;
            };
            *index += 1;
            anchor_count += 1;
            if !anchor.trim().is_empty() {
                anchors.push(anchor);
            }
        }
        (anchors, anchor_count)
    }

    fn advance_cursor_to_anchor(
        anchor: &str,
        input_lines: &[String],
        cursor: usize,
        require_match: bool,
        force_forward: bool,
    ) -> ApplicationResult<usize> {
        let before_cursor: &[String] = &input_lines[..cursor.min(input_lines.len())];
        let mut cursor: usize = cursor;
        let mut found: bool = false;

        // First the line as it is, then ignoring surrounding whitespace.
        if !force_forward && before_cursor.iter().any(|line: &String| line == anchor) {
            found = true;
        } else if let Some(offset) = input_lines.iter().skip(cursor).position(|line: &String| line == anchor) {
            cursor += offset + 1;
            found = true;
        }

        if !found {
            if !force_forward && before_cursor.iter().any(|line: &String| line.trim() == anchor.trim()) {
                found = true;
            } else if let Some(offset) = input_lines
                .iter()
                .skip(cursor)
                .position(|line: &String| line.trim() == anchor.trim())
            {
                cursor += offset + 1;
                found = true;
            }
        }

        if require_match && !found {
            return Err(V4aDiffApplier::to_exception(format!(
                "Invalid Anchor {cursor}:\n{anchor}"
            )));
        }
        Ok(cursor)
    }

    fn read_section(lines: &[String], start_index: usize) -> ApplicationResult<V4aSection> {
        let mut context: Vec<String> = Vec::new();
        let mut del_lines: Vec<String> = Vec::new();
        let mut ins_lines: Vec<String> = Vec::new();
        let mut section_chunks: Vec<V4aChunk> = Vec::new();
        let mut mode: LineMode = LineMode::Keep;
        let mut index: usize = start_index;

        while let Some(raw_line) = lines.get(index) {
            if raw_line.starts_with("@@")
                || END_SECTION_MARKERS
                    .iter()
                    .any(|marker: &&str| raw_line.starts_with(marker))
                || raw_line == "***"
            {
                break;
            }
            if raw_line.starts_with("***") {
                return Err(V4aDiffApplier::to_exception(format!("Invalid Line: {raw_line}")));
            }
            index += 1;

            let last_mode: LineMode = mode;
            // An empty line is a context line whose content is empty.
            let line: &str = if raw_line.is_empty() { " " } else { raw_line };
            let content: String = match line.chars().next() {
                Some('+') => {
                    mode = LineMode::Add;
                    line[1..].to_string()
                }
                Some('-') => {
                    mode = LineMode::Delete;
                    line[1..].to_string()
                }
                Some(' ') => {
                    mode = LineMode::Keep;
                    line[1..].to_string()
                }
                _ => return Err(V4aDiffApplier::to_exception(format!("Invalid Line: {line}"))),
            };

            // Back to context after changes: the changes so far are one chunk.
            if mode == LineMode::Keep && last_mode != LineMode::Keep && (!del_lines.is_empty() || !ins_lines.is_empty())
            {
                section_chunks.push(V4aChunk {
                    orig_index: context.len() - del_lines.len(),
                    del_lines: std::mem::take(&mut del_lines),
                    ins_lines: std::mem::take(&mut ins_lines),
                });
            }

            match mode {
                LineMode::Delete => {
                    del_lines.push(content.clone());
                    context.push(content);
                }
                LineMode::Add => ins_lines.push(content),
                LineMode::Keep => context.push(content),
            }
        }

        if !del_lines.is_empty() || !ins_lines.is_empty() {
            section_chunks.push(V4aChunk {
                orig_index: context.len() - del_lines.len(),
                del_lines,
                ins_lines,
            });
        }

        if lines.get(index).is_some_and(|line: &String| line == END_FILE) {
            return Ok(V4aSection {
                next_context: context,
                section_chunks,
                end_index: index + 1,
                eof: true,
            });
        }
        if index == start_index {
            let next_line: &str = lines.get(index).map_or("", String::as_str);
            return Err(V4aDiffApplier::to_exception(format!(
                "Nothing in this section - index={index} {next_line}"
            )));
        }
        Ok(V4aSection {
            next_context: context,
            section_chunks,
            end_index: index,
            eof: false,
        })
    }

    // Where the section's context is in the file, at or after `start`; a section at the end of the file is looked for
    // there first.
    fn find_context(lines: &[String], context: &[String], start: usize, eof: bool) -> Option<usize> {
        if !eof {
            return V4aDiffApplier::find_context_core(lines, context, start);
        }
        let search_lines: &[String] = match lines.last() {
            Some(last_line) if last_line.is_empty() => &lines[..lines.len() - 1],
            _ => lines,
        };
        let end_start: usize = search_lines.len().saturating_sub(context.len());
        V4aDiffApplier::find_context_core(search_lines, context, end_start)
            .or_else(|| V4aDiffApplier::find_context_core(search_lines, context, start.min(search_lines.len())))
    }

    fn find_context_core(lines: &[String], context: &[String], start: usize) -> Option<usize> {
        if context.is_empty() {
            return Some(start);
        }
        let comparisons: [fn(&str) -> &str; 3] = [|line: &str| line, str::trim_end, str::trim];
        for compare in comparisons {
            for index in start..lines.len() {
                if V4aDiffApplier::slice_equals(lines, context, index, compare) {
                    return Some(index);
                }
            }
        }
        None
    }

    fn slice_equals(source: &[String], target: &[String], start: usize, compare: fn(&str) -> &str) -> bool {
        if start + target.len() > source.len() {
            return false;
        }
        target
            .iter()
            .enumerate()
            .all(|(offset, target_line): (usize, &String)| compare(&source[start + offset]) == compare(target_line))
    }

    fn apply_chunks(text: &str, chunks: &[V4aChunk], newline: &str) -> ApplicationResult<String> {
        let original_lines: Vec<&str> = text.split('\n').collect();
        let mut destination_lines: Vec<&str> = Vec::new();
        let mut cursor: usize = 0;

        for chunk in chunks {
            if chunk.orig_index > original_lines.len() {
                return Err(V4aDiffApplier::to_exception(format!(
                    "applyDiff: chunk.origIndex {} > input length {}",
                    chunk.orig_index,
                    original_lines.len()
                )));
            }
            if cursor > chunk.orig_index {
                return Err(V4aDiffApplier::to_exception(format!(
                    "applyDiff: overlapping chunk at {} (cursor {cursor})",
                    chunk.orig_index
                )));
            }
            destination_lines.extend_from_slice(&original_lines[cursor..chunk.orig_index]);
            destination_lines.extend(chunk.ins_lines.iter().map(String::as_str));
            cursor = chunk.orig_index + chunk.del_lines.len();
        }
        destination_lines.extend_from_slice(&original_lines[cursor.min(original_lines.len())..]);
        Ok(destination_lines.join(newline))
    }

    fn to_exception(message: String) -> ApplicationBaseException {
        ApplicationBaseException::new(AgentRunEngineException::InvalidPatch, message, ContextualData::new())
    }
}
