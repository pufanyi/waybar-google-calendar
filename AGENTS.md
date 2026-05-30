# AGENTS.md

Guidance for agents working in this repository.

## Project Overview

- This is a Rust 2024 GTK4/libadwaita app using Relm4.
- The binary is `waybar-gcal`, with source entrypoint at `src/main.rs`.
- Main UI modes:
  - `agenda`: interactive Google Calendar agenda popup.
  - `month`: local month calendar popup.
  - `auth` / `auth-ui`: Google OAuth authentication flows.
- Google Calendar access is implemented with `yup-oauth2` and `reqwest`.
- The built-in GTK CSS theme is `assets/themes/apple-light.css`.

## Common Commands

- Format check: `cargo fmt --check`
- Tests: `cargo test`
- Build: `cargo build`
- Release build: `cargo build --release`

Run `cargo fmt` before finalizing changes when formatting is needed.

## Editing Notes

- Keep changes scoped to the requested behavior.
- Do not remove or overwrite user changes in a dirty worktree.
- Prefer existing patterns in `src/agenda.rs`, `src/month.rs`, `src/auth_ui.rs`, and shared helpers in `src/ui.rs`.
- For UI changes, update `assets/themes/apple-light.css` only when new or changed CSS classes need theme support.
- Keep user-facing OAuth/setup text accurate and concrete; the README documents the current authentication flow.

## AGENTS.md Maintenance

After every repository modification, review this file before finishing.

Update `AGENTS.md` when a change affects:

- Build, test, formatting, packaging, or release commands.
- Project structure, entrypoints, modes, or major module responsibilities.
- Required environment variables, credentials, cache locations, or runtime dependencies.
- UI/theming conventions or important CSS class names.
- Agent workflow expectations for this repository.

Do not update `AGENTS.md` for incidental implementation details that are already clear from nearby code.
