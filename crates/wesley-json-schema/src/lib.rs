//! JSON Schema 2020-12 for Wesley IR.
//!
//! Two things live here:
//!
//! - [`to_document_string`], the canonical printing every JSON Schema produced by a Wesley target
//!   goes through: a JSON value is its own syntax tree, so a document is built as a
//!   [`serde_json::Value`] and printed by `serde_json`, never assembled as text.
//! - [`emit_ir`], a generic emitter for plain GraphQL: one document per object and input type.
//!   (and interface). It understands no directives. A target that gives directives a meaning (for example
//!   `wesley-parley`) builds its own documents and prints them with [`to_document_string`].

#![deny(missing_docs)]

use serde_json::{json, Map, Value};
use wesley_core::{TypeDefinition, TypeKind, TypeReference, WesleyIR};

/// The JSON Schema dialect every document declares.
pub const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// The Wesley IR version this crate reads.
pub const IR_VERSION: &str = "1.0.0";

/// Why a document could not be emitted.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// The IR is a version this crate does not read.
    #[error("Wesley IR version {found} is not supported; this target reads {IR_VERSION}")]
    IrVersion {
        /// The version the IR declared.
        found: String,
    },
    /// A field's type names a type the IR does not define.
    #[error("{owner}.{field} names type {name}, which the IR does not define")]
    UnknownType {
        /// The type that declares the field.
        owner: String,
        /// The field.
        field: String,
        /// The missing type's name.
        name: String,
    },
}

/// How documents are identified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Prefix of every `$id`; a type's id is this prefix followed by the type's name.
    pub id_base: String,
}

/// One emitted document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// The file name the document is conventionally written to: `<TypeName>.schema.json`.
    pub file_name: String,
    /// The document.
    pub value: Value,
}

/// Prints a JSON value as a canonical document: two-space indentation, keys in insertion order,
/// non-ASCII characters written as themselves, and exactly one trailing newline.
#[must_use]
pub fn to_document_string(value: &Value) -> String {
    let mut out = serde_json::to_string_pretty(value)
        .unwrap_or_else(|_| unreachable!("a serde_json::Value always serialises"));
    out.push('\n');
    out
}

/// Emits one JSON Schema document per object, interface and input-object type of a plain GraphQL schema,
/// in type-name order. Directives are ignored.
///
/// # Errors
///
/// [`Error::IrVersion`] for an IR this crate does not read, and [`Error::UnknownType`] when a
/// field names an undefined type.
pub fn emit_ir(ir: &WesleyIR, options: &Options) -> Result<Vec<Document>, Error> {
    if ir.version != IR_VERSION {
        return Err(Error::IrVersion {
            found: ir.version.clone(),
        });
    }
    let mut types: Vec<&TypeDefinition> = ir
        .types
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                TypeKind::Object | TypeKind::InputObject | TypeKind::Interface
            )
        })
        .collect();
    types.sort_by(|a, b| a.name.cmp(&b.name));
    types
        .into_iter()
        .map(|t| {
            Ok(Document {
                file_name: format!("{}.schema.json", t.name),
                value: document(ir, t, options)?,
            })
        })
        .collect()
}

fn document(ir: &WesleyIR, t: &TypeDefinition, options: &Options) -> Result<Value, Error> {
    let mut doc = Map::new();
    doc.insert("$schema".into(), json!(DIALECT));
    doc.insert("$id".into(), json!(type_id(options, &t.name)));
    doc.insert("title".into(), json!(t.name));
    if let Some(description) = &t.description {
        doc.insert("description".into(), json!(description));
    }
    doc.insert("type".into(), json!("object"));
    let required: Vec<&str> = t
        .fields
        .iter()
        // An input field with a default may be omitted (GraphQL applies the default).
        .filter(|f| !f.r#type.nullable && f.default_value.is_none())
        .map(|f| f.name.as_str())
        .collect();
    if !required.is_empty() {
        doc.insert("required".into(), json!(required));
    }
    if t.kind == TypeKind::InputObject {
        // GraphQL rejects an input field the type does not declare; an output object may grow.
        doc.insert("additionalProperties".into(), json!(false));
    }
    let mut properties = Map::new();
    for field in &t.fields {
        let mut schema = type_schema(ir, options, &t.name, &field.name, &field.r#type)?;
        if let (Some(description), Value::Object(map)) = (&field.description, &mut schema) {
            map.insert("description".into(), json!(description));
        }
        properties.insert(field.name.clone(), schema);
    }
    doc.insert("properties".into(), Value::Object(properties));
    Ok(Value::Object(doc))
}

fn type_id(options: &Options, name: &str) -> String {
    let mut id = options.id_base.clone();
    id.push_str(name);
    id
}

/// The schema of a whole type reference: list wrappers outermost first, then the named leaf.
fn type_schema(
    ir: &WesleyIR,
    options: &Options,
    owner: &str,
    field: &str,
    r: &TypeReference,
) -> Result<Value, Error> {
    // (nullable) for each level, outermost first, ending with the leaf.
    let mut levels: Vec<bool> = Vec::new();
    if !r.list_wrappers.is_empty() {
        levels.extend(r.list_wrappers.iter().map(|w| w.nullable));
        levels.push(r.leaf_nullable.unwrap_or(true));
    } else if r.is_list {
        levels.push(r.nullable);
        levels.push(r.list_item_nullable.unwrap_or(true));
    } else {
        levels.push(r.nullable);
    }
    let leaf_nullable = levels.pop().unwrap_or(true);
    let mut schema = nullable(
        named_schema(ir, options, owner, field, &r.base)?,
        leaf_nullable,
    );
    for list_nullable in levels.into_iter().rev() {
        schema = json!({"type": "array", "items": schema});
        schema = nullable(schema, list_nullable);
    }
    Ok(schema)
}

fn nullable(schema: Value, is_nullable: bool) -> Value {
    let accepts_anything = schema.as_object().is_some_and(Map::is_empty);
    if accepts_anything {
        // A custom scalar admits any value; a non-null one admits any value but null.
        return if is_nullable {
            schema
        } else {
            json!({"not": {"type": "null"}})
        };
    }
    if !is_nullable {
        return schema;
    }
    // A union is already an `anyOf`; null joins its branches rather than nesting it.
    if let Value::Object(mut map) = schema {
        if map.len() == 1 {
            if let Some(Value::Array(branches)) = map.get_mut("anyOf") {
                branches.push(json!({"type": "null"}));
                return Value::Object(map);
            }
        }
        return json!({"anyOf": [Value::Object(map), {"type": "null"}]});
    }
    json!({"anyOf": [schema, {"type": "null"}]})
}

fn named_schema(
    ir: &WesleyIR,
    options: &Options,
    owner: &str,
    field: &str,
    name: &str,
) -> Result<Value, Error> {
    match name {
        "String" | "ID" => return Ok(json!({"type": "string"})),
        // GraphQL `Int` is a signed 32-bit integer; JSON Schema `integer` alone is unbounded.
        "Int" => return Ok(json!({"type": "integer", "minimum": i32::MIN, "maximum": i32::MAX})),
        "Float" => return Ok(json!({"type": "number"})),
        "Boolean" => return Ok(json!({"type": "boolean"})),
        _ => {}
    }
    let t = ir
        .types
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| Error::UnknownType {
            owner: owner.to_string(),
            field: field.to_string(),
            name: name.to_string(),
        })?;
    Ok(match t.kind {
        TypeKind::Scalar => json!({}),
        TypeKind::Enum => json!({"enum": t.enum_values}),
        TypeKind::Union => json!({
            "anyOf": t.union_members.iter().map(|m| json!({"$ref": type_id(options, m)})).collect::<Vec<_>>()
        }),
        TypeKind::Object | TypeKind::InputObject | TypeKind::Interface => {
            json!({"$ref": type_id(options, name)})
        }
    })
}
