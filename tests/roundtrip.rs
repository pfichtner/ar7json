#[cfg(test)]
mod tests {
    use ar7json::ast::*;
    use ar7json::{parse, serialize};
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
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

    fn roundtrip(ar7_path: &str) {
        let input_path = fixtures_dir().join(ar7_path);
        let input = fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", input_path.display(), e));

        let mut doc1 = parse(&input)
            .unwrap_or_else(|e| panic!("failed to parse {}: {}", input_path.display(), e));
        let json = ar7json::json::document_to_json(&doc1).unwrap();
        let doc2 = ar7json::json::json_to_document(&json).unwrap();
        let output = serialize(&doc2).unwrap();
        let mut doc3 = parse(&output).unwrap_or_else(|e| {
            panic!(
                "failed to re-parse serialized output for {}:\n{}\nerror: {}",
                input_path.display(),
                output,
                e
            )
        });

        strip_spans_doc(&mut doc1);
        strip_spans_doc(&mut doc3);

        assert_eq!(
            doc1.entries.len(),
            doc3.entries.len(),
            "entry count mismatch for {}",
            input_path.display()
        );
        entries_equal(&doc1.entries, &doc3.entries);
    }

    fn stronger_roundtrip(ar7_path: &str) {
        let input_path = fixtures_dir().join(ar7_path);
        let input = fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", input_path.display(), e));

        let doc1 = parse(&input).unwrap();
        let json1 = ar7json::json::document_to_json(&doc1).unwrap();

        let ar7 = serialize(&doc1).unwrap();
        let doc2 = parse(&ar7).unwrap();
        let json2 = ar7json::json::document_to_json(&doc2).unwrap();

        assert_eq!(
            json1,
            json2,
            "JSON mismatch for {} (stronger invariant)",
            input_path.display()
        );
    }

    #[test]
    fn roundtrip_minimal() {
        roundtrip("minimal.ar7");
    }
    #[test]
    fn roundtrip_nested() {
        roundtrip("nested.ar7");
    }
    #[test]
    fn roundtrip_strings() {
        roundtrip("strings.ar7");
    }
    #[test]
    fn roundtrip_lists() {
        roundtrip("lists.ar7");
    }
    #[test]
    fn roundtrip_durations() {
        roundtrip("durations.ar7");
    }
    #[test]
    fn roundtrip_real01() {
        roundtrip("real-world/real01.ar7");
    }
    #[test]
    fn roundtrip_real02() {
        roundtrip("real-world/real02.ar7");
    }
    #[test]
    fn roundtrip_real03() {
        roundtrip("real-world/real03.ar7");
    }
    #[test]
    fn roundtrip_real04() {
        roundtrip("real-world/real04.ar7");
    }
    #[test]
    fn roundtrip_real05() {
        roundtrip("real-world/real05.ar7");
    }

    #[test]
    fn stronger_minimal() {
        stronger_roundtrip("minimal.ar7");
    }
    #[test]
    fn stronger_nested() {
        stronger_roundtrip("nested.ar7");
    }
    #[test]
    fn stronger_strings() {
        stronger_roundtrip("strings.ar7");
    }
    #[test]
    fn stronger_lists() {
        stronger_roundtrip("lists.ar7");
    }
    #[test]
    fn stronger_durations() {
        stronger_roundtrip("durations.ar7");
    }
    #[test]
    fn stronger_real01() {
        stronger_roundtrip("real-world/real01.ar7");
    }
    #[test]
    fn stronger_real02() {
        stronger_roundtrip("real-world/real02.ar7");
    }
    #[test]
    fn stronger_real03() {
        stronger_roundtrip("real-world/real03.ar7");
    }
    #[test]
    fn stronger_real04() {
        stronger_roundtrip("real-world/real04.ar7");
    }
    #[test]
    fn stronger_real05() {
        stronger_roundtrip("real-world/real05.ar7");
    }

    #[test]
    fn serialize_minimal() {
        let input = fs::read_to_string(fixtures_dir().join("minimal.ar7")).unwrap();
        let doc = parse(&input).unwrap();
        let output = serialize(&doc).unwrap();
        let doc2 = parse(&output).unwrap();
        assert_eq!(doc.entries.len(), doc2.entries.len());
        assert_eq!(doc.entries[0].key, doc2.entries[0].key);
    }
}
