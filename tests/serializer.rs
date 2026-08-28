#[cfg(test)]
mod tests {
    use ar7json::{parse, serialize};

    #[test]
    fn top_level_comment_preserved() {
        let input = "// top comment\nfoo = 42;\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        assert!(output.contains("// top comment"), "top-level comment lost: {}", output);
        assert!(output.contains("foo = 42;"), "value lost: {}", output);
    }

    #[test]
    fn comment_inside_block_indented() {
        let input = "block {\n    // inside\n    key = value;\n}\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.iter().any(|l| l.trim() == "// inside"),
            "comment missing or not indented: {}", output);
        let comment_line = lines.iter().find(|l| l.contains("// inside")).unwrap();
        assert!(comment_line.starts_with("    "),
            "comment should be indented 4 spaces, got: {:?}", comment_line);
    }

    #[test]
    fn block_comment_inside_block_indented() {
        let input = "block {\n    /* block comment */\n    key = value;\n}\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        let comment_line = lines.iter().find(|l| l.contains("block comment")).unwrap();
        assert!(comment_line.starts_with("    "),
            "block comment should be indented 4 spaces, got: {:?}", comment_line);
    }

    #[test]
    fn deeply_nested_comment_indented() {
        let input = "a {\n    b {\n        // deep comment\n        c = 1;\n    }\n}\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        let comment_line = lines.iter().find(|l| l.contains("deep comment")).unwrap();
        assert!(comment_line.starts_with("        "),
            "deeply nested comment should be indented 8 spaces, got: {:?}", comment_line);
    }

    #[test]
    fn consecutive_comments_preserved() {
        let input = "// first\n// second\nfoo = 1;\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        assert!(output.contains("// first"), "first comment lost: {}", output);
        assert!(output.contains("// second"), "second comment lost: {}", output);
    }

    #[test]
    fn inline_trailing_comment_preserved() {
        let input = "foo = 1; // trailing\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        assert!(output.contains("// trailing"), "trailing comment lost: {}", output);
        let line = output.lines().find(|l| l.contains("foo")).unwrap();
        assert!(line.contains("// trailing"),
            "trailing comment should be on same line as value: {:?}", line);
    }

    #[test]
    fn comment_between_entries() {
        let input = "a = 1;\n// between\nb = 2;\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        assert!(output.contains("// between"), "comment between entries lost: {}", output);
        let lines: Vec<&str> = output.lines().collect();
        let comment_idx = lines.iter().position(|l| l.contains("between")).unwrap();
        let b_idx = lines.iter().position(|l| l.starts_with("b")).unwrap();
        assert!(comment_idx < b_idx, "comment should come before b");
    }

    #[test]
    fn comment_before_closing_brace() {
        let input = "block {\n    key = value;\n    // before close\n}\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        let comment_idx = lines.iter().position(|l| l.contains("before close")).unwrap();
        let close_idx = lines.iter().position(|l| l.trim() == "}").unwrap();
        assert!(comment_idx < close_idx, "comment should come before closing brace");
    }

    #[test]
    fn block_comment_top_level() {
        let input = "/* header comment */\nfoo = 42;\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        assert!(output.contains("/* header comment */"), "block comment lost: {}", output);
    }

    #[test]
    fn no_extra_newlines_after_format() {
        let input = "foo = 1;\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        assert_eq!(output, "foo = 1;\n", "extra newlines added: {:?}", output);
    }

    #[test]
    fn no_double_newlines_after_trailing_comment() {
        let input = "foo = 1; // trailing\n";
        let doc = parse(input).unwrap();
        let output = serialize(&doc).unwrap();
        assert_eq!(output, "foo = 1; // trailing\n",
            "double newline after trailing comment: {:?}", output);
    }

    #[test]
    fn format_real01_preserves_comments() {
        let input = std::fs::read_to_string("tests/fixtures/real-world/real01.ar7").unwrap();
        let doc = parse(&input).unwrap();
        let output = serialize(&doc).unwrap();
        assert!(output.contains("// Configuration mode"),
            "real01 comment lost after format: {}", output);
        assert!(output.contains("/*"),
            "real01 block comment lost after format: {}", output);
    }

    #[test]
    fn format_roundtrip_idempotent() {
        let input = "// comment\nblock {\n    // inner\n    key = value;\n}\n";
        let doc = parse(input).unwrap();
        let out1 = serialize(&doc).unwrap();
        let doc2 = parse(&out1).unwrap();
        let out2 = serialize(&doc2).unwrap();
        assert_eq!(out1, out2, "format is not idempotent");
    }

}
