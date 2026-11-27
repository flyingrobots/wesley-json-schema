//! The published crate carries the README, the license and the NOTICE that Apache-2.0 requires
//! every redistribution to keep. `LICENSE` and `NOTICE` in the crate are links to the
//! repository's own files, which `cargo package` follows.

use std::process::Command;

#[test]
fn the_package_lists_readme_license_and_notice() {
    let output = Command::new(env!("CARGO"))
        .args(["package", "--list", "--allow-dirty", "--locked"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo package --list runs");
    assert!(
        output.status.success(),
        "cargo package --list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let listing = String::from_utf8(output.stdout).expect("utf-8 listing");
    let files: Vec<&str> = listing.lines().collect();
    for name in ["README.md", "LICENSE", "NOTICE"] {
        assert!(
            files.contains(&name),
            "{name} is missing from the package: {files:?}"
        );
    }
}

/// crates.io resolves a README's relative links against the crate's directory in the
/// repository, so each one must name a file there; a link to anything else is absolute.
#[test]
fn every_relative_link_in_the_readme_names_a_file_in_the_crate() {
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(crate_dir)
        .output()
        .expect("cargo metadata runs");
    assert!(output.status.success(), "cargo metadata failed");
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).expect("metadata");
    let readme = metadata["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .find(|p| p["name"] == "wesley-json-schema")
        .and_then(|p| p["readme"].as_str())
        .expect("the manifest names a readme");
    let text = std::fs::read_to_string(crate_dir.join(readme)).expect("readme");
    let mut broken = Vec::new();
    for (_, rest) in text.match_indices("](").map(|(i, _)| text.split_at(i + 2)) {
        let Some(end) = rest.find(')') else { continue };
        let target = &rest[..end];
        let path = target.split('#').next().unwrap_or_default();
        if path.is_empty() || target.contains("://") || target.starts_with("mailto:") {
            continue;
        }
        if !crate_dir.join(path).is_file() {
            broken.push(target.to_owned());
        }
    }
    assert!(
        broken.is_empty(),
        "README links that crates.io cannot resolve: {broken:?}"
    );
}
