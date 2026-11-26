//! The `wesley-json-schema` binary reads IR from stdin and writes the same documents the library emits.

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn emit_writes_the_golden_documents_from_ir_on_stdin() {
    let ir = wesley_core::lower_schema_sdl(include_str!("fixtures/plain.graphql")).expect("lowers");
    let out = std::env::temp_dir().join(format!("wesley-json-schema-cli-{}", std::process::id()));
    let mut child = Command::new(env!("CARGO_BIN_EXE_wesley-json-schema"))
        .args([
            "emit",
            "--id-base",
            "https://example.test/schemas/",
            "--out",
        ])
        .arg(&out)
        .stdin(Stdio::piped())
        .spawn()
        .expect("runs");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(serde_json::to_string(&ir).expect("ir").as_bytes())
        .expect("writes");
    assert!(child.wait().expect("exits").success());
    for name in [
        "Page.schema.json",
        "User.schema.json",
        "UserFilter.schema.json",
    ] {
        let written = std::fs::read_to_string(out.join(name)).expect("written");
        let golden = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/golden")
                .join(name),
        )
        .expect("golden");
        assert_eq!(written, golden, "{name}");
    }
    std::fs::remove_dir_all(&out).expect("cleaned");
}

#[test]
fn a_missing_option_is_a_usage_error() {
    let status = Command::new(env!("CARGO_BIN_EXE_wesley-json-schema"))
        .args(["emit", "--out", "x"])
        .stderr(Stdio::null())
        .status()
        .expect("runs");
    assert_eq!(status.code(), Some(2));
}
