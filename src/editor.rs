use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::constants::UNDO_LIMIT;

#[derive(Clone)]
struct Snapshot {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct VisualRow {
    pub start: usize,
    pub end: usize,
    pub width: u16,
}

pub struct Editor {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
    preferred_column: Option<u16>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
}

impl Editor {
    pub fn new(text: String) -> Self {
        let cursor = text.len();
        Self {
            text,
            cursor,
            anchor: None,
            preferred_column: None,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        let anchor = self.anchor?;
        (anchor != self.cursor).then(|| anchor.min(self.cursor)..anchor.max(self.cursor))
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selection().map(|range| &self.text[range])
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.len();
        self.preferred_column = None;
    }

    pub fn set_cursor(&mut self, cursor: usize, extend: bool) {
        let cursor = floor_char_boundary(&self.text, cursor.min(self.text.len()));
        self.prepare_selection(extend);
        self.cursor = cursor;
        self.preferred_column = None;
    }

    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.anchor = Some(floor_char_boundary(&self.text, anchor.min(self.text.len())));
        self.cursor = floor_char_boundary(&self.text, cursor.min(self.text.len()));
        self.preferred_column = None;
    }

    pub fn insert(&mut self, value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        self.record();
        self.replace_selection_raw(value);
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.selection().is_some() {
            self.record();
            self.replace_selection_raw("");
            return true;
        }
        let Some(previous) = previous_grapheme_boundary(&self.text, self.cursor) else {
            return false;
        };
        self.record();
        self.text.replace_range(previous..self.cursor, "");
        self.cursor = previous;
        self.preferred_column = None;
        true
    }

    pub fn delete(&mut self) -> bool {
        if self.selection().is_some() {
            self.record();
            self.replace_selection_raw("");
            return true;
        }
        let Some(next) = next_grapheme_boundary(&self.text, self.cursor) else {
            return false;
        };
        self.record();
        self.text.replace_range(self.cursor..next, "");
        self.preferred_column = None;
        true
    }

    pub fn undo(&mut self) -> bool {
        let Some(snapshot) = self.undo.pop() else {
            return false;
        };
        let current = self.snapshot();
        self.redo.push(current);
        self.restore(snapshot);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(snapshot) = self.redo.pop() else {
            return false;
        };
        let current = self.snapshot();
        self.undo.push(current);
        self.restore(snapshot);
        true
    }

    pub fn move_horizontal(&mut self, direction: i8, extend: bool, by_word: bool) {
        if !extend
            && !by_word
            && let Some(selection) = self.selection()
        {
            self.cursor = if direction < 0 {
                selection.start
            } else {
                selection.end
            };
            self.anchor = None;
            self.preferred_column = None;
            return;
        }
        self.prepare_selection(extend);
        self.cursor = if by_word {
            if direction < 0 {
                previous_word_boundary(&self.text, self.cursor)
            } else {
                next_word_boundary(&self.text, self.cursor)
            }
        } else if direction < 0 {
            previous_grapheme_boundary(&self.text, self.cursor).unwrap_or(self.cursor)
        } else {
            next_grapheme_boundary(&self.text, self.cursor).unwrap_or(self.cursor)
        };
        self.preferred_column = None;
    }

    pub fn move_vertical(&mut self, rows: &[VisualRow], direction: i8, extend: bool) {
        if rows.is_empty() {
            return;
        }
        self.prepare_selection(extend);
        let (current_row, current_column) = cursor_row_column(&self.text, self.cursor, rows);
        let column = self.preferred_column.unwrap_or(current_column);
        let target_row = if direction < 0 {
            current_row.saturating_sub(1)
        } else {
            (current_row + 1).min(rows.len() - 1)
        };
        self.cursor = byte_at_column(&self.text, &rows[target_row], column);
        self.preferred_column = Some(column);
    }

    pub fn move_line_edge(&mut self, rows: &[VisualRow], end: bool, extend: bool) {
        self.prepare_selection(extend);
        let (row, _) = cursor_row_column(&self.text, self.cursor, rows);
        self.cursor = if end { rows[row].end } else { rows[row].start };
        self.preferred_column = None;
    }

    pub fn move_page(&mut self, rows: &[VisualRow], distance: usize, down: bool, extend: bool) {
        if rows.is_empty() {
            return;
        }
        self.prepare_selection(extend);
        let (current_row, current_column) = cursor_row_column(&self.text, self.cursor, rows);
        let column = self.preferred_column.unwrap_or(current_column);
        let target_row = if down {
            current_row.saturating_add(distance).min(rows.len() - 1)
        } else {
            current_row.saturating_sub(distance)
        };
        self.cursor = byte_at_column(&self.text, &rows[target_row], column);
        self.preferred_column = Some(column);
    }

    pub fn move_document_edge(&mut self, end: bool, extend: bool) {
        self.prepare_selection(extend);
        self.cursor = if end { self.text.len() } else { 0 };
        self.preferred_column = None;
    }

    pub fn hit_test(&self, rows: &[VisualRow], row: usize, column: u16) -> usize {
        let row = &rows[row.min(rows.len().saturating_sub(1))];
        byte_at_column(&self.text, row, column)
    }

    fn record(&mut self) {
        let snapshot = self.snapshot();
        self.undo.push(snapshot);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
        }
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.text = snapshot.text;
        self.cursor = snapshot.cursor;
        self.anchor = snapshot.anchor;
        self.preferred_column = None;
    }

    fn replace_selection_raw(&mut self, replacement: &str) {
        let range = self.selection().unwrap_or(self.cursor..self.cursor);
        let start = range.start;
        self.text.replace_range(range, replacement);
        self.cursor = start + replacement.len();
        self.anchor = None;
        self.preferred_column = None;
    }

    fn prepare_selection(&mut self, extend: bool) {
        if extend {
            self.anchor.get_or_insert(self.cursor);
        } else {
            self.anchor = None;
        }
    }
}

pub fn visual_rows(text: &str, width: u16) -> Vec<VisualRow> {
    let width = width.max(1);
    let mut rows = Vec::new();
    let mut start = 0;
    let mut row_width = 0;

    for (index, grapheme) in text.grapheme_indices(true) {
        if grapheme == "\n" {
            rows.push(VisualRow {
                start,
                end: index,
                width: row_width,
            });
            start = index + grapheme.len();
            row_width = 0;
            continue;
        }
        let grapheme_width = display_width(grapheme);
        if row_width > 0 && row_width.saturating_add(grapheme_width) > width {
            rows.push(VisualRow {
                start,
                end: index,
                width: row_width,
            });
            start = index;
            row_width = 0;
        }
        row_width = row_width.saturating_add(grapheme_width);
    }
    rows.push(VisualRow {
        start,
        end: text.len(),
        width: row_width,
    });
    rows
}

pub fn cursor_row_column(text: &str, cursor: usize, rows: &[VisualRow]) -> (usize, u16) {
    let row_index = rows
        .iter()
        .enumerate()
        .rev()
        .find(|(_, row)| row.start <= cursor && cursor <= row.end)
        .map(|(index, _)| index)
        .unwrap_or(0);
    let row = &rows[row_index];
    let end = cursor.min(row.end);
    (row_index, display_width(&text[row.start..end]))
}

pub fn display_width(value: &str) -> u16 {
    if value.is_empty() {
        0
    } else if value.chars().any(|character| character.is_control()) {
        1
    } else {
        UnicodeWidthStr::width(value).max(1).min(u16::MAX as usize) as u16
    }
}

pub fn display_grapheme(value: &str) -> &str {
    if value.chars().any(|character| character.is_control()) {
        "�"
    } else {
        value
    }
}

fn byte_at_column(text: &str, row: &VisualRow, column: u16) -> usize {
    let mut current = 0_u16;
    for (offset, grapheme) in text[row.start..row.end].grapheme_indices(true) {
        let width = display_width(grapheme);
        if column < current.saturating_add(width.div_ceil(2)) {
            return row.start + offset;
        }
        current = current.saturating_add(width);
    }
    row.end
}

fn previous_grapheme_boundary(text: &str, cursor: usize) -> Option<usize> {
    text[..cursor]
        .grapheme_indices(true)
        .next_back()
        .map(|(i, _)| i)
}

fn next_grapheme_boundary(text: &str, cursor: usize) -> Option<usize> {
    text[cursor..]
        .grapheme_indices(true)
        .next()
        .map(|(_, grapheme)| cursor + grapheme.len())
}

fn previous_word_boundary(text: &str, cursor: usize) -> usize {
    let mut boundary = 0;
    for (index, segment) in text[..cursor].split_word_bound_indices() {
        if segment.chars().any(char::is_alphanumeric) {
            boundary = index;
        }
    }
    boundary
}

fn next_word_boundary(text: &str, cursor: usize) -> usize {
    for (offset, segment) in text[cursor..].split_word_bound_indices() {
        if offset > 0 && segment.chars().any(char::is_alphanumeric) {
            return cursor + offset;
        }
    }
    text.len()
}

fn floor_char_boundary(text: &str, mut index: usize) -> usize {
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}
