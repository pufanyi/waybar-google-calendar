# AGENTS.md

Guidance for agents working in this repository.

## Document Boundaries

- `README.md` is for humans: users, packagers, and contributors. Keep it focused
  on features, installation/build steps, authentication, Waybar setup, theming,
  environment variables, packaging, and a concise source layout.
- `AGENTS.md` is for agents and maintainers working in the repository. Keep it
  focused on workflow rules, code organization guidance, verification commands,
  and documentation maintenance expectations.
- Do not put agent workflow rules in `README.md`.
- Do not duplicate full user-facing instructions in `AGENTS.md`; point agents to
  update `README.md` when those instructions change.

## Project Overview

- This is a Rust 2024 GTK4/libadwaita app using Relm4.
- The binary is `waybar-gcal`, with source entrypoint at `src/main.rs`.
- Main UI modes:
  - `agenda`: interactive Google Calendar agenda popup.
  - `month`: local month calendar popup.
  - `auth` / `auth-ui`: Google OAuth authentication flows.
- Shared project modules are grouped by responsibility:
  - `src/app/`: CLI parsing and single-instance process handling.
  - `src/auth_ui/`: standalone graphical authentication helper.
  - `src/calendar/`: calendar/domain model types and date helpers.
  - `src/storage/`: filesystem paths and agenda cache.
  - `src/google/`: Google Calendar API, OAuth, transport helpers, and response types.
  - `src/month/`: standalone local month calendar popup.
  - `src/ui/`: shared GTK widget helpers and theme loading.
- `src/agenda.rs` defines the agenda component entrypoint, state, messages, and
  Relm4 wiring.
  Agenda rendering helpers live under `src/agenda/`:
  - `controller.rs`: agenda input and command handlers, event loading, month navigation.
  - `auth_prompt.rs`: embedded OAuth/setup wizard composition.
  - `auth_prompt/`: wizard pages, credential form, auth status, and auth helper widgets.
  - `view.rs`: agenda popup composition.
  - `view/`: calendar pane, agenda list, event/message cards, and status text.
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
- Prefer existing patterns in `src/agenda.rs`, `src/agenda/`, `src/month/`,
  `src/auth_ui/`, and shared helpers in `src/ui/`.
- For UI changes, update `assets/themes/apple-light.css` only when new or changed CSS classes need theme support.
- Keep user-facing OAuth/setup text accurate and concrete; the README documents the current authentication flow.

## Documentation Maintenance

After every repository modification, review both `AGENTS.md` and `README.md`
before finishing.

Update `AGENTS.md` when a change affects:

- Build, test, formatting, packaging, or release commands.
- Project structure, entrypoints, modes, or major module responsibilities.
- Required environment variables, credentials, cache locations, or runtime dependencies.
- UI/theming conventions or important CSS class names.
- Agent workflow expectations for this repository.

Update `README.md` when a change affects:

- User-visible commands, modes, flags, examples, or Waybar setup.
- Authentication flow, credential/token locations, or Google Cloud setup steps.
- Runtime/build dependencies, installation, packaging, or release instructions.
- Environment variables, cache behavior, timeouts, or config paths.
- Theme behavior, important CSS classes, or custom theme instructions.
- High-level source layout that a human contributor would reasonably need.

Do not update `AGENTS.md` for incidental implementation details that are already clear from nearby code.
Do not update `README.md` for private refactors that do not change user-facing behavior or contributor-facing layout.
