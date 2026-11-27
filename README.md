# wesley-json-schema

A [Wesley](https://github.com/flyingrobots/wesley) target that emits JSON Schema 2020-12 documents, one per message type.

Wesley compiles GraphQL SDL into an intermediate representation (IR), and does nothing else: it attaches no meaning to any directive and runs no targets. Every output language is its own repository, a standalone program and library that reads the IR (`wesley schema lower --json`). This one is a **generic** target: it understands no directives and emits plain GraphQL, and it is the language tooling other targets (such as [wesley-parley](https://github.com/flyingrobots/wesley-parley)) render through. The decisions are recorded in flyingrobots/parley#116 and flyingrobots/wesley#882.

## Status

The MVP exists, in `crates/wesley-json-schema`:

- **`to_document_string`**, the canonical printing every JSON Schema from a Wesley target goes through: a document is built as a `serde_json::Value` and printed with two-space indentation, keys in insertion order, non-ASCII characters as themselves, and one trailing newline.
- **`emit_ir`** and the `wesley-json-schema` binary, a generic emitter for plain GraphQL: one document per object, interface and input type, in type-name order. Non-null fields are required; a nullable value also admits `null`; enums become `enum`; unions and named types become `$ref`s to sibling documents by `$id`; a custom scalar admits any value. Directives are ignored.

```sh
wesley schema lower --schema s.graphql --json | wesley-json-schema emit --id-base https://example.test/schemas/ --out schemas/
```

It reads the IR through the published `wesley-core` 0.3.0-alpha.4 (IR version `1.0.0`) until the IR is its own contract (flyingrobots/wesley#881). Every golden document is validated against the 2020-12 meta-schema, and the emitted documents validate real instances in the tests.

## Where it comes from

Issue #1 was transferred from flyingrobots/wesley#856. Its first consumer is Parley: flyingrobots/wesley-parley builds Parley's 17 message schemas from Parley's own `@parley_*` directives and prints them with `to_document_string` (flyingrobots/parley#119).

## How it generates code

The output is built as a syntax tree of the target language and printed by that language's printer. There is no string templating and no regex anywhere in the generator.

| Concern | Choice |
| --- | --- |
| Written in | Rust (stable) |
| Syntax tree | `serde_json::Value` (a JSON value is its own syntax tree) |
| Printer | canonical `serde_json` serialisation |
| Checked by | validation of every golden document against the JSON Schema 2020-12 meta-schema, and of example instances against the goldens |

Rules every change keeps:

- **No code as strings.** A generated identifier, literal or comment enters the output as a node of the tree, never by concatenating source text. Escaping is the printer's job.
- **Golden output is checked by the real toolchain.** Every golden file is compiled or type-checked in CI by the toolchain named above, not only compared byte for byte.
- **Deterministic to the byte.** The same IR and options give the same bytes on every machine. CI runs the generator twice and compares.
- **The IR is the only input.** The target reads the published IR (flyingrobots/wesley#881) and never depends on Wesley's internals.

## Releasing

The crate is published to crates.io from an annotated tag by the release workflow, after the owner approves it. [RELEASING.md](https://github.com/flyingrobots/wesley-json-schema/blob/main/RELEASING.md) gives the procedure.

## License

Apache 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
