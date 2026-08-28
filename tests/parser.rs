#[cfg(test)]
mod tests {
    use ar7json::ast::*;
    use ar7json::parse;

    #[test]
    fn empty_document() {
        let doc = parse("").unwrap();
        assert!(doc.entries.is_empty());
    }

    #[test]
    fn single_assignment() {
        let doc = parse("foo = 42;").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "foo");
        assert!(matches!(doc.entries[0].value, Value::Integer(_)));
    }

    #[test]
    fn nested_block() {
        let doc = parse("block { key = value; }").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "block");
        if let Value::Object(obj) = &doc.entries[0].value {
            assert_eq!(obj.entries.len(), 1);
            assert_eq!(obj.entries[0].key, "key");
        } else {
            panic!("expected object");
        }
    }

    #[test]
    fn deeply_nested_blocks() {
        let doc = parse("a { b { c { d = 1; } } }").unwrap();
        assert_eq!(doc.entries.len(), 1);
        if let Value::Object(obj) = &doc.entries[0].value {
            assert_eq!(obj.entries.len(), 1);
            if let Value::Object(obj2) = &obj.entries[0].value {
                assert_eq!(obj2.entries.len(), 1);
                if let Value::Object(obj3) = &obj2.entries[0].value {
                    assert_eq!(obj3.entries.len(), 1);
                }
            }
        }
    }

    #[test]
    fn list_values() {
        let doc = parse("foo = \"a\", \"b\", \"c\";").unwrap();
        assert_eq!(doc.entries.len(), 1);
        if let Value::List(list) = &doc.entries[0].value {
            assert_eq!(list.items.len(), 3);
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn multiline_list() {
        let doc = parse("foo = \"a\",\n      \"b\",\n      \"c\";").unwrap();
        assert_eq!(doc.entries.len(), 1);
        if let Value::List(list) = &doc.entries[0].value {
            assert_eq!(list.items.len(), 3);
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn duplicate_keys() {
        let doc = parse("foo = 1;\nfoo = 2;").unwrap();
        assert_eq!(doc.entries.len(), 2);
        assert_eq!(doc.entries[0].key, "foo");
        assert_eq!(doc.entries[1].key, "foo");
    }

    #[test]
    fn comments_preserved() {
        let doc = parse("// line comment\nfoo = 1;\n/* block */\nbar = 2;").unwrap();
        assert_eq!(doc.entries.len(), 2);
        assert_eq!(doc.entries[0].key, "foo");
        assert_eq!(doc.entries[1].key, "bar");
    }

    #[test]
    fn mixed_values() {
        let doc = parse(
            r#"
            str = "hello";
            int = 42;
            neg = -7;
            bool_yes = yes;
            bool_no = no;
            dur = 1m;
            ident = some_value;
            flt = 3.14;
        "#,
        )
        .unwrap();
        assert_eq!(doc.entries.len(), 8);
        assert!(matches!(doc.entries[0].value, Value::String(_)));
        assert!(matches!(doc.entries[1].value, Value::Integer(_)));
        assert!(matches!(doc.entries[2].value, Value::Integer(_)));
        assert!(matches!(doc.entries[3].value, Value::Boolean(ref b) if b.value));
        assert!(matches!(doc.entries[4].value, Value::Boolean(ref b) if !b.value));
        assert!(matches!(doc.entries[5].value, Value::Duration(_)));
        assert!(matches!(doc.entries[6].value, Value::Identifier(_)));
        assert!(matches!(doc.entries[7].value, Value::Number(_)));
    }

    #[test]
    fn unknown_identifiers_accepted() {
        let doc = parse("completely_unknown_avm_setting = completely_unknown_value;").unwrap();
        assert_eq!(doc.entries.len(), 1);
        if let Value::Identifier(id) = &doc.entries[0].value {
            assert_eq!(id.value, "completely_unknown_value");
        }
    }

    #[test]
    fn unknown_blocks_accepted() {
        let doc = parse("future_config { new_option = new_value; }").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "future_config");
    }

    #[test]
    fn minimal_fixture() {
        let input = std::fs::read_to_string("tests/fixtures/minimal.ar7").unwrap();
        let doc = parse(&input).unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "meta");
    }

    #[test]
    fn nested_fixture() {
        let input = std::fs::read_to_string("tests/fixtures/nested.ar7").unwrap();
        let doc = parse(&input).unwrap();
        assert!(doc.entries.len() >= 2);
    }

    #[test]
    fn real_world_01() {
        let input = std::fs::read_to_string("tests/fixtures/real-world/real01.ar7").unwrap();
        let doc = parse(&input).unwrap();
        assert!(doc.entries.len() >= 2);
    }

    #[test]
    fn avm_export_header_stripped() {
        let input = "\
**** FRITZ!Box 7490 (UI) CONFIGURATION EXPORT
Password=$$$xxx
FirmwareVersion=113.07.59
**** CFGFILE:ar7.cfg
/*
 * /var/tmp.cfg
 * Sun Sep  1 11:05:45 2024
 */

meta { encoding = \"utf-8\"; }
ar7cfg {
    mode = dsldmode_bridge;
}
";
        let doc = parse(input).unwrap();
        assert_eq!(doc.entries.len(), 2);
        assert_eq!(doc.entries[0].key, "meta");
        assert_eq!(doc.entries[1].key, "ar7cfg");
    }

    #[test]
    fn plain_ar7_not_affected() {
        let input = "meta { encoding = \"utf-8\"; }\nfoo = 42;";
        let doc = parse(input).unwrap();
        assert_eq!(doc.entries.len(), 2);
        assert_eq!(doc.entries[0].key, "meta");
        assert_eq!(doc.entries[1].key, "foo");
    }

    #[test]
    fn unclosed_block_accepted() {
        let doc = parse("block { key = 1").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "block");
    }

    #[test]
    fn unexpected_token_after_key() {
        let result = parse("key 42");
        assert!(result.is_err());
    }

    #[test]
    fn boolean_as_key() {
        let doc = parse("yes = 42;").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "yes");
    }

    #[test]
    fn trailing_block_comment() {
        let doc = parse("foo = 1;\n/* trailing */\n").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "foo");
    }

    #[test]
    fn trailing_line_comment() {
        let doc = parse("foo = 1;\n// trailing\n").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "foo");
    }

    #[test]
    fn list_with_block_entries() {
        let doc = parse("items = {\n    a = 1;\n}, {\n    b = 2;\n};").unwrap();
        assert_eq!(doc.entries.len(), 1);
        if let Value::List(list) = &doc.entries[0].value {
            assert_eq!(list.items.len(), 2);
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn empty_value_after_equals() {
        let doc = parse("foo = ;").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "foo");
    }

    #[test]
    fn inline_block_comment_preserved() {
        let doc = parse("foo = 1; /* inline */").unwrap();
        assert_eq!(doc.entries.len(), 1);
        assert_eq!(doc.entries[0].key, "foo");
    }
}
