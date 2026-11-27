//! The published crate carries the README, the license and the NOTICE that Apache-2.0 requires
//! every redistribution to keep, and its copies are the repository's own, byte for byte.

use std::path::{Path, PathBuf};
use std::process::Command;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_root() -> PathBuf {
    crate_dir()
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn the_crate_copies_of_license_and_notice_are_the_repository_files() {
    for name in ["LICENSE", "NOTICE"] {
        assert_eq!(
            read(&crate_dir().join(name)),
            read(&repository_root().join(name)),
            "crates/wesley-json-schema/{name} differs from the repository's {name}"
        );
    }
}

#[test]
fn the_package_lists_readme_license_and_notice() {
    let output = Command::new(env!("CARGO"))
        .args([
            "package",
            "--list",
            "--allow-dirty",
            "--locked",
            "-p",
            "wesley-json-schema",
        ])
        .current_dir(repository_root())
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
