# Changelog

All notable changes to this project are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.1.0-alpha.1] - 2026-11-26

The first release on crates.io.

### Added

- Release tooling: [RELEASING.md](RELEASING.md), and a `release` workflow that publishes from an annotated `v*` tag on `main` after a gate (the tag is annotated and on `main`, equals the crate version and has a changelog entry; fmt, clippy, the tests and `cargo publish --dry-run` pass) and the owner's approval of the `crates-io` environment, then installs the published crate and creates the GitHub release. The crate carries `LICENSE` and `NOTICE`, CI runs `cargo publish --dry-run` on every pull request, and every action is pinned by commit.
- `to_document_string`, canonical printing for JSON Schema documents, and `emit_ir` with the `wesley-json-schema emit` binary: one JSON Schema 2020-12 document per object, interface and input type of a plain GraphQL schema, directives ignored; a non-null custom scalar admits any value but `null`, and `Int` is bounded to the signed 32-bit range; an input object rejects undeclared fields, and an input field with a default is not required; following GraphQL input coercion, an input `ID` also accepts an integer and an input list also accepts a single value that is neither null nor an array, at every level. CI and git hooks run `cargo fmt --check`, `clippy -D warnings` and the tests.
- The repository, its license, and the rules for how the target generates code (flyingrobots/parley#116, flyingrobots/wesley#882).
