# Repository Guidelines

## Project Structure & Module Organization
- Core code lives in `src/` and is split by responsibility:
- `src/main.rs`: entrypoint, CLI flow, env validation, and output branching.
- `src/cli.rs`: argument definitions, help template, and help JSON builder.
- `src/client.rs`: Cloudflare Markdown API client and response parsing.
- `src/output.rs`: filename derivation and file writing.
- `src/error.rs`: shared `AppError` definitions.
- Unit/integration-style tests are colocated with each module via `#[cfg(test)]` (there is no top-level `tests/` directory yet).
- Build artifacts are generated under `target/` and should not be edited manually.

## Build, Test, and Development Commands
- `cargo run -- <URL>`: run the CLI against a URL.
- `cargo run -- --help` / `cargo run -- --help-json`: inspect human/machine-readable help.
- `cargo test --all-targets`: run all tests (including async tests).
- `cargo fmt --all -- --check`: formatting check.
- `cargo clippy --all-targets -- -D warnings`: lint with warnings treated as errors.
- `nix develop`: enter the pinned Rust dev shell.
- `nix build`: build the package with Nix.
- `nix flake check`: run formatter, clippy, and test checks defined in `flake.nix`.

## Coding Style & Naming Conventions
- Rust edition is 2021; use standard rustfmt style (4-space indentation).
- Naming: `snake_case` for functions/modules, `UpperCamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Prefer explicit error propagation with `Result<_, AppError>` over panics in production code.
- Keep modules focused and avoid cross-module leakage of unrelated logic.

## Testing Guidelines
- Use `#[test]` for sync logic and `#[tokio::test]` for async API paths.
- Use `wiremock` to simulate Cloudflare API responses and `tempfile` for filesystem behavior.
- Name tests by behavior, e.g. `fetch_markdown_returns_api_error`.
- Before opening a PR, run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all-targets`.

## Commit & Pull Request Guidelines
- This repository currently has no established commit history; start with Conventional Commits (e.g. `feat: ...`, `fix: ...`, `docs: ...`).
- Keep commits atomic (one logical change per commit).
- PRs should include: objective, key changes, verification commands/results, and linked issue (if available).
- If CLI behavior changes, include a short example command/output in the PR description.

## Security & Configuration Tips
- Required env vars: `CF_ACCOUNT_ID` and `CF_API_TOKEN` (see `.env.example`).
- Never commit credentials; keep real values in local `.env` only.
- When touching API request logic, confirm `rejectRequestPattern` behavior remains intentional.
