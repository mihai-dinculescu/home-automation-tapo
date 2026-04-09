# Pre-Commit

Run all checks, fix any issues found, then present a summary table.

## Checks

### Rust checks

Run independent checks (`cargo check`, `cargo clippy`, `cargo fmt`, `cargo test`) in parallel.

- `cargo check`
- `cargo clippy`
- `cargo fmt --all`
- `cargo test`
- No `unwrap()` in non-test code without a `// safe:` comment
- No `unsafe` in non-test code without a `// SAFETY:` comment
- No unnecessary clones
- No deeply nested `use` (max one level of `{}` nesting)

## Code Review

After fixing all issues found in the checks, review the code changes for correctness, readability, and maintainability and propose improvements.
Summarize the findings according to severity.
