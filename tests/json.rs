#[cfg(test)]
mod tests {
    use ar7json::ast::*;
    use ar7json::json::{document_to_json, json_to_document};
    use ar7json::{parse, serialize};

    fn strip_spans_value(v: &mut Value) {
        match v {
            Value::Object(obj) => {
                for entry in &mut obj.entries {
                    entry.span = None;
                    strip_spans_value(&mut entry.value);
                }
            }
            Value::List(list) => {
                for item in &mut list.items {
                    strip_spans_value(item);
                }
            }
            _ => {}
        }
    }

    fn strip_spans_doc(doc: &mut Document) {
        for entry in &mut doc.entries {
            entry.span = None;
            strip_spans_value(&mut entry.value);
        }
    }

    fn values_eq(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Identifier(a), Value::Identifier(b)) => a == b,
            (Value::Duration(a), Value::Duration(b)) => a == b,
            (Value::IpAddress(a), Value::IpAddress(b)) => a == b,
            (Value::MacAddress(a), Value::MacAddress(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                a.items.len() == b.items.len()
                    && a.items
                        .iter()
                        .zip(b.items.iter())
                        .all(|(a, b)| values_eq(a, b))
            }
            (Value::Object(a), Value::Object(b)) => {
                a.entries.len() == b.entries.len()
                    && a.entries
                        .iter()
                        .zip(b.entries.iter())
                        .all(|(a, b)| a.key == b.key && values_eq(&a.value, &b.value))
            }
            (Value::Raw(a), Value::Raw(b)) => a == b,
            _ => false,
        }
    }

    fn entries_equal(a: &[Entry], b: &[Entry]) {
        assert_eq!(a.len(), b.len(), "entry count mismatch");
        for (i, (ea, eb)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(ea.key, eb.key, "key mismatch at entry {}", i);
            assert!(
                values_eq(&ea.value, &eb.value),
                "value mismatch at entry {} ('{}'):\n  left: {:?}\n  right: {:?}",
                i,
                ea.key,
                ea.value,
                eb.value
            );
        }
    }

    #[test]
    fn string_roundtrip() {
        let doc = parse(r#"foo = "hello";"#).unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn integer_roundtrip() {
        let doc = parse("foo = 42;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn boolean_roundtrip() {
        let doc = parse("foo = yes;\nbar = no;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn duration_roundtrip() {
        let doc = parse("foo = 1m;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn object_roundtrip() {
        let doc = parse("block { a = 1; b = 2; }").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn list_roundtrip() {
        let doc = parse(r#"foo = "a", "b", "c";"#).unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn nested_object_roundtrip() {
        let doc = parse("a { b { c = 1; } }").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn duplicate_keys_roundtrip() {
        let doc = parse("foo = 1;\nfoo = 2;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn negative_integer_roundtrip() {
        let doc = parse("foo = -7;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn full_ar7_roundtrip() {
        let input = std::fs::read_to_string("tests/fixtures/nested.ar7").unwrap();
        let mut doc = parse(&input).unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        let output = serialize(&doc2).unwrap();
        let mut doc3 = parse(&output).unwrap();

        strip_spans_doc(&mut doc);
        strip_spans_doc(&mut doc3);

        assert_eq!(doc.entries.len(), doc3.entries.len());
        for (a, b) in doc.entries.iter().zip(doc3.entries.iter()) {
            assert_eq!(a.key, b.key);
            assert!(values_eq(&a.value, &b.value));
        }
    }

    #[test]
    fn real_world_01_roundtrip() {
        let input = std::fs::read_to_string("tests/fixtures/real-world/real01.ar7").unwrap();
        let mut doc = parse(&input).unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        let output = serialize(&doc2).unwrap();
        let mut doc3 = parse(&output).unwrap();

        strip_spans_doc(&mut doc);
        strip_spans_doc(&mut doc3);

        assert_eq!(doc.entries.len(), doc3.entries.len());
        for (a, b) in doc.entries.iter().zip(doc3.entries.iter()) {
            assert_eq!(a.key, b.key);
            assert!(values_eq(&a.value, &b.value));
        }
    }

    #[test]
    fn json_deterministic() {
        let input = "foo = 42;\nbar = yes;";
        let doc = parse(input).unwrap();
        let json1 = document_to_json(&doc).unwrap();
        let json2 = document_to_json(&doc).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn serializer_deterministic() {
        let input = "foo = 42;\nbar = yes;";
        let doc = parse(input).unwrap();
        let out1 = serialize(&doc).unwrap();
        let out2 = serialize(&doc).unwrap();
        assert_eq!(out1, out2);
    }

    #[test]
    fn simple_json_mode() {
        let input = std::fs::read_to_string("tests/fixtures/nested.ar7").unwrap();
        let doc = parse(&input).unwrap();
        let simple = ar7json::json::document_to_simple_json(&doc).unwrap();
        let obj = simple.as_object().unwrap();
        assert!(obj.contains_key("meta"));
        assert!(obj.contains_key("ar7cfg"));
    }

    #[test]
    fn malformed_json_rejected() {
        let json = serde_json::json!({
            "type": "boolean",
            "value": "yes"
        });
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": {
                "entries": [{
                    "key": "foo",
                    "value": json
                }]
            }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn wrong_format_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "wrong",
            "version": 1,
            "document": { "entries": [] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn wrong_version_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 2,
            "document": { "entries": [] }
        }));
        assert!(result.is_err());
    }
}
