//! Parse the small RON subset used by AA contract fixtures and schema validation.

use serde_json::{Number, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RonParseError {
    pub message: String,
}

impl std::fmt::Display for RonParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RonParseError {}

struct RonSubsetParser<'a> {
    text: &'a str,
    index: usize,
}

impl<'a> RonSubsetParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, index: 0 }
    }

    fn parse(mut self) -> Result<Value, RonParseError> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.index != self.text.len() {
            return Err(RonParseError {
                message: format!("Unexpected token at byte {}", self.index),
            });
        }
        Ok(value)
    }

    fn skip_ws(&mut self) {
        while self.index < self.text.len() {
            if self.text[self.index..].starts_with("//") {
                let next_line = self.text[self.index..].find('\n');
                self.index = match next_line {
                    Some(offset) => self.index + offset + 1,
                    None => self.text.len(),
                };
                continue;
            }
            let Some(ch) = self.text[self.index..].chars().next() else {
                break;
            };
            if ch.is_whitespace() {
                self.index += ch.len_utf8();
                continue;
            }
            break;
        }
    }

    fn parse_value(&mut self) -> Result<Value, RonParseError> {
        self.skip_ws();
        if self.index >= self.text.len() {
            return Err(RonParseError {
                message: "Unexpected end of RON".into(),
            });
        }
        let ch = self.peek();
        if ch == '(' {
            return self.parse_group();
        }
        if ch == '[' {
            return self.parse_array();
        }
        if ch == '"' {
            return self.parse_string().map(Value::String);
        }
        if ch == '-' || ch.is_ascii_digit() {
            return self.parse_number();
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let ident = self.parse_identifier()?;
            self.skip_ws();
            if self.peek() == '(' {
                return self.parse_group();
            }
            return Ok(match ident.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                "null" | "None" => Value::Null,
                other => Value::String(other.to_string()),
            });
        }
        Err(RonParseError {
            message: format!("Unsupported token '{ch}' at byte {}", self.index),
        })
    }

    fn parse_group(&mut self) -> Result<Value, RonParseError> {
        self.expect('(')?;
        self.skip_ws();
        if self.peek() == ')' {
            self.index += 1;
            return Ok(Value::Object(Default::default()));
        }
        if self.looks_like_object_entry() {
            return self.parse_object_body();
        }
        self.parse_tuple_body()
    }

    fn looks_like_object_entry(&mut self) -> bool {
        let snapshot = self.index;
        let is_object = (|| {
            self.parse_key().ok()?;
            self.skip_ws();
            Some(self.peek() == ':')
        })()
        .unwrap_or(false);
        self.index = snapshot;
        is_object
    }

    fn parse_object_body(&mut self) -> Result<Value, RonParseError> {
        let mut result = serde_json::Map::new();
        loop {
            self.skip_ws();
            if self.peek() == ')' {
                self.index += 1;
                return Ok(Value::Object(result));
            }
            let key = self.parse_key()?;
            self.skip_ws();
            self.expect(':')?;
            result.insert(key, self.parse_value()?);
            self.skip_ws();
            if self.peek() == ',' {
                self.index += 1;
            }
        }
    }

    fn parse_tuple_body(&mut self) -> Result<Value, RonParseError> {
        let mut result = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == ')' {
                self.index += 1;
                return Ok(Value::Array(result));
            }
            result.push(self.parse_value()?);
            self.skip_ws();
            if self.peek() == ',' {
                self.index += 1;
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, RonParseError> {
        self.expect('[')?;
        let mut result = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == ']' {
                self.index += 1;
                return Ok(Value::Array(result));
            }
            result.push(self.parse_value()?);
            self.skip_ws();
            if self.peek() == ',' {
                self.index += 1;
            }
        }
    }

    fn parse_key(&mut self) -> Result<String, RonParseError> {
        self.skip_ws();
        if self.peek() == '"' {
            return self.parse_string();
        }
        self.parse_identifier()
    }

    fn parse_identifier(&mut self) -> Result<String, RonParseError> {
        self.skip_ws();
        let start = self.index;
        while self.index < self.text.len() {
            let ch = self.text[self.index..].chars().next().unwrap();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.index += ch.len_utf8();
            } else {
                break;
            }
        }
        if self.index == start {
            return Err(RonParseError {
                message: format!("Expected identifier at byte {}", self.index),
            });
        }
        Ok(self.text[start..self.index].to_string())
    }

    fn parse_string(&mut self) -> Result<String, RonParseError> {
        self.expect('"')?;
        let mut chars = String::new();
        while self.index < self.text.len() {
            let ch = self.text[self.index..].chars().next().unwrap();
            self.index += ch.len_utf8();
            if ch == '"' {
                return Ok(chars);
            }
            if ch == '\\' {
                if self.index >= self.text.len() {
                    return Err(RonParseError {
                        message: "Unterminated string escape".into(),
                    });
                }
                let escaped = self.text[self.index..].chars().next().unwrap();
                self.index += escaped.len_utf8();
                let decoded = match escaped {
                    '"' => '"',
                    '\\' => '\\',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    other => {
                        return Err(RonParseError {
                            message: format!("Unsupported string escape '\\{other}'"),
                        });
                    }
                };
                chars.push(decoded);
            } else {
                chars.push(ch);
            }
        }
        Err(RonParseError {
            message: "Unterminated string".into(),
        })
    }

    fn parse_number(&mut self) -> Result<Value, RonParseError> {
        self.skip_ws();
        let start = self.index;
        if self.peek() == '-' {
            self.index += 1;
        }
        while self.index < self.text.len() {
            let ch = self.text[self.index..].chars().next().unwrap();
            if ch.is_ascii_digit() {
                self.index += ch.len_utf8();
            } else {
                break;
            }
        }
        if self.peek() == '.' {
            self.index += 1;
            while self.index < self.text.len() {
                let ch = self.text[self.index..].chars().next().unwrap();
                if ch.is_ascii_digit() {
                    self.index += ch.len_utf8();
                } else {
                    break;
                }
            }
        }
        if matches!(self.peek(), 'e' | 'E') {
            self.index += 1;
            if matches!(self.peek(), '+' | '-') {
                self.index += 1;
            }
            while self.index < self.text.len() {
                let ch = self.text[self.index..].chars().next().unwrap();
                if ch.is_ascii_digit() {
                    self.index += ch.len_utf8();
                } else {
                    break;
                }
            }
        }
        let token = &self.text[start..self.index];
        if token.is_empty() || token == "-" || token == "." || token == "-." {
            return Err(RonParseError {
                message: format!("Expected number at byte {start}"),
            });
        }
        if token.contains(['.', 'e', 'E']) {
            let value: f64 = token
                .parse()
                .map_err(|_| RonParseError {
                    message: format!("Invalid float literal {token:?}"),
                })?;
            Number::from_f64(value)
                .map(Value::Number)
                .ok_or_else(|| RonParseError {
                    message: format!("Invalid float literal {token:?}"),
                })
        } else {
            let value: i64 = token
                .parse()
                .map_err(|_| RonParseError {
                    message: format!("Invalid integer literal {token:?}"),
                })?;
            Ok(Value::Number(value.into()))
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), RonParseError> {
        self.skip_ws();
        if self.peek() != expected {
            return Err(RonParseError {
                message: format!("Expected '{expected}' at byte {}", self.index),
            });
        }
        self.index += 1;
        Ok(())
    }

    fn peek(&self) -> char {
        self.text[self.index..].chars().next().unwrap_or('\0')
    }
}

/// Parse RON text using the AA contract subset parser.
pub fn parse_ron_subset(text: &str) -> Result<Value, RonParseError> {
    RonSubsetParser::new(text).parse()
}

/// Parse a RON file using the AA contract subset parser.
pub fn load_ron_subset(path: &std::path::Path) -> Result<Value, RonParseError> {
    let text = std::fs::read_to_string(path).map_err(|e| RonParseError {
        message: format!("{}: {e}", path.display()),
    })?;
    parse_ron_subset(&text)
}
