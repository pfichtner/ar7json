pub mod ast;
pub mod error;
pub mod json;
pub mod lexer;
pub mod parser;
pub mod serializer;

pub use ast::Document;
pub use error::Ar7Error;
pub use json::{document_to_json, document_to_simple_json, json_to_document};
pub use parser::parse;
pub use serializer::serialize;

pub const SYMLINK_NAMES: &[&str] = &["ar7-to-json", "json-to-ar7", "ar7-check", "ar7-fmt"];
