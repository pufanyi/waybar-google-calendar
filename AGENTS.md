# AGENTS.md

Guidance for agents working in this repository.

## Document Boundaries

- `README.md` is for humans: users, packagers, and contributors. Keep it focused
  on features, installation/build steps, authentication, Waybar setup, theming,
  environment variables, packaging, and a concise source layout.
- `docs/` is for longer human-facing guides that are too detailed for the
  README, such as provider setup walkthroughs.
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
  - `src/calendar/`: calendar/domain model types, date helpers, and shared
    calendar navigation view state.
  - `src/storage/`: filesystem paths, agenda cache, and persistent user settings.
  - `src/google/`: Google Calendar API, OAuth, transport helpers, and response types.
  - `src/i18n/`: localized UI text and calendar labels.
  - `src/month/`: standalone local month calendar popup.
  - `src/ui/`: shared GTK widget helpers and theme loading.
- `src/agenda.rs` defines the agenda component entrypoint, state, messages, and
  Relm4 wiring.
  Agenda rendering helpers live under `src/agenda/`:
  - `controller.rs`: agenda input and command handlers, event loading, month navigation.
  - `settings/`: in-window settings panel composition, section modules, form
    wiring, localization, and account status controls.
  - `auth_prompt.rs`: embedded OAuth/setup prompt composition.
  - `auth_prompt/`: setup page, credential form, auth status, and auth helper widgets.
  - `view.rs`: agenda popup composition.
  - `view/`: calendar pane, agenda list, event editor/detail panel,
    event/message cards, and status text.
  - `view/list/`: agenda timeline grouping, current-time marker, and event rows.
- Google Calendar access is implemented with `yup-oauth2` and `reqwest`;
  agenda uses Google Calendar read and event management scopes.
- The built-in GTK CSS theme is `assets/themes/apple-light.css`.
- The agenda settings panel uses settings-specific CSS classes such as
  `.settings-panel`, `.settings-section`, `.settings-icon-tile`,
  `.settings-icon-glyph`, `.settings-form-row`, and `.settings-footer`; keep
  theme support in sync when changing that panel.
- The agenda dashboard uses `.agenda-pane`, `.agenda-context-bar`,
  `.agenda-view-tabs`, `.agenda-view-tab`, `.agenda-list-header`,
  `.agenda-list`, `.agenda-day-section`, `.agenda-timeline-row`, and
  `.agenda-now-marker`; event editing uses `.event-editor-panel`,
  `.event-detail-row`, `.event-form-row`, and `.event-editor-actions`; keep
  theme support in sync when changing those views.
- The Google OAuth setup prompt links to
  `https://github.com/pufanyi/waybar-google-calendar/blob/main/docs/google-oauth.md`;
  keep the linked doc accurate when setup behavior changes.
- The AUR publishing workflow is `.github/workflows/publish-aur.yml`; it runs
  binary release asset builds, package metadata generation, and AUR git pushes
  as the non-root `builder` user. Package tags use `v<pkgver>-<pkgrel>`, for
  example `v0.0.1-1` and `v0.0.1-2`; packaging-only fixes increment only the
  `pkgrel` part. The workflow publishes both `packaging/aur` and
  `packaging/aur-bin`.

## AUR Publishing Notes

- `packaging/aur/` is the source-build AUR package for
  `waybar-google-calendar`.
- `packaging/aur-bin/` is the prebuilt binary AUR package for
  `waybar-google-calendar-bin`.
- Both packages are published by `.github/workflows/publish-aur.yml` from the
  same package tag format: `v<pkgver>-<pkgrel>`.
- The source package downloads the GitHub tag tarball for the package tag, for
  example `v0.0.1-4`.
- The binary package downloads the GitHub Release asset built by the workflow,
  for example `waybar-google-calendar-bin-0.0.1-4-x86_64.tar.zst`.
- When changing AUR metadata, keep both package directories in sync where
  applicable and update their README files if the release flow changes.
- Do not move or rewrite published release tags. If a packaging-only fix is
  needed for the same application version, increment `pkgrel` and publish a new
  tag.
- `waybar-google-calendar-bin` should continue to provide and conflict with
  `waybar-google-calendar`.

## Common Commands

- Format check: `cargo fmt --check`
- Tests: `cargo test`
- Build: `cargo build`
- Release build: `cargo build --release`
- GitHub Actions lint: `actionlint`
- Source AUR metadata: `cd packaging/aur && makepkg --printsrcinfo`
- Binary AUR metadata: `cd packaging/aur-bin && makepkg --printsrcinfo`

Run `cargo fmt` before finalizing changes when formatting is needed.

## Editing Notes

- Keep changes scoped to the requested behavior.
- Do not remove or overwrite user changes in a dirty worktree.
- Prefer existing patterns in `src/agenda.rs`, `src/agenda/`, `src/month/`,
  `src/auth_ui/`, and shared helpers in `src/ui/`.
- Prefer Relm4 for UI architecture: model state, messages, component lifecycle,
  commands, and update flow should live in `Component`/`SimpleComponent`
  patterns where practical. Use direct gtk4-rs/libadwaita APIs mainly for
  widget construction, styling, and low-level signal forwarding into Relm4
  messages.
- Avoid deprecated GTK/libadwaita APIs in new code when a maintained
  replacement exists. If a deprecated API must be kept temporarily, keep the
  allowance as narrow as possible and document why it remains.
- For UI changes, update `assets/themes/apple-light.css` only when new or changed CSS classes need theme support.
- Keep user-facing OAuth/setup text accurate and concrete; the README documents the current authentication flow.
- Prefer in-window state changes, panels, and view transitions over adding new
  popup windows or modal dialogs. Use a separate window only when it clearly
  improves the workflow or matches an existing app mode.
- When a GTK window or dialog is meant to be reused, hide it with
  `set_visible(false)` and present it again instead of closing/destroying it.

## Documentation Maintenance

After every repository modification, review both `AGENTS.md` and `README.md`
before finishing.

Update `AGENTS.md` when a change affects:

- Build, test, formatting, packaging, or release commands.
- GitHub Actions workflows, especially release or AUR publishing behavior.
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

Update files in `docs/` when a user-facing workflow needs more detail than
belongs in the README. If the app links to a doc, keep the app text and the doc
in sync.

Do not update `AGENTS.md` for incidental implementation details that are already clear from nearby code.
Do not update `README.md` for private refactors that do not change user-facing behavior or contributor-facing layout.
