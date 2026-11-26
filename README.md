# wesley-json-schema

A [Wesley](https://github.com/flyingrobots/wesley) target that emits JSON Schema 2020-12 documents, one per message type.

Wesley compiles GraphQL SDL into an intermediate representation (IR), and does nothing else: it attaches no meaning to any directive and runs no targets. Every output language is its own repository, a standalone program and library that reads the IR (`wesley schema lower --json`). This one is a **generic** target: it understands no directives and emits plain GraphQL, and it is the language tooling other targets (such as [wesley-parley](https://github.com/flyingrobots/wesley-parley)) render through. The decisions are recorded in flyingrobots/parley#116 and flyingrobots/wesley#882.

## Status

Not started. This repository holds the plan and its rules; the first code waits on the versioned IR contract (flyingrobots/wesley#881).

## Where it comes from

The plan is issue #1 (transferred from flyingrobots/wesley#856). Its first consumer is Parley, whose 17 hand-written message schemas become generated output once this target can reproduce them as JSON values (flyingrobots/parley#119).

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

## License

Apache 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
