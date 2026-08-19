use crate::ast::*;
use crate::error::Ar7Error;
use crate::lexer::{Lexer, Token, TokenKind};

const MAX_NESTING_DEPTH: usize = 256;
const MAX_LIST_LENGTH: usize = 4096;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
        }
    }

    pub fn parse_document(&mut self) -> Result<Document, Ar7Error> {
        let mut leading_trivia = Vec::new();
        let mut entries = Vec::new();
        let mut trailing_trivia = Vec::new();

        // collect leading trivia
        while let Some(trivia) = self.try_collect_trivia()? {
            leading_trivia.push(trivia);
        }

        // parse entries
        while !self.is_at_end() {
            if let Some(entry) = self.try_parse_entry()? {
                entries.push(entry);
            } else if let Some(trivia) = self.try_collect_trivia()? {
                trailing_trivia.push(trivia);
            } else {
                break;
            }
        }

        // collect any trailing trivia
        while let Some(trivia) = self.try_collect_trivia()? {
            trailing_trivia.push(trivia);
        }

        Ok(Document {
            leading_trivia,
            entries,
            trailing_trivia,
        })
    }

    fn try_collect_trivia(&mut self) -> Result<Option<Trivia>, Ar7Error> {
        let tok = self.peek();
        match &tok.kind {
            TokenKind::Whitespace(s) => {
                let t = Trivia::Whitespace(s.clone());
                self.advance();
                Ok(Some(t))
            }
            TokenKind::LineComment(s) => {
                let t = Trivia::LineComment(s.clone());
                self.advance();
                Ok(Some(t))
            }
            TokenKind::BlockComment(s) => {
                let t = Trivia::BlockComment(s.clone());
                self.advance();
                Ok(Some(t))
            }
            _ => Ok(None),
        }
    }

    fn try_parse_entry(&mut self) -> Result<Option<Entry>, Ar7Error> {
        // skip trivia
        while self.skip_trivia() {}

        if self.is_at_end() {
            return Ok(None);
        }

        let key = self.parse_key()?;
        let key_span = self.peek_prev().span;

        // skip trivia after key
        while self.skip_trivia() {}

        if self.check(&TokenKind::LBrace) {
            // block entry: key { ... }
            self.advance(); // consume {
            let value = self.parse_block_entries(None)?;
            // skip trivia before semicolon (blocks don't have semicolons, but check)
            while self.skip_trivia() {}
            // blocks don't end with semicolons
            Ok(Some(Entry {
                key,
                value,
                span: Some(key_span),
            }))
        } else if self.check(&TokenKind::Equals) {
            self.advance(); // consume =
            while self.skip_trivia() {}
            let value = self.parse_value()?;
            while self.skip_trivia() {}
            // expect semicolon
            if self.check(&TokenKind::Semicolon) {
                self.advance();
            }
            Ok(Some(Entry {
                key,
                value,
                span: Some(key_span),
            }))
        } else {
            Err(self.unexpected_token(&["'{'", "'='"], &key))
        }
    }

    fn parse_key(&mut self) -> Result<String, Ar7Error> {
        match &self.peek().kind {
            TokenKind::Identifier(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            TokenKind::Boolean(b) => {
                let s = if *b {
                    "yes".to_string()
                } else {
                    "no".to_string()
                };
                self.advance();
                Ok(s)
            }
            _ => Err(self.unexpected_token(&["identifier"], self.peek().kind.name())),
        }
    }

    fn parse_value(&mut self) -> Result<Value, Ar7Error> {
        self.parse_value_with_depth(0)
    }

    fn parse_single_value(&mut self, depth: usize) -> Result<Value, Ar7Error> {
        if depth > MAX_NESTING_DEPTH {
            return Err(Ar7Error::general("maximum nesting depth exceeded"));
        }

        while self.skip_trivia() {}

        if self.is_at_end() {
            return Err(self.unexpected_eof(&["value"]));
        }

        match &self.peek().kind.clone() {
            TokenKind::String { value, raw } => {
                let v = Value::String(StringValue {
                    value: value.clone(),
                    raw: raw.clone(),
                });
                self.advance();
                Ok(v)
            }
            TokenKind::Integer { value, raw } => {
                let v = Value::Integer(IntegerValue {
                    value: *value,
                    raw: raw.clone(),
                });
                self.advance();
                Ok(v)
            }
            TokenKind::Number { value, raw } => {
                let v = Value::Number(NumberValue {
                    value: value.clone(),
                    raw: raw.clone(),
                });
                self.advance();
                Ok(v)
            }
            TokenKind::Boolean(b) => {
                let raw = if *b {
                    "yes".to_string()
                } else {
                    "no".to_string()
                };
                let v = Value::Boolean(BooleanValue { value: *b, raw });
                self.advance();
                Ok(v)
            }
            TokenKind::Duration { value, unit, raw } => {
                let v = Value::Duration(DurationValue {
                    value: *value,
                    unit: unit.clone(),
                    raw: raw.clone(),
                });
                self.advance();
                Ok(v)
            }
            TokenKind::LBrace => {
                self.advance(); // consume {
                let obj = self.parse_block_entries(Some(depth))?;
                Ok(obj)
            }
            TokenKind::Identifier(s) => {
                let ident = s.clone();
                self.advance();
                if ident.contains(':') && is_mac_address(&ident) {
                    Ok(Value::MacAddress(MacAddressValue { value: ident }))
                } else {
                    Ok(Value::Identifier(IdentifierValue { value: ident }))
                }
            }
            kind => Err(self.unexpected_token(&["value"], kind.name())),
        }
    }

    fn parse_value_with_depth(&mut self, depth: usize) -> Result<Value, Ar7Error> {
        let value = self.parse_single_value(depth)?;

        // Check for list continuation (comma)
        let mut items = vec![value];
        while self.skip_trivia() {}
        while self.check(&TokenKind::Comma) {
            if items.len() >= MAX_LIST_LENGTH {
                return Err(Ar7Error::general("maximum list length exceeded"));
            }
            self.advance(); // consume comma
            while self.skip_trivia() {}
            if self.is_at_end()
                || self.check(&TokenKind::Semicolon)
                || self.check(&TokenKind::RBrace)
            {
                break;
            }
            let item = self.parse_single_value(depth)?;
            items.push(item);
            while self.skip_trivia() {}
        }

        if items.len() == 1 {
            Ok(items.into_iter().next().unwrap())
        } else {
            Ok(Value::List(ListValue { items }))
        }
    }

    fn parse_block_entries(&mut self, depth: Option<usize>) -> Result<Value, Ar7Error> {
        let d = depth.map(|d| d + 1).unwrap_or(1);
        if d > MAX_NESTING_DEPTH {
            return Err(Ar7Error::general("maximum nesting depth exceeded"));
        }

        let mut entries = Vec::new();

        loop {
            while self.skip_trivia() {}

            if self.is_at_end() {
                if depth.is_some() {
                    return Err(self.unexpected_eof(&["'}'", "entry"]));
                }
                break;
            }

            if self.check(&TokenKind::RBrace) {
                self.advance(); // consume }
                break;
            }

            // Try to parse an entry inside the block
            let key = self.parse_key()?;
            let key_span = self.peek_prev().span;

            while self.skip_trivia() {}

            if self.check(&TokenKind::LBrace) {
                // nested block
                self.advance(); // consume {
                let value = self.parse_block_entries(Some(d))?;
                entries.push(Entry {
                    key,
                    value,
                    span: Some(key_span),
                });
            } else if self.check(&TokenKind::Equals) {
                self.advance(); // consume =
                while self.skip_trivia() {}
                let value = self.parse_value_with_depth(d)?;
                while self.skip_trivia() {}
                if self.check(&TokenKind::Semicolon) {
                    self.advance();
                }
                entries.push(Entry {
                    key,
                    value,
                    span: Some(key_span),
                });
            } else {
                return Err(self.unexpected_token(&["'{'", "'='"], self.peek().kind.name()));
            }
        }

        Ok(Value::Object(ObjectValue { entries }))
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
            span: Span::dummy(),
        })
    }

    fn peek_prev(&self) -> &Token {
        if self.pos == 0 {
            &self.tokens[0]
        } else {
            &self.tokens[self.pos - 1]
        }
    }

    fn advance(&mut self) -> &Token {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Eof
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.tokens[self.pos].kind) == std::mem::discriminant(kind)
    }

    fn skip_trivia(&mut self) -> bool {
        if self.is_at_end() {
            return false;
        }
        match &self.tokens[self.pos].kind {
            TokenKind::Whitespace(_) | TokenKind::LineComment(_) | TokenKind::BlockComment(_) => {
                self.pos += 1;
                true
            }
            _ => false,
        }
    }

    fn unexpected_token(&self, expected: &[&str], found: &str) -> Ar7Error {
        let tok = self.peek();
        let span = tok.span;
        let line_start = self.source[..span.start]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);
        let line_end = self.source[span.end..]
            .find('\n')
            .map(|p| span.end + p)
            .unwrap_or(self.source.len());
        let _line_text = &self.source[line_start..line_end];

        Ar7Error::unexpected_token(
            expected.to_vec(),
            found,
            self.source.to_string(),
            miette::SourceSpan::new(span.start.into(), (span.end - span.start).into()),
        )
    }

    fn unexpected_eof(&self, expected: &[&str]) -> Ar7Error {
        let pos = self.source.len();
        Ar7Error::unexpected_eof(
            expected.to_vec(),
            self.source.to_string(),
            miette::SourceSpan::new(pos.into(), 0.into()),
        )
    }
}

fn is_mac_address(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 6 {
        return false;
    }
    parts
        .iter()
        .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

pub fn parse(input: &str) -> Result<Document, Ar7Error> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| Ar7Error::General {
        message: e.to_string(),
    })?;

    let mut parser = Parser::new(input, tokens);
    parser.parse_document()
}
