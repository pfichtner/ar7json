use crate::ast::*;
use crate::error::Ar7Error;
use serde_json::{json, Value as JsonValue};

pub fn document_to_json(doc: &Document) -> Result<JsonValue, Ar7Error> {
    let mut entries = Vec::new();

    for entry in &doc.entries {
        entries.push(entry_to_json(entry)?);
    }

    Ok(json!({
        "format": "ar7json",
        "version": 1,
        "document": {
            "entries": entries
        }
    }))
}

fn entry_to_json(entry: &Entry) -> Result<JsonValue, Ar7Error> {
    Ok(json!({
        "key": entry.key,
        "value": value_to_json(&entry.value)?
    }))
}

fn value_to_json(value: &Value) -> Result<JsonValue, Ar7Error> {
    match value {
        Value::String(s) => Ok(json!({
            "type": "string",
            "value": s.value,
            "raw": s.raw
        })),
        Value::Integer(i) => Ok(json!({
            "type": "integer",
            "value": i.value,
            "raw": i.raw
        })),
        Value::Number(n) => Ok(json!({
            "type": "number",
            "value": n.value,
            "raw": n.raw
        })),
        Value::Boolean(b) => Ok(json!({
            "type": "boolean",
            "value": b.value,
            "raw": b.raw
        })),
        Value::Identifier(id) => Ok(json!({
            "type": "identifier",
            "value": id.value
        })),
        Value::Duration(d) => Ok(json!({
            "type": "duration",
            "value": d.value,
            "unit": d.unit,
            "raw": d.raw
        })),
        Value::IpAddress(ip) => Ok(json!({
            "type": "ip_address",
            "value": ip.value
        })),
        Value::MacAddress(mac) => Ok(json!({
            "type": "mac_address",
            "value": mac.value
        })),
        Value::List(list) => {
            let items: Result<Vec<JsonValue>, _> = list.items.iter().map(value_to_json).collect();
            Ok(json!({
                "type": "list",
                "items": items?
            }))
        }
        Value::Object(obj) => {
            let entries: Result<Vec<JsonValue>, _> =
                obj.entries.iter().map(entry_to_json).collect();
            Ok(json!({
                "type": "object",
                "entries": entries?
            }))
        }
        Value::Raw(r) => Ok(json!({
            "type": "raw",
            "text": r.text
        })),
    }
}

pub fn json_to_document(value: &JsonValue) -> Result<Document, Ar7Error> {
    let obj = value
        .as_object()
        .ok_or_else(|| Ar7Error::invalid_json("root must be an object"))?;

    let format = obj
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Ar7Error::invalid_json("missing 'format' field"))?;
    if format != "ar7json" {
        return Err(Ar7Error::invalid_json(&format!(
            "expected format 'ar7json', got '{}'",
            format
        )));
    }

    let version = obj
        .get("version")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Ar7Error::invalid_json("missing 'version' field"))?;
    if version != 1 {
        return Err(Ar7Error::invalid_json(&format!(
            "unsupported version {}",
            version
        )));
    }

    let doc = obj
        .get("document")
        .ok_or_else(|| Ar7Error::invalid_json("missing 'document' field"))?;
    let doc_obj = doc
        .as_object()
        .ok_or_else(|| Ar7Error::invalid_json("'document' must be an object"))?;

    let entries_val = doc_obj
        .get("entries")
        .ok_or_else(|| Ar7Error::invalid_json("document missing 'entries'"))?;
    let entries_arr = entries_val
        .as_array()
        .ok_or_else(|| Ar7Error::invalid_json("'entries' must be an array"))?;

    let mut entries = Vec::new();
    for entry_val in entries_arr {
        entries.push(json_to_entry(entry_val)?);
    }

    Ok(Document {
        leading_trivia: Vec::new(),
        entries,
        trailing_trivia: Vec::new(),
    })
}

fn json_to_entry(value: &JsonValue) -> Result<Entry, Ar7Error> {
    let obj = value
        .as_object()
        .ok_or_else(|| Ar7Error::invalid_json("entry must be an object"))?;

    let key = obj
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Ar7Error::invalid_json("entry missing 'key'"))?
        .to_string();

    let val = obj
        .get("value")
        .ok_or_else(|| Ar7Error::invalid_json("entry missing 'value'"))?;

    Ok(Entry {
        key,
        value: json_to_value(val)?,
        leading_trivia: Vec::new(),
        trailing_trivia: Vec::new(),
        span: None,
    })
}

fn json_to_value(value: &JsonValue) -> Result<Value, Ar7Error> {
    let obj = value
        .as_object()
        .ok_or_else(|| Ar7Error::invalid_json("value must be an object"))?;

    let typ = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Ar7Error::invalid_json("value missing 'type'"))?;

    match typ {
        "string" => {
            let val = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Ar7Error::invalid_json("string value missing or invalid 'value'"))?;
            let raw = obj.get("raw").and_then(|v| v.as_str()).unwrap_or(val);
            Ok(Value::String(StringValue {
                value: val.to_string(),
                raw: raw.to_string(),
            }))
        }
        "integer" => {
            let val = obj.get("value").and_then(|v| v.as_i64()).ok_or_else(|| {
                Ar7Error::invalid_json("integer value missing or invalid 'value'")
            })?;
            let raw_str = val.to_string();
            let raw = obj.get("raw").and_then(|v| v.as_str()).unwrap_or(&raw_str);
            Ok(Value::Integer(IntegerValue {
                value: val,
                raw: raw.to_string(),
            }))
        }
        "number" => {
            let val = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Ar7Error::invalid_json("number value missing or invalid 'value'"))?;
            let raw = obj.get("raw").and_then(|v| v.as_str()).unwrap_or(val);
            Ok(Value::Number(NumberValue {
                value: val.to_string(),
                raw: raw.to_string(),
            }))
        }
        "boolean" => {
            let val = obj.get("value").and_then(|v| v.as_bool()).ok_or_else(|| {
                Ar7Error::invalid_json("boolean value missing or invalid 'value'")
            })?;
            let raw = if val {
                "yes".to_string()
            } else {
                "no".to_string()
            };
            Ok(Value::Boolean(BooleanValue { value: val, raw }))
        }
        "identifier" => {
            let val = obj.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
                Ar7Error::invalid_json("identifier value missing or invalid 'value'")
            })?;
            Ok(Value::Identifier(IdentifierValue {
                value: val.to_string(),
            }))
        }
        "duration" => {
            let val = obj.get("value").and_then(|v| v.as_i64()).ok_or_else(|| {
                Ar7Error::invalid_json("duration value missing or invalid 'value'")
            })?;
            let unit = obj
                .get("unit")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Ar7Error::invalid_json("duration missing 'unit'"))?;
            let raw_default = format!("{}{}", val, unit);
            let raw = obj
                .get("raw")
                .and_then(|v| v.as_str())
                .unwrap_or(&raw_default);
            Ok(Value::Duration(DurationValue {
                value: val,
                unit: unit.to_string(),
                raw: raw.to_string(),
            }))
        }
        "ip_address" => {
            let val = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Ar7Error::invalid_json("ip_address missing 'value'"))?;
            Ok(Value::IpAddress(IpAddressValue {
                value: val.to_string(),
            }))
        }
        "mac_address" => {
            let val = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Ar7Error::invalid_json("mac_address missing 'value'"))?;
            Ok(Value::MacAddress(MacAddressValue {
                value: val.to_string(),
            }))
        }
        "list" => {
            let items_val = obj
                .get("items")
                .ok_or_else(|| Ar7Error::invalid_json("list missing 'items'"))?;
            let items_arr = items_val
                .as_array()
                .ok_or_else(|| Ar7Error::invalid_json("'items' must be an array"))?;
            let mut items = Vec::new();
            for item in items_arr {
                items.push(json_to_value(item)?);
            }
            Ok(Value::List(ListValue { items }))
        }
        "object" => {
            let entries_val = obj
                .get("entries")
                .ok_or_else(|| Ar7Error::invalid_json("object missing 'entries'"))?;
            let entries_arr = entries_val
                .as_array()
                .ok_or_else(|| Ar7Error::invalid_json("'entries' must be an array"))?;
            let mut entries = Vec::new();
            for entry in entries_arr {
                entries.push(json_to_entry(entry)?);
            }
            Ok(Value::Object(ObjectValue { entries }))
        }
        "raw" => {
            let text = obj
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Ar7Error::invalid_json("raw missing 'text'"))?;
            Ok(Value::Raw(RawValue {
                text: text.to_string(),
            }))
        }
        _ => Err(Ar7Error::invalid_json(&format!(
            "unknown value type: '{}'",
            typ
        ))),
    }
}

/// Simplified JSON mode: flatten to simple key-value representation
pub fn document_to_simple_json(doc: &Document) -> Result<JsonValue, Ar7Error> {
    let mut root = serde_json::Map::new();

    for entry in &doc.entries {
        let val = simple_value(&entry.value)?;
        root.insert(entry.key.clone(), val);
    }

    Ok(JsonValue::Object(root))
}

fn simple_value(value: &Value) -> Result<JsonValue, Ar7Error> {
    match value {
        Value::String(s) => Ok(JsonValue::String(s.value.clone())),
        Value::Integer(i) => Ok(json!(i.value)),
        Value::Number(n) => Ok(JsonValue::String(n.value.clone())),
        Value::Boolean(b) => Ok(JsonValue::Bool(b.value)),
        Value::Identifier(id) => Ok(JsonValue::String(id.value.clone())),
        Value::Duration(d) => Ok(JsonValue::String(d.raw.clone())),
        Value::IpAddress(ip) => Ok(JsonValue::String(ip.value.clone())),
        Value::MacAddress(mac) => Ok(JsonValue::String(mac.value.clone())),
        Value::List(list) => {
            let items: Result<Vec<JsonValue>, _> = list.items.iter().map(simple_value).collect();
            Ok(JsonValue::Array(items?))
        }
        Value::Object(obj) => {
            let mut map = serde_json::Map::new();
            for entry in &obj.entries {
                map.insert(entry.key.clone(), simple_value(&entry.value)?);
            }
            Ok(JsonValue::Object(map))
        }
        Value::Raw(r) => Ok(JsonValue::String(r.text.clone())),
    }
}
