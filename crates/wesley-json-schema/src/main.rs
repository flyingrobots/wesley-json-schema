//! `wesley-json-schema`: reads Wesley IR and writes one JSON Schema document per type.
//!
//! ```text
//! wesley schema lower --schema s.graphql --json | wesley-json-schema emit --id-base URL --out DIR
//! ```

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use wesley_json_schema::{emit_ir, to_document_string, Options};

const USAGE: &str = "usage: wesley-json-schema emit --id-base <url> --out <dir> [--ir <file>]\n\nReads Wesley IR JSON (from --ir, or stdin) and writes <Type>.schema.json for every object,\ninterface and input type. Directives are ignored.";

struct Args {
    id_base: String,
    out: PathBuf,
    ir: Option<PathBuf>,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
    match args.next().as_deref() {
        Some("emit") => {}
        Some("--help" | "-h") | None => return Err(String::new()),
        Some(other) => return Err(format!("unknown command {other}")),
    }
    let (mut id_base, mut out, mut ir) = (None, None, None);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--id-base" => id_base = Some(value()?),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--ir" => ir = Some(PathBuf::from(value()?)),
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(Args {
        id_base: id_base.ok_or("--id-base is required")?,
        out: out.ok_or("--out is required")?,
        ir,
    })
}

fn run(args: &Args) -> Result<(), String> {
    let text = match &args.ir {
        Some(path) => {
            std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?
        }
        None => {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .map_err(|e| format!("stdin: {e}"))?;
            s
        }
    };
    let ir = serde_json::from_str(&text).map_err(|e| format!("IR: {e}"))?;
    let documents = emit_ir(
        &ir,
        &Options {
            id_base: args.id_base.clone(),
        },
    )
    .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&args.out).map_err(|e| format!("{}: {e}", args.out.display()))?;
    for d in documents {
        let path = args.out.join(&d.file_name);
        std::fs::write(&path, to_document_string(&d.value))
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) if message.is_empty() => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("wesley-json-schema: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("wesley-json-schema: {message}");
            ExitCode::FAILURE
        }
    }
}
