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
