use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    pub fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Trivia {
    Whitespace(String),
    LineComment(String),
    BlockComment(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub leading_trivia: Vec<Trivia>,
    pub entries: Vec<Entry>,
    pub trailing_trivia: Vec<Trivia>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub key: String,
    pub value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Value {
    #[serde(rename = "string")]
    String(StringValue),
    #[serde(rename = "integer")]
    Integer(IntegerValue),
    #[serde(rename = "number")]
    Number(NumberValue),
    #[serde(rename = "boolean")]
    Boolean(BooleanValue),
    #[serde(rename = "identifier")]
    Identifier(IdentifierValue),
    #[serde(rename = "duration")]
    Duration(DurationValue),
    #[serde(rename = "ip_address")]
    IpAddress(IpAddressValue),
    #[serde(rename = "mac_address")]
    MacAddress(MacAddressValue),
    #[serde(rename = "list")]
    List(ListValue),
    #[serde(rename = "object")]
    Object(ObjectValue),
    #[serde(rename = "raw")]
    Raw(RawValue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringValue {
    pub value: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegerValue {
    pub value: i64,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberValue {
    pub value: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanValue {
    pub value: bool,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifierValue {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DurationValue {
    pub value: i64,
    pub unit: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IpAddressValue {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacAddressValue {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListValue {
    pub items: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectValue {
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawValue {
    pub text: String,
}
