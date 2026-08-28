use std::fmt;

use crate::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    String {
        value: String,
        raw: String,
    },
    Integer {
        value: i64,
        raw: String,
    },
    Number {
        value: String,
        raw: String,
    },
    Boolean(bool),
    Duration {
        value: i64,
        unit: String,
        raw: String,
    },
    LBrace,
    RBrace,
    Equals,
    Comma,
    Semicolon,
    LineComment(String),
    BlockComment(String),
    Whitespace(String),
    Eof,
}

impl TokenKind {
    pub fn name(&self) -> &str {
        match self {
            TokenKind::Identifier(_) => "identifier",
            TokenKind::String { .. } => "string",
            TokenKind::Integer { .. } => "integer",
            TokenKind::Number { .. } => "number",
            TokenKind::Boolean(_) => "boolean",
            TokenKind::Duration { .. } => "duration",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::Equals => "'='",
            TokenKind::Comma => "','",
            TokenKind::Semicolon => "';'",
            TokenKind::LineComment(_) => "line comment",
            TokenKind::BlockComment(_) => "block comment",
            TokenKind::Whitespace(_) => "whitespace",
            TokenKind::Eof => "end of file",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    max_file_size: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            max_file_size: 64 * 1024 * 1024,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        if self.input.len() > self.max_file_size {
            return Err(LexerError::FileTooLarge(self.input.len()));
        }

        loop {
            if self.pos >= self.input.len() {
                self.tokens.push(Token {
                    kind: TokenKind::Eof,
                    span: Span::new(self.pos, self.pos, self.line, self.column),
                });
                break;
            }

            let ch = self.current_byte();

            match ch {
                b' ' | b'\t' | b'\r' | b'\n' => self.read_whitespace()?,
                b'/' => {
                    if self.peek() == Some(b'/') {
                        self.read_line_comment()?;
                    } else if self.peek() == Some(b'*') {
                        self.read_block_comment()?;
                    } else {
                        self.read_identifier()?;
                    }
                }
                b'{' => self.add_token(TokenKind::LBrace, 1),
                b'}' => self.add_token(TokenKind::RBrace, 1),
                b'=' => self.add_token(TokenKind::Equals, 1),
                b',' => self.add_token(TokenKind::Comma, 1),
                b';' => self.add_token(TokenKind::Semicolon, 1),
                b'"' => self.read_string()?,
                b'0'..=b'9' | b'-' => {
                    if self.looks_like_mac_from_start() || self.looks_like_ipv6_from_start() {
                        self.read_identifier()?;
                    } else {
                        self.read_number_or_duration()?;
                    }
                }
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => self.read_identifier()?,
                b':' => self.read_identifier()?,
                _ => {
                    if ch.is_ascii_graphic() || ch == b' ' {
                        self.read_identifier()?;
                    } else {
                        return Err(LexerError::UnexpectedByte {
                            byte: ch,
                            position: self.pos,
                            line: self.line,
                            column: self.column,
                        });
                    }
                }
            }
        }

        Ok(self.tokens.clone())
    }

    fn current_byte(&self) -> u8 {
        self.input[self.pos]
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> u8 {
        let ch = self.input[self.pos];
        if ch == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.pos += 1;
        ch
    }

    fn add_token(&mut self, kind: TokenKind, len: usize) {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;

        for _ in 0..len {
            self.advance();
        }

        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos, start_line, start_column),
        });
    }

    fn read_whitespace(&mut self) -> Result<(), LexerError> {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();

        while self.pos < self.input.len() {
            let ch = self.current_byte();
            match ch {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    value.push(ch as char);
                    self.advance();
                }
                _ => break,
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Whitespace(value),
            span: Span::new(start, self.pos, start_line, start_column),
        });

        Ok(())
    }

    fn read_line_comment(&mut self) -> Result<(), LexerError> {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();

        // skip //
        self.advance();
        self.advance();

        while self.pos < self.input.len() {
            let ch = self.current_byte();
            if ch == b'\n' {
                break;
            }
            value.push(ch as char);
            self.advance();
        }

        self.tokens.push(Token {
            kind: TokenKind::LineComment(value),
            span: Span::new(start, self.pos, start_line, start_column),
        });

        Ok(())
    }

    fn read_block_comment(&mut self) -> Result<(), LexerError> {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();
        let mut depth = 1u32;

        // skip /*
        self.advance();
        self.advance();

        while self.pos < self.input.len() && depth > 0 {
            let ch = self.current_byte();
            if ch == b'/' && self.peek() == Some(b'*') {
                depth += 1;
                value.push(ch as char);
                self.advance();
                value.push(self.advance() as char);
            } else if ch == b'*' && self.peek() == Some(b'/') {
                self.advance();
                self.advance();
                depth -= 1;
                if depth > 0 {
                    value.push(ch as char);
                    value.push('/');
                }
            } else {
                value.push(ch as char);
                self.advance();
            }
        }

        if depth > 0 {
            return Err(LexerError::UnterminatedBlockComment {
                position: start,
                line: start_line,
                column: start_column,
            });
        }

        self.tokens.push(Token {
            kind: TokenKind::BlockComment(value),
            span: Span::new(start, self.pos, start_line, start_column),
        });

        Ok(())
    }

    fn read_string(&mut self) -> Result<(), LexerError> {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();
        let mut raw = String::new();

        // opening quote
        raw.push(self.advance() as char);

        loop {
            if self.pos >= self.input.len() {
                return Err(LexerError::UnterminatedString {
                    position: start,
                    line: start_line,
                    column: start_column,
                });
            }

            let ch = self.current_byte();
            if ch == b'"' {
                raw.push(self.advance() as char);
                break;
            } else if ch == b'\\' {
                raw.push(self.advance() as char);
                if self.pos >= self.input.len() {
                    return Err(LexerError::UnterminatedStringEscape {
                        position: self.pos - 1,
                        line: self.line,
                        column: self.column - 1,
                    });
                }
                if self.current_byte() == b'"' {
                    raw.push(self.advance() as char);
                    value.push('"');
                } else {
                    value.push('\\');
                }
            } else {
                let (c, len) = decode_utf8_char(&self.input[self.pos..]);
                for _ in 0..len {
                    self.advance();
                }
                let raw_str: String = c.encode_utf8(&mut [0u8; 4]).to_string();
                value.push(c);
                raw.push_str(&raw_str);
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::String { value, raw },
            span: Span::new(start, self.pos, start_line, start_column),
        });

        Ok(())
    }

    fn read_number_or_duration(&mut self) -> Result<(), LexerError> {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;
        let mut raw = String::new();

        // optional minus
        if self.current_byte() == b'-' {
            raw.push(self.advance() as char);
        }

        // digits
        if self.pos >= self.input.len() || !self.current_byte().is_ascii_digit() {
            return Err(LexerError::UnexpectedByte {
                byte: self.current_byte(),
                position: self.pos,
                line: self.line,
                column: self.column,
            });
        }

        while self.pos < self.input.len() && self.current_byte().is_ascii_digit() {
            raw.push(self.advance() as char);
        }

        // check for IP address pattern (digits.digits.digits.digits) BEFORE floating point
        if self.pos < self.input.len()
            && self.current_byte() == b'.'
            && self.looks_like_ip_after_number()
        {
            self.read_ip_address(start, start_line, start_column, &mut raw)?;
            return Ok(());
        }

        // check for dot (floating point)
        if self.pos < self.input.len() && self.current_byte() == b'.' {
            raw.push(self.advance() as char);
            while self.pos < self.input.len() && self.current_byte().is_ascii_digit() {
                raw.push(self.advance() as char);
            }
            self.tokens.push(Token {
                kind: TokenKind::Number {
                    value: raw.clone(),
                    raw,
                },
                span: Span::new(start, self.pos, start_line, start_column),
            });
            return Ok(());
        }

        // check for duration unit (ms, s, m, h, d)
        if self.pos < self.input.len() {
            let ch = self.current_byte();
            // check for two-char units first: "ms"
            if ch == b'm'
                && self.pos + 1 < self.input.len()
                && self.input[self.pos + 1] == b's'
                && (self.pos + 2 >= self.input.len() || !is_ident_part(self.input[self.pos + 2]))
            {
                let unit = "ms".to_string();
                raw.push(self.advance() as char); // 'm'
                raw.push(self.advance() as char); // 's'
                let num_str = &raw[..raw.len() - 2];
                let value: i64 = num_str.parse().map_err(|_| LexerError::InvalidNumber {
                    text: raw.clone(),
                    position: start,
                    line: start_line,
                    column: start_column,
                })?;
                self.tokens.push(Token {
                    kind: TokenKind::Duration { value, unit, raw },
                    span: Span::new(start, self.pos, start_line, start_column),
                });
                return Ok(());
            }
            // single-char units: s, m, h, d, w
            if matches!(ch, b'm' | b's' | b'h' | b'd' | b'w')
                && (self.pos + 1 >= self.input.len() || !is_ident_part(self.input[self.pos + 1]))
            {
                let unit = (ch as char).to_string();
                raw.push(self.advance() as char);
                let num_str = &raw[..raw.len() - 1];
                let value: i64 = num_str.parse().map_err(|_| LexerError::InvalidNumber {
                    text: raw.clone(),
                    position: start,
                    line: start_line,
                    column: start_column,
                })?;
                self.tokens.push(Token {
                    kind: TokenKind::Duration { value, unit, raw },
                    span: Span::new(start, self.pos, start_line, start_column),
                });
                return Ok(());
            }
        }

        let value: i64 = raw.parse().map_err(|_| LexerError::InvalidNumber {
            text: raw.clone(),
            position: start,
            line: start_line,
            column: start_column,
        })?;
        self.tokens.push(Token {
            kind: TokenKind::Integer { value, raw },
            span: Span::new(start, self.pos, start_line, start_column),
        });

        Ok(())
    }

    fn looks_like_ip_after_number(&self) -> bool {
        // Check if current position has dot followed by digits
        let saved_pos = self.pos;
        let mut p = saved_pos;

        // We're at the dot after the first number group
        if p >= self.input.len() || self.input[p] != b'.' {
            return false;
        }
        p += 1; // skip dot

        // expect digits
        if p >= self.input.len() || !self.input[p].is_ascii_digit() {
            return false;
        }
        while p < self.input.len() && self.input[p].is_ascii_digit() {
            p += 1;
        }

        // could be dot.digits.digits pattern (IP-like)
        // We need at least one more .digits
        if p < self.input.len() && self.input[p] == b'.' {
            p += 1;
            if p >= self.input.len() || !self.input[p].is_ascii_digit() {
                return false;
            }
            while p < self.input.len() && self.input[p].is_ascii_digit() {
                p += 1;
            }

            // check for third dot
            if p < self.input.len() && self.input[p] == b'.' {
                p += 1;
                if p >= self.input.len() || !self.input[p].is_ascii_digit() {
                    return false;
                }
                while p < self.input.len() && self.input[p].is_ascii_digit() {
                    p += 1;
                }
                // After the full IP, it should not be followed by an identifier char
                if p >= self.input.len() || !is_ident_part(self.input[p]) {
                    return true;
                }
            }
        }

        false
    }

    fn read_ip_address(
        &mut self,
        start: usize,
        start_line: usize,
        start_column: usize,
        raw: &mut String,
    ) -> Result<(), LexerError> {
        // raw already has the first number group
        while self.pos < self.input.len() && self.current_byte() == b'.' {
            raw.push(self.advance() as char);
            while self.pos < self.input.len() && self.current_byte().is_ascii_digit() {
                raw.push(self.advance() as char);
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Number {
                value: raw.clone(),
                raw: raw.clone(),
            },
            span: Span::new(start, self.pos, start_line, start_column),
        });

        Ok(())
    }

    fn looks_like_mac_from_start(&self) -> bool {
        // Check if current position starts a MAC address: hexhex:hexhex:hexhex:hexhex:hexhex:hexhex
        let mut p = self.pos;
        for group in 0..6u32 {
            // Need two hex digits
            if p + 1 >= self.input.len() {
                return false;
            }
            if !self.input[p].is_ascii_hexdigit() || !self.input[p + 1].is_ascii_hexdigit() {
                return false;
            }
            p += 2;
            if group < 5 {
                // Expect ':'
                if p >= self.input.len() || self.input[p] != b':' {
                    return false;
                }
                p += 1;
            }
        }
        // After the MAC, should not be followed by an ident-part char (to avoid matching inside a longer token)
        p < self.input.len() && !is_ident_part(self.input[p])
    }

    fn looks_like_ipv6_from_start(&self) -> bool {
        // Check if current position starts an IPv6 address containing ':'
        // Heuristic: digits followed by ':' somewhere in the token
        let mut p = self.pos;
        // Read initial digits (hex)
        while p < self.input.len() && self.input[p].is_ascii_hexdigit() {
            p += 1;
        }
        // Must have at least one ':' following the digits to be IPv6-like
        p < self.input.len() && self.input[p] == b':'
    }

    fn read_identifier(&mut self) -> Result<(), LexerError> {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();

        while self.pos < self.input.len() {
            let ch = self.current_byte();
            if is_ident_part(ch) {
                value.push(self.advance() as char);
            } else {
                break;
            }
        }

        if self.pos == start {
            value.push(self.advance() as char);
        }

        // Check for boolean keywords
        let kind = match value.as_str() {
            "yes" => TokenKind::Boolean(true),
            "no" => TokenKind::Boolean(false),
            _ => TokenKind::Identifier(value),
        };

        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos, start_line, start_column),
        });

        Ok(())
    }
}

fn is_ident_part(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'.' || ch == b'-' || ch == b':'
}

fn decode_utf8_char(bytes: &[u8]) -> (char, usize) {
    if bytes.is_empty() {
        return ('\0', 0);
    }
    let first = bytes[0];
    let len = if first < 0xC0 {
        1 // ASCII (0x00..0x80) or invalid leading byte (0x80..0xC0)
    } else if first < 0xE0 {
        2
    } else if first < 0xF0 {
        3
    } else {
        4
    };
    let len = len.min(bytes.len());
    match std::str::from_utf8(&bytes[..len]) {
        Ok(s) => {
            let c = s.chars().next().unwrap_or('\0');
            (c, len)
        }
        Err(_) => (bytes[0] as char, 1),
    }
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedByte {
        byte: u8,
        position: usize,
        line: usize,
        column: usize,
    },
    UnexpectedEof {
        position: usize,
        line: usize,
        column: usize,
    },
    UnterminatedString {
        position: usize,
        line: usize,
        column: usize,
    },
    UnterminatedBlockComment {
        position: usize,
        line: usize,
        column: usize,
    },
    UnterminatedStringEscape {
        position: usize,
        line: usize,
        column: usize,
    },
    InvalidHexEscape {
        position: usize,
        line: usize,
        column: usize,
    },
    InvalidUnicodeEscape {
        position: usize,
        line: usize,
        column: usize,
    },
    InvalidNumber {
        text: String,
        position: usize,
        line: usize,
        column: usize,
    },
    FileTooLarge(usize),
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::UnexpectedByte {
                byte,
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "unexpected byte 0x{:02x} ({}) at position {} (line {}, column {})",
                    byte,
                    if byte.is_ascii_graphic() || *byte == b' ' {
                        format!("'{}'", *byte as char)
                    } else {
                        format!("0x{:02x}", byte)
                    },
                    position,
                    line,
                    column
                )
            }
            LexerError::UnexpectedEof {
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "unexpected end of file at position {} (line {}, column {})",
                    position, line, column
                )
            }
            LexerError::UnterminatedString {
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "unterminated string starting at position {} (line {}, column {})",
                    position, line, column
                )
            }
            LexerError::UnterminatedBlockComment {
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "unterminated block comment starting at position {} (line {}, column {})",
                    position, line, column
                )
            }
            LexerError::UnterminatedStringEscape {
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "unterminated string escape at position {} (line {}, column {})",
                    position, line, column
                )
            }
            LexerError::InvalidHexEscape {
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "invalid hex escape at position {} (line {}, column {})",
                    position, line, column
                )
            }
            LexerError::InvalidUnicodeEscape {
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "invalid unicode escape at position {} (line {}, column {})",
                    position, line, column
                )
            }
            LexerError::InvalidNumber {
                text,
                position,
                line,
                column,
            } => {
                write!(
                    f,
                    "invalid number '{}' at position {} (line {}, column {})",
                    text, position, line, column
                )
            }
            LexerError::FileTooLarge(size) => {
                write!(f, "file too large: {} bytes", size)
            }
        }
    }
}

impl std::error::Error for LexerError {}
