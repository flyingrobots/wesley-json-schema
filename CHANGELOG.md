# Changelog

All notable changes to this project are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `to_document_string`, canonical printing for JSON Schema documents, and `emit_ir` with the `wesley-json-schema emit` binary: one JSON Schema 2020-12 document per object, interface and input type of a plain GraphQL schema, directives ignored; a non-null custom scalar admits any value but `null`, and `Int` is bounded to the signed 32-bit range. CI and git hooks run `cargo fmt --check`, `clippy -D warnings` and the tests.
- The repository, its license, and the rules for how the target generates code (flyingrobots/parley#116, flyingrobots/wesley#882).
