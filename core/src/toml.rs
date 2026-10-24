//! A small subset of TOML: enough for the roster and `summary.toml`, so the tool needs no dependencies.
//!
//! Supported: comments, bare and quoted keys, basic and literal strings, integers, booleans, and arrays of
//! tables (`[[repo]]`). Anything else is rejected with the line number.

use std::collections::BTreeMap;
use std::fmt;

/// A scalar value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A string.
    String(String),
    /// An integer.
    Integer(i64),
    /// A boolean.
    Boolean(bool),
}

/// A table of key-value pairs.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Table {
    entries: BTreeMap<String, Value>,
    line: usize,
}

/// A parsed document: the root table and the arrays of tables.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Document {
    /// Keys before the first `[[...]]` header.
    pub root: Table,
    /// `[[name]]` sections, in order.
    pub arrays: BTreeMap<String, Vec<Table>>,
}

/// A parse or type error, with its line (1-based; 0 when unknown).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlError {
    /// The line the error is on.
    pub line: usize,
    /// What went wrong.
    pub message: String,
}

impl fmt::Display for TomlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for TomlError {}

fn error(line: usize, message: impl Into<String>) -> TomlError {
    TomlError {
        line,
        message: message.into(),
    }
}

impl Table {
    /// The line the table starts on.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The keys present.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// An optional string.
    pub fn string(&self, key: &str) -> Result<Option<String>, TomlError> {
        match self.entries.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(error(self.line, format!("`{key}` must be a string"))),
        }
    }

    /// An optional integer.
    pub fn integer(&self, key: &str) -> Result<Option<i64>, TomlError> {
        match self.entries.get(key) {
            None => Ok(None),
            Some(Value::Integer(i)) => Ok(Some(*i)),
            Some(_) => Err(error(self.line, format!("`{key}` must be an integer"))),
        }
    }

    /// A required string.
    pub fn required_string(&self, key: &str) -> Result<String, TomlError> {
        self.string(key)?
            .ok_or_else(|| error(self.line, format!("`{key}` is missing")))
    }

    /// A required integer.
    pub fn required_integer(&self, key: &str) -> Result<i64, TomlError> {
        self.integer(key)?
            .ok_or_else(|| error(self.line, format!("`{key}` is missing")))
    }

    /// Rejects keys outside `allowed`, to catch typos.
    pub fn only(&self, allowed: &[&str]) -> Result<(), TomlError> {
        match self.keys().find(|key| !allowed.contains(key)) {
            Some(key) => Err(error(self.line, format!("unknown key `{key}`"))),
            None => Ok(()),
        }
    }
}

/// Parses a document.
pub fn parse(text: &str) -> Result<Document, TomlError> {
    let mut document = Document::default();
    let mut current: Option<(String, Table)> = None;
    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = strip_comment(raw, line_no)?.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("[[") {
            let name = rest
                .strip_suffix("]]")
                .ok_or_else(|| error(line_no, "unterminated `[[` header"))?
                .trim();
            if !is_bare_key(name) {
                return Err(error(line_no, format!("invalid table name `{name}`")));
            }
            if let Some((previous, table)) = current.take() {
                document.arrays.entry(previous).or_default().push(table);
            }
            current = Some((
                name.to_string(),
                Table {
                    entries: BTreeMap::new(),
                    line: line_no,
                },
            ));
            continue;
        }
        if line.starts_with('[') {
            return Err(error(line_no, "only `[[array]]` tables are supported"));
        }
        let (key, value) = parse_pair(line, line_no)?;
        let table = match current.as_mut() {
            Some((_, table)) => table,
            None => &mut document.root,
        };
        if table.entries.insert(key.clone(), value).is_some() {
            return Err(error(line_no, format!("duplicate key `{key}`")));
        }
    }
    if let Some((name, table)) = current {
        document.arrays.entry(name).or_default().push(table);
    }
    document.root.line = 1;
    Ok(document)
}

fn strip_comment(line: &str, line_no: usize) -> Result<&str, TomlError> {
    let mut in_basic = false;
    let mut in_literal = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if in_basic {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_basic = false;
            }
        } else if in_literal {
            if c == '\'' {
                in_literal = false;
            }
        } else if c == '"' {
            in_basic = true;
        } else if c == '\'' {
            in_literal = true;
        } else if c == '#' {
            return Ok(&line[..i]);
        }
    }
    if in_basic || in_literal {
        return Err(error(line_no, "unterminated string"));
    }
    Ok(line)
}

fn is_bare_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn parse_pair(line: &str, line_no: usize) -> Result<(String, Value), TomlError> {
    let (key, rest) = if let Some(rest) = line.strip_prefix('"') {
        let (key, consumed) = parse_basic_string(rest, line_no)?;
        (key, rest[consumed..].trim_start())
    } else {
        let end = line
            .find(|c: char| c == '=' || c.is_whitespace())
            .ok_or_else(|| error(line_no, "expected `key = value`"))?;
        let key = &line[..end];
        if !is_bare_key(key) {
            return Err(error(line_no, format!("invalid key `{key}`")));
        }
        (key.to_string(), line[end..].trim_start())
    };
    let value_text = rest
        .strip_prefix('=')
        .ok_or_else(|| error(line_no, "expected `=`"))?
        .trim();
    Ok((key, parse_value(value_text, line_no)?))
}

fn parse_value(text: &str, line_no: usize) -> Result<Value, TomlError> {
    if let Some(rest) = text.strip_prefix('"') {
        let (s, consumed) = parse_basic_string(rest, line_no)?;
        expect_end(&rest[consumed..], line_no)?;
        return Ok(Value::String(s));
    }
    if let Some(rest) = text.strip_prefix('\'') {
        let end = rest
            .find('\'')
            .ok_or_else(|| error(line_no, "unterminated string"))?;
        expect_end(&rest[end + 1..], line_no)?;
        return Ok(Value::String(rest[..end].to_string()));
    }
    match text {
        "true" => return Ok(Value::Boolean(true)),
        "false" => return Ok(Value::Boolean(false)),
        _ => {}
    }
    let digits: String = text.chars().filter(|c| *c != '_').collect();
    digits
        .parse::<i64>()
        .map(Value::Integer)
        .map_err(|_| error(line_no, format!("unsupported value `{text}`")))
}

fn expect_end(rest: &str, line_no: usize) -> Result<(), TomlError> {
    if rest.trim().is_empty() {
        Ok(())
    } else {
        Err(error(line_no, format!("unexpected `{}`", rest.trim())))
    }
}

/// Parses the inside of a basic string (after the opening quote); returns the value and the bytes consumed,
/// closing quote included.
fn parse_basic_string(text: &str, line_no: usize) -> Result<(String, usize), TomlError> {
    let mut out = String::new();
    let mut chars = text.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Ok((out, i + 1)),
            '\\' => {
                let (_, escape) = chars
                    .next()
                    .ok_or_else(|| error(line_no, "unterminated escape"))?;
                match escape {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'u' => {
                        let hex: String = chars.by_ref().take(4).map(|(_, c)| c).collect();
                        let code = u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .ok_or_else(|| error(line_no, format!("invalid escape `\\u{hex}`")))?;
                        out.push(code);
                    }
                    other => return Err(error(line_no, format!("invalid escape `\\{other}`"))),
                }
            }
            c => out.push(c),
        }
    }
    Err(error(line_no, "unterminated string"))
}

/// Quotes a string as a TOML basic string, on one line.
pub fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_and_arrays_of_tables() {
        let doc = parse(
            r#"
            # roster
            work_dir = "/tmp/w"   # trailing comment
            jobs = 6
            [[repo]]
            name = "a"
            url = 'git@x:a.git'
            [[repo]]
            name = "b"
            "quoted key" = true
            "#,
        )
        .unwrap();
        assert_eq!(doc.root.string("work_dir").unwrap().unwrap(), "/tmp/w");
        assert_eq!(doc.root.integer("jobs").unwrap(), Some(6));
        let repos = &doc.arrays["repo"];
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].string("url").unwrap().unwrap(), "git@x:a.git");
        assert_eq!(repos[1].keys().collect::<Vec<_>>(), ["name", "quoted key"]);
    }

    #[test]
    fn quote_round_trips_awkward_text() {
        let text = "a \"b\" \\ c\nd\te # f \u{1}";
        let doc = parse(&format!("k = {}", quote(text))).unwrap();
        assert_eq!(doc.root.string("k").unwrap().unwrap(), text);
    }

    #[test]
    fn integers_accept_underscores_and_signs() {
        let doc = parse("a = 21_600\nb = -3").unwrap();
        assert_eq!(doc.root.integer("a").unwrap(), Some(21600));
        assert_eq!(doc.root.integer("b").unwrap(), Some(-3));
    }

    #[test]
    fn errors_name_the_line() {
        let cases = [
            ("a = 1\na = 2", 2, "duplicate"),
            ("a = \"x", 1, "unterminated"),
            ("[table]", 1, "only"),
            ("a = 1.5", 1, "unsupported"),
            ("a 1", 1, "expected `=`"),
            ("a = \"x\" y", 1, "unexpected"),
        ];
        for (text, line, fragment) in cases {
            let err = parse(text).unwrap_err();
            assert_eq!(err.line, line, "{text}");
            assert!(err.message.contains(fragment), "{text}: {}", err.message);
        }
    }

    #[test]
    fn typed_accessors_reject_the_wrong_type_and_unknown_keys() {
        let doc = parse("a = 1\nb = \"x\"").unwrap();
        assert!(doc.root.string("a").is_err());
        assert!(doc.root.integer("b").is_err());
        assert!(doc.root.required_string("c").is_err());
        assert!(doc.root.only(&["a"]).is_err());
        assert!(doc.root.only(&["a", "b"]).is_ok());
    }
}
