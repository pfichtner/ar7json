use std::fmt::Write;

use crate::ast::*;
use crate::error::Ar7Error;

const INDENT: &str = "    ";

pub fn serialize(doc: &Document) -> Result<String, Ar7Error> {
    let mut output = String::with_capacity(4096);

    // leading trivia
    for t in &doc.leading_trivia {
        serialize_trivia(&mut output, t)?;
    }

    // entries
    for (i, entry) in doc.entries.iter().enumerate() {
        if i > 0 {
            // blank line between top-level blocks
            if matches!(entry.value, Value::Object(_)) {
                output.push('\n');
            }
        }
        serialize_entry(&mut output, entry, 0)?;
    }

    // trailing trivia
    for t in &doc.trailing_trivia {
        serialize_trivia(&mut output, t)?;
    }

    // ensure final newline
    if !output.ends_with('\n') {
        output.push('\n');
    }

    Ok(output)
}

fn serialize_trivia(output: &mut String, trivia: &Trivia) -> Result<(), Ar7Error> {
    match trivia {
        Trivia::Whitespace(s) => output.push_str(s),
        Trivia::LineComment(s) => {
            output.push_str("//");
            output.push_str(s);
        }
        Trivia::BlockComment(s) => {
            output.push_str("/*");
            output.push_str(s);
            output.push_str("*/");
        }
    }
    Ok(())
}

fn serialize_entry(output: &mut String, entry: &Entry, depth: usize) -> Result<(), Ar7Error> {
    let indent = repeat_indent(depth);
    write!(output, "{}{} ", indent, entry.key).unwrap();

    match &entry.value {
        Value::Object(obj) => {
            output.push_str("{\n");
            for e in &obj.entries {
                serialize_entry(output, e, depth + 1)?;
            }
            write!(output, "{}}}", indent).unwrap();
            output.push('\n');
        }
        _ => {
            output.push_str("= ");
            serialize_value(output, &entry.value, depth)?;
            output.push_str(";\n");
        }
    }

    Ok(())
}

fn serialize_value(output: &mut String, value: &Value, depth: usize) -> Result<(), Ar7Error> {
    match value {
        Value::String(s) => {
            output.push_str(&s.raw);
        }
        Value::Integer(i) => {
            output.push_str(&i.raw);
        }
        Value::Number(n) => {
            output.push_str(&n.raw);
        }
        Value::Boolean(b) => {
            output.push_str(&b.raw);
        }
        Value::Identifier(id) => {
            output.push_str(&id.value);
        }
        Value::Duration(d) => {
            output.push_str(&d.raw);
        }
        Value::IpAddress(ip) => {
            output.push_str(&ip.value);
        }
        Value::MacAddress(mac) => {
            output.push_str(&mac.value);
        }
        Value::List(list) => {
            serialize_list(output, &list.items, depth)?;
        }
        Value::Object(obj) => {
            output.push_str("{\n");
            for e in &obj.entries {
                serialize_entry(output, e, depth + 1)?;
            }
            write!(output, "{}}}", repeat_indent(depth)).unwrap();
        }
        Value::Raw(r) => {
            output.push_str(&r.text);
        }
    }
    Ok(())
}

fn serialize_list(output: &mut String, items: &[Value], depth: usize) -> Result<(), Ar7Error> {
    if items.is_empty() {
        output.push_str("");
        return Ok(());
    }

    if items.len() == 1 {
        serialize_value(output, &items[0], depth)?;
        return Ok(());
    }

    // Check if any item is complex (object or nested list)
    let has_complex = items
        .iter()
        .any(|v| matches!(v, Value::Object(_) | Value::List(_)));
    let total_len: usize = items.iter().map(value_estimate_len).sum();

    if !has_complex && total_len < 80 {
        // inline
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            serialize_value(output, item, depth)?;
        }
    } else {
        // multiline
        let indent = repeat_indent(depth + 1);
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                output.push_str(",\n");
                output.push_str(&indent);
            } else {
                output.push_str(&indent);
            }
            serialize_value(output, item, depth + 1)?;
        }
        output.push('\n');
        output.push_str(&repeat_indent(depth));
    }

    Ok(())
}

fn value_estimate_len(value: &Value) -> usize {
    match value {
        Value::String(s) => s.raw.len(),
        Value::Integer(i) => i.raw.len(),
        Value::Number(n) => n.raw.len(),
        Value::Boolean(b) => b.raw.len(),
        Value::Identifier(id) => id.value.len(),
        Value::Duration(d) => d.raw.len(),
        Value::IpAddress(ip) => ip.value.len(),
        Value::MacAddress(mac) => mac.value.len(),
        Value::List(list) => {
            let inner: usize = list.items.iter().map(value_estimate_len).sum();
            inner + list.items.len() * 2
        }
        Value::Object(obj) => {
            let inner: usize = obj
                .entries
                .iter()
                .map(|e| e.key.len() + value_estimate_len(&e.value))
                .sum();
            inner + obj.entries.len() * 4
        }
        Value::Raw(r) => r.text.len(),
    }
}

fn repeat_indent(depth: usize) -> String {
    INDENT.repeat(depth)
}
