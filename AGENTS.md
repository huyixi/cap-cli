# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` contains the entire CLI implementation and command routing.
- `Cargo.toml` and `Cargo.lock` define Rust dependencies and build settings.
- `~/.capmind/capmind.db` is the local SQLite database used by the CLI at runtime; treat it as a local artifact, not a source file.
- `target/` is Cargo build output (generated).

## Build, Test, and Development Commands
- `cargo build` compiles the `cap` binary into `target/`.
- `cargo run -- <args>` runs the CLI locally, e.g. `cargo run -- hello world` or `cargo run -- list`.
- `cargo test` runs the test suite (currently none), but is still the standard check before changes.

## Coding Style & Naming Conventions
- Use standard Rust style: 4-space indentation, `snake_case` for functions/variables, `CamelCase` for types.
- Keep CLI subcommands in the `Command` enum and argument parsing in `Cli`.
- Prefer concise, user-facing messages; keep errors actionable.
- Formatting follows `rustfmt` defaults; run `cargo fmt` when making stylistic changes.

## Testing Guidelines
- No automated tests are present yet; add unit tests alongside new logic when possible.
- If you introduce tests, use Rust’s built-in `#[test]` framework and clear names like `adds_memo_with_timestamp`.
- Run `cargo test` before submitting changes.

## Commit & Pull Request Guidelines
- Commit messages follow a lightweight conventional style: `feat: ...`, `fix: ...`, `chore: ...`.
- Keep commits focused on one logical change.
- PRs should include a short summary, steps to verify (commands run), and example usage if CLI behavior changes.

## Security & Data Memos
- The app writes memos to `~/.capmind/capmind.db`. Avoid committing personal data; do not add this file to source control.
