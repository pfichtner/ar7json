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

    #[test]
    fn root_not_an_object_rejected() {
        let result = json_to_document(&serde_json::json!("just a string"));
        assert!(result.is_err());
    }

    #[test]
    fn root_array_rejected() {
        let result = json_to_document(&serde_json::json!([1, 2, 3]));
        assert!(result.is_err());
    }

    #[test]
    fn missing_format_field_rejected() {
        let result = json_to_document(&serde_json::json!({
            "version": 1,
            "document": { "entries": [] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn missing_version_field_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "document": { "entries": [] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn missing_document_field_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1
        }));
        assert!(result.is_err());
    }

    #[test]
    fn document_not_an_object_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": "not an object"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn document_missing_entries_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": {}
        }));
        assert!(result.is_err());
    }

    #[test]
    fn entries_not_an_array_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": "not an array" }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn entry_not_an_object_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [42] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn entry_missing_key_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "value": { "type": "integer", "value": 1, "raw": "1" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn entry_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{ "key": "foo" }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn value_not_an_object_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": "not an object"
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn value_missing_type_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "value": "hello" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn unknown_value_type_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "type": "bogus", "value": "x" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn ip_address_roundtrip() {
        let doc = parse("wan = 192.168.1.1;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn mac_address_roundtrip() {
        let doc = parse("mac = ab:cd:ef:01:23:45;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn raw_value_roundtrip() {
        let json = serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "raw",
                "value": { "type": "raw", "text": "some arbitrary text" }
            }] }
        });
        let doc = json_to_document(&json).unwrap();
        assert_eq!(doc.entries.len(), 1);
        if let Value::Raw(r) = &doc.entries[0].value {
            assert_eq!(r.text, "some arbitrary text");
        } else {
            panic!("expected raw value");
        }
        let json2 = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json2).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn number_json_roundtrip() {
        let doc = parse("flt = 3.14;").unwrap();
        let json = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn simple_json_with_list() {
        let input = r#"foo = "a", "b", "c";"#;
        let doc = parse(input).unwrap();
        let simple = ar7json::json::document_to_simple_json(&doc).unwrap();
        let obj = simple.as_object().unwrap();
        let foo = obj.get("foo").unwrap();
        assert!(foo.is_array());
        assert_eq!(foo.as_array().unwrap().len(), 3);
    }

    #[test]
    fn simple_json_with_ip_address() {
        let input = "wan = 192.168.1.1;";
        let doc = parse(input).unwrap();
        let simple = ar7json::json::document_to_simple_json(&doc).unwrap();
        let obj = simple.as_object().unwrap();
        assert_eq!(obj.get("wan").unwrap(), "192.168.1.1");
    }

    #[test]
    fn simple_json_with_mac_address() {
        let input = "mac = ab:cd:ef:01:23:45;";
        let doc = parse(input).unwrap();
        let simple = ar7json::json::document_to_simple_json(&doc).unwrap();
        let obj = simple.as_object().unwrap();
        assert_eq!(obj.get("mac").unwrap(), "ab:cd:ef:01:23:45");
    }

    #[test]
    fn simple_json_with_raw_value() {
        let json = serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "raw",
                "value": { "type": "raw", "text": "some text" }
            }] }
        });
        let doc = json_to_document(&json).unwrap();
        let simple = ar7json::json::document_to_simple_json(&doc).unwrap();
        let obj = simple.as_object().unwrap();
        assert_eq!(obj.get("raw").unwrap(), "some text");
    }

    #[test]
    fn integer_json_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "type": "integer", "raw": "42" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn number_json_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "type": "number", "raw": "3.14" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn boolean_json_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "type": "boolean", "raw": "yes" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn raw_json_missing_text_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "type": "raw" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn ip_address_from_json_roundtrip() {
        let json = serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "wan",
                "value": { "type": "ip_address", "value": "192.168.1.1" }
            }] }
        });
        let doc = json_to_document(&json).unwrap();
        let json_out = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json_out).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn mac_address_from_json_roundtrip() {
        let json = serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "mac",
                "value": { "type": "mac_address", "value": "ab:cd:ef:01:23:45" }
            }] }
        });
        let doc = json_to_document(&json).unwrap();
        let json_out = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json_out).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn list_from_json_roundtrip() {
        let json = serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "items",
                "value": {
                    "type": "list",
                    "items": [
                        { "type": "integer", "value": 1, "raw": "1" },
                        { "type": "integer", "value": 2, "raw": "2" }
                    ]
                }
            }] }
        });
        let doc = json_to_document(&json).unwrap();
        let json_out = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json_out).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn object_from_json_roundtrip() {
        let json = serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "block",
                "value": {
                    "type": "object",
                    "entries": [{
                        "key": "inner",
                        "value": { "type": "integer", "value": 42, "raw": "42" }
                    }]
                }
            }] }
        });
        let doc = json_to_document(&json).unwrap();
        let json_out = document_to_json(&doc).unwrap();
        let doc2 = json_to_document(&json_out).unwrap();
        entries_equal(&doc.entries, &doc2.entries);
    }

    #[test]
    fn ip_address_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "wan",
                "value": { "type": "ip_address" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn mac_address_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "mac",
                "value": { "type": "mac_address" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn list_missing_items_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "items",
                "value": { "type": "list" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn list_items_not_array_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "items",
                "value": { "type": "list", "items": "not_array" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn object_missing_entries_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "block",
                "value": { "type": "object" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn object_entries_not_array_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "block",
                "value": { "type": "object", "entries": "not_array" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn identifier_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "type": "identifier" }
            }] }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn duration_missing_value_rejected() {
        let result = json_to_document(&serde_json::json!({
            "format": "ar7json",
            "version": 1,
            "document": { "entries": [{
                "key": "foo",
                "value": { "type": "duration", "unit": "m", "raw": "1m" }
            }] }
        }));
        assert!(result.is_err());
    }

}
