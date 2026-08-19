#[cfg(test)]
mod tests {
    use ar7json::lexer::{Lexer, TokenKind};

    fn tokenize(input: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn identifiers() {
        let tokens = tokenize("foo bar_baz test-123 dot.separated");
        let ids: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::Identifier(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(ids, vec!["foo", "bar_baz", "test-123", "dot.separated"]);
    }

    #[test]
    fn integers() {
        let tokens = tokenize("0 1 123 -1 1500");
        let ints: Vec<i64> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::Integer { value, .. } => Some(*value),
                _ => None,
            })
            .collect();
        assert_eq!(ints, vec![0, 1, 123, -1, 1500]);
    }

    #[test]
    fn integer_raw_preserved() {
        let tokens = tokenize("001");
        let raws: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::Integer { raw, .. } => Some(raw.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(raws, vec!["001"]);
    }

    #[test]
    fn floats() {
        let tokens = tokenize("1.5 -0.5 3.14");
        let nums: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::Number { raw, .. } => Some(raw.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(nums, vec!["1.5", "-0.5", "3.14"]);
    }

    #[test]
    fn strings() {
        let tokens = tokenize(r#""hello" "world""#);
        let strs: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::String { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(strs, vec!["hello", "world"]);
    }

    #[test]
    fn escaped_strings() {
        let tokens = tokenize(r#""foo \"bar\"" "C:\\foo\\bar""#);
        let strs: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::String { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(strs, vec![r#"foo "bar""#, r"C:\foo\bar"]);
    }

    #[test]
    fn booleans() {
        let tokens = tokenize("yes no");
        let bools: Vec<bool> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::Boolean(b) => Some(*b),
                _ => None,
            })
            .collect();
        assert_eq!(bools, vec![true, false]);
    }

    #[test]
    fn durations() {
        let tokens = tokenize("1m 30s 2h 7d 500ms");
        let durs: Vec<(i64, &str)> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::Duration { value, unit, .. } => Some((*value, unit.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(durs.len(), 5);
        assert_eq!(durs[0], (1, "m"));
        assert_eq!(durs[1], (30, "s"));
        assert_eq!(durs[2], (2, "h"));
        assert_eq!(durs[3], (7, "d"));
        assert_eq!(durs[4], (500, "ms"));
    }

    #[test]
    fn line_comment() {
        let tokens = tokenize("// this is a comment\n");
        let has_comment = tokens
            .iter()
            .any(|t| matches!(t, TokenKind::LineComment(_)));
        assert!(has_comment);
    }

    #[test]
    fn block_comment() {
        let tokens = tokenize("/* block\n comment */");
        let has_comment = tokens
            .iter()
            .any(|t| matches!(t, TokenKind::BlockComment(_)));
        assert!(has_comment);
    }

    #[test]
    fn braces_and_punctuation() {
        let tokens = tokenize("{ } = , ;");
        let kinds: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::LBrace => Some("LBrace"),
                TokenKind::RBrace => Some("RBrace"),
                TokenKind::Equals => Some("Equals"),
                TokenKind::Comma => Some("Comma"),
                TokenKind::Semicolon => Some("Semicolon"),
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["LBrace", "RBrace", "Equals", "Comma", "Semicolon"]
        );
    }

    #[test]
    fn crlf_whitespace() {
        let tokens = tokenize("a\r\nb\r\n");
        let ws_count = tokens
            .iter()
            .filter(|t| matches!(t, TokenKind::Whitespace(_)))
            .count();
        assert!(ws_count > 0);
    }

    #[test]
    fn utf8_content() {
        let tokens = tokenize(r#""Ünïcödé""#);
        let strs: Vec<&str> = tokens
            .iter()
            .filter_map(|t| match t {
                TokenKind::String { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(strs, vec!["Ünïcödé"]);
    }

    #[test]
    fn nested_braces() {
        let tokens = tokenize("a { b { c = 1; } }");
        let lbrace_count = tokens
            .iter()
            .filter(|t| matches!(t, TokenKind::LBrace))
            .count();
        let rbrace_count = tokens
            .iter()
            .filter(|t| matches!(t, TokenKind::RBrace))
            .count();
        assert_eq!(lbrace_count, 2);
        assert_eq!(rbrace_count, 2);
    }
}
