use std::fmt;

use crate::constants::{FORMAT_ESCAPE, FORMAT_ESCAPED_CHAR, FORMAT_NOTE_MARKER};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub notes: Vec<String>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    ContentBeforeFirstNote,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentBeforeFirstNote => {
                formatter.write_str("content appears before the first note marker")
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl Document {
    pub fn new() -> Self {
        Self {
            notes: vec![String::new()],
        }
    }

    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let source = source.strip_prefix('\u{feff}').unwrap_or(source);
        let normalized = source.replace("\r\n", "\n");
        let normalized = normalized.strip_suffix('\n').unwrap_or(&normalized);
        if normalized.is_empty() {
            return Ok(Self { notes: Vec::new() });
        }

        let mut notes = Vec::new();
        let mut current: Option<Vec<String>> = None;
        for line in normalized.split('\n') {
            if line == FORMAT_NOTE_MARKER {
                if let Some(body) = current.take() {
                    notes.push(decode_lines(body));
                }
                current = Some(Vec::new());
            } else if let Some(body) = &mut current {
                body.push(line.to_owned());
            } else {
                return Err(ParseError::ContentBeforeFirstNote);
            }
        }
        if let Some(body) = current {
            notes.push(decode_lines(body));
        }

        Ok(Self { notes })
    }

    pub fn serialize(&self) -> String {
        let mut output = String::new();
        for note in &self.notes {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(FORMAT_NOTE_MARKER);
            if !note.is_empty() {
                output.push('\n');
                output.push_str(&encode_body(note));
            }
        }
        if !self.notes.is_empty() {
            output.push('\n');
        }
        output
    }
}

fn decode_lines(lines: Vec<String>) -> String {
    lines
        .into_iter()
        .map(|line| decode_body_line(&line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_body_line(line: &str) -> String {
    let mut decoded = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if character == FORMAT_ESCAPE && chars.peek() == Some(&FORMAT_ESCAPED_CHAR) {
            decoded.push(FORMAT_ESCAPED_CHAR);
            chars.next();
        } else {
            decoded.push(character);
        }
    }
    decoded
}

fn encode_body(body: &str) -> String {
    body.split('\n')
        .map(encode_body_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn encode_body_line(line: &str) -> String {
    if line == FORMAT_NOTE_MARKER {
        return FORMAT_NOTE_MARKER
            .chars()
            .map(|character| format!("{FORMAT_ESCAPE}{character}"))
            .collect();
    }

    let mut encoded = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if character == FORMAT_ESCAPE && chars.peek() == Some(&FORMAT_ESCAPED_CHAR) {
            encoded.push(FORMAT_ESCAPE);
        }
        encoded.push(character);
    }
    encoded
}
