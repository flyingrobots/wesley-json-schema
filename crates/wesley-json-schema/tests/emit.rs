//! The plain-GraphQL emitter and the canonical printer, held to golden files and to the
//! JSON Schema 2020-12 meta-schema.

use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;
use serde_json::{json, Value};
use wesley_json_schema::{emit_ir, to_document_string, Error, Options, DIALECT};

const ID_BASE: &str = "https://example.test/schemas/";

fn fixture_ir() -> wesley_core::WesleyIR {
    let sdl = include_str!("fixtures/plain.graphql");
    wesley_core::lower_schema_sdl(sdl).expect("fixture lowers")
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn options() -> Options {
    Options {
        id_base: ID_BASE.to_string(),
    }
}

#[test]
fn emits_one_document_per_object_and_input_type_in_name_order() {
    let documents = emit_ir(&fixture_ir(), &options()).expect("emits");
    let names: Vec<&str> = documents.iter().map(|d| d.file_name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Page.schema.json",
            "User.schema.json",
            "UserFilter.schema.json"
        ]
    );
}

#[test]
fn every_document_matches_its_golden_file() {
    let update = std::env::var_os("WESLEY_UPDATE_GOLDEN").is_some();
    for document in emit_ir(&fixture_ir(), &options()).expect("emits") {
        let path = golden_dir().join(&document.file_name);
        let printed = to_document_string(&document.value);
        if update {
            std::fs::write(&path, &printed).expect("golden written");
        }
        let golden =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(printed, golden, "{}", document.file_name);
    }
}

#[test]
fn every_document_is_a_valid_2020_12_schema() {
    for document in emit_ir(&fixture_ir(), &options()).expect("emits") {
        assert_eq!(document.value["$schema"], json!(DIALECT));
        let result = jsonschema::meta::validate(&document.value);
        assert!(result.is_ok(), "{}: {result:?}", document.file_name);
    }
}

#[test]
fn a_user_document_accepts_a_real_user_and_rejects_a_wrong_one() {
    let documents = emit_ir(&fixture_ir(), &options()).expect("emits");
    let mut by_id = std::collections::HashMap::new();
    for d in &documents {
        by_id.insert(
            d.value["$id"].as_str().expect("id").to_string(),
            d.value.clone(),
        );
    }
    let user = documents
        .iter()
        .find(|d| d.file_name == "User.schema.json")
        .expect("user");
    let validator = jsonschema::options()
        .with_retriever(Documents(by_id))
        .build(&user.value)
        .expect("compiles");

    let good = json!({
        "id": "u1", "name": null, "age": 30, "active": true, "role": "ADMIN",
        "tags": ["a"], "friends": [null], "meta": {"anything": [1, "x"]},
        "best": {"url": "/home"}
    });
    assert!(validator.is_valid(&good));

    let missing_required = json!({"id": "u1", "age": 30, "active": true, "role": "ADMIN"});
    assert!(!validator.is_valid(&missing_required));

    let bad_enum = json!({"id": "u1", "age": 30, "active": true, "role": "OWNER", "tags": []});
    assert!(!validator.is_valid(&bad_enum));

    let null_in_non_null_list =
        json!({"id": "u1", "age": 1, "active": true, "role": "ADMIN", "tags": [null]});
    assert!(!validator.is_valid(&null_in_non_null_list));
}

#[test]
fn directives_are_ignored_entirely() {
    let documents = emit_ir(&fixture_ir(), &options()).expect("emits");
    let page = documents
        .iter()
        .find(|d| d.file_name == "Page.schema.json")
        .expect("page");
    assert_eq!(page.value["properties"]["url"], json!({"type": "string"}));
}

#[test]
fn printing_is_deterministic_and_ends_in_one_newline() {
    let documents = emit_ir(&fixture_ir(), &options()).expect("emits");
    for d in &documents {
        let a = to_document_string(&d.value);
        let b = to_document_string(&d.value);
        assert_eq!(a, b);
        assert!(a.ends_with("}\n") && !a.ends_with("\n\n"));
    }
}

#[test]
fn printing_keeps_insertion_order_and_writes_non_ascii_raw() {
    let mut v = serde_json::Map::new();
    v.insert("z".to_string(), json!("§3.1"));
    v.insert("a".to_string(), json!(1));
    let printed = to_document_string(&Value::Object(v));
    assert_eq!(printed, "{\n  \"z\": \"§3.1\",\n  \"a\": 1\n}\n");
}

#[test]
fn an_unknown_ir_version_is_refused() {
    let mut ir = fixture_ir();
    ir.version = "9.0.0".to_string();
    match emit_ir(&ir, &options()) {
        Err(Error::IrVersion { found }) => assert_eq!(found, "9.0.0"),
        other => panic!("expected IrVersion error, got {other:?}"),
    }
}

#[test]
fn a_field_naming_an_undefined_type_is_refused() {
    let ir = wesley_core::lower_schema_sdl("type A { b: Missing! }").expect("lowers");
    match emit_ir(&ir, &options()) {
        Err(Error::UnknownType { owner, field, name }) => {
            assert_eq!(
                (owner.as_str(), field.as_str(), name.as_str()),
                ("A", "b", "Missing")
            );
        }
        other => panic!("expected UnknownType, got {other:?}"),
    }
}

/// Resolves `$ref`s to the emitted sibling documents, by `$id`.
struct Documents(std::collections::HashMap<String, Value>);

impl jsonschema::Retrieve for Documents {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| format!("no document {}", uri.as_str()).into())
    }
}
