# Claude Guidelines for this repository

## Release & versioning

Every PR merged into `main` automatically bumps the version in `Cargo.toml`,
commits the change, and pushes a tag — which triggers the release workflow.

**You must add exactly one of these labels to every PR you create or review:**

| Label   | When to use                                              | Example                        |
|---------|----------------------------------------------------------|--------------------------------|
| `major` | Breaking change (removes/changes public API or behavior) | Rename a CLI flag, drop a command |
| `minor` | Backwards-compatible new feature                         | New subcommand, new flag       |
| `patch` | Bug fix, refactor, docs, tests, chores                   | Fix a crash, update deps       |

If no label is present the workflow defaults to **patch**.

## Workflow summary

1. PR passes CI (`cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo doc`).
2. PR is merged into `main`.
3. `version-bump` workflow fires, reads the label, bumps `Cargo.toml` / `Cargo.lock`,
   commits `chore(release): bump version to X.Y.Z`, and pushes tag `vX.Y.Z`.
4. `release` workflow fires on the new tag and publishes the Windows binary.

## Commit style

Follow conventional commits: `feat:`, `fix:`, `chore:`, `refactor:`, `test:`, `docs:`.
Scope is optional but encouraged, e.g. `feat(minecraft):`.
