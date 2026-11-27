# Releasing

`wesley-json-schema` (the library and the `wesley-json-schema` binary) is published to [crates.io](https://crates.io/crates/wesley-json-schema) by the [release workflow](.github/workflows/release.yml), never from a laptop. A release is an annotated tag on `main`; publishing waits for the owner's approval.

## Prepare

Open a pull request that sets `version` in [crates/wesley-json-schema/Cargo.toml](crates/wesley-json-schema/Cargo.toml), updates `Cargo.lock`, and turns `## Unreleased` in [CHANGELOG.md](CHANGELOG.md) into `## [<version>] - <date>` with an empty `## Unreleased` above it. CI runs `cargo publish --dry-run --locked` on every pull request, so a package that cannot be built from its `.crate` fails there. Merge the pull request with a merge commit.

## Tag

Create an annotated tag on the merged `main` and push it. The workflow refuses a lightweight tag.

```sh
git switch main
git pull --ff-only
git tag -a v0.1.0-alpha.1 -m "wesley-json-schema 0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

Pushing the tag starts the `release` workflow. To run it again for a tag that already exists, dispatch it from the tag: `gh workflow run release.yml --ref v0.1.0-alpha.1`. A dispatch from any ref that is not a tag is refused.

## What the workflow does

1. **gate** refuses the release unless the tag is annotated, its commit is on `origin/main`, the tag without its `v` equals the crate's `version` as `cargo metadata` reports it, and `CHANGELOG.md` has a `## [<version>]` heading. It then runs `cargo fmt --check`, `cargo clippy -D warnings`, the full test suite and `cargo publish --dry-run`, all with `--locked`.
2. **publish** runs in the `crates-io` environment, which waits for the owner's approval: open the run under Actions, choose **Review deployments**, select `crates-io` and approve. It then runs `cargo publish --locked`. If that version is already on crates.io, for example on a re-run, it skips the upload instead of failing.
3. **verify** waits for the version to appear on crates.io (up to 20 checks, 15 seconds apart), installs it with `cargo install wesley-json-schema --version =<version> --locked`, and runs `wesley-json-schema --help`. A release that does not install from crates.io fails here.
4. **github-release** creates the GitHub release for the tag, with that version's changelog section as its notes. A version with a `-` suffix, such as `0.1.0-alpha.1`, is marked as a prerelease.

## Yank

A version on crates.io cannot be deleted or replaced. If a published version is broken, yank it so that new lockfiles stop selecting it, then release a fixed version with a new number.

```sh
cargo yank --version 0.1.0-alpha.1 wesley-json-schema
```

`cargo yank --version 0.1.0-alpha.1 --undo wesley-json-schema` reverses a yank. Record the yank and the reason in `CHANGELOG.md`, and edit the GitHub release to say the version is yanked.
