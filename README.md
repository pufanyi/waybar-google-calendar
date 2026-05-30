# waybar-google-calendar

GTK4/Relm4 Google Calendar popup for Waybar.

This project provides a small native Linux desktop popup intended for Waybar's
clock module:

- `waybar-gcal agenda` shows Google Calendar events for the visible calendar grid.
- `waybar-gcal agenda --calendar Work --timezone Asia/Singapore` filters the agenda.
- `waybar-gcal month` shows a local month calendar.
- `waybar-gcal auth` starts Google Calendar OAuth authentication.
- `waybar-gcal auth-ui` opens the graphical authentication helper.
- `waybar-gcal print-theme` prints the built-in CSS theme.

The UI is built with Relm4 on top of GTK4/libadwaita. Google Calendar access is
implemented natively with `yup-oauth2` and `reqwest`; no external Google CLI is
required at runtime.

## Dependencies

Runtime:

- `gtk4`
- `libadwaita`

Build:

- `rust`
- `cargo`

On Arch Linux:

```bash
sudo pacman -S gtk4 libadwaita rust
```

## Build

```bash
cargo build --release
```

## Development

Common checks:

```bash
cargo fmt --check
cargo test
```

The source tree is grouped by responsibility:

- `src/app/`: CLI parsing and single-instance process handling.
- `src/agenda/`: Google Calendar agenda popup internals.
- `src/auth_ui/`: standalone graphical authentication helper.
- `src/calendar/`: shared calendar/date model and date helpers.
- `src/google/`: Google OAuth and Calendar API access.
- `src/month/`: standalone local month calendar popup.
- `src/storage/`: cache and filesystem paths.
- `src/ui/`: shared GTK helpers and theme loading.

## Authentication

Create an OAuth desktop client in Google Cloud with the Calendar API enabled.
Follow the full setup guide in [`docs/google-oauth.md`](docs/google-oauth.md)
to get the Client ID and Client Secret.

On first launch, the agenda popup can open the same setup guide from the
embedded setup panel. Paste the Client ID and Client Secret into the app, then
press `Save & Authenticate`.

When OAuth setup is incomplete, the agenda popup focuses the setup panel instead
of showing stale cached events.

If you prefer using the downloaded JSON, place it at:

```text
~/.config/waybar-google-calendar/client_secret.json
```

Or point to it explicitly:

```bash
export WAYBAR_GCAL_CLIENT_SECRET=/path/to/client_secret.json
```

Then authenticate:

```bash
waybar-gcal auth
```

The generated token is stored at
`~/.local/share/waybar-google-calendar/oauth-token.json` by default. This token
is created by Google OAuth after browser login; it is not something users need
to paste manually.

The separate graphical helper remains available:

```bash
waybar-gcal auth-ui
```

## Waybar

Use `examples/waybar-clock.json` as a starting point:

```json
{
  "clock": {
    "on-click": "waybar-gcal agenda",
    "on-click-right": "waybar-gcal month"
  }
}
```

The agenda popup includes an interactive month pane. Use the arrow buttons to
move between months, click a day to filter the agenda list, or use `All` and
`Today` for quick selection. Events are fetched dynamically for the visible
calendar grid, so changing months refreshes the Google Calendar range for that
month. The standalone month popup supports month and year navigation plus day
selection.

Agenda can also filter to a calendar name or ID:

```json
{
  "clock": {
    "on-click": "waybar-gcal agenda --calendar Work --timezone Asia/Singapore"
  }
}
```

## Implementation Notes

The project currently avoids a dedicated calendar UI library. The maintained
pieces are split by responsibility:

- GTK/Relm4 owns the windowing and component lifecycle.
- `chrono` owns local date arithmetic and formatting.
- `yup-oauth2` owns Google OAuth and token refresh.
- `reqwest` owns Google Calendar HTTP requests.

Google Calendar recurrence expansion is handled by querying `events.list` with
`singleEvents=true`. If this project adds offline `.ics` support later, good
candidate crates are `icalendar` for iCalendar parsing/building and `rrule` for
RFC recurrence expansion.

## Themes

The app ships with a built-in `apple-light` GTK CSS theme. User CSS is appended
after the built-in theme, so custom files can override only the selectors they
care about.

Default user theme path:

```text
~/.config/waybar-google-calendar/style.css
```

Create a full editable copy:

```bash
mkdir -p ~/.config/waybar-google-calendar
waybar-gcal print-theme > ~/.config/waybar-google-calendar/style.css
```

Or run with a one-off theme:

```bash
waybar-gcal agenda --theme ~/.config/waybar-google-calendar/style.css
WAYBAR_GCAL_THEME=~/.config/waybar-google-calendar/style.css waybar-gcal month
```

Important CSS classes:

- `.panel`, `.topbar`, `.left-pane`
- `.agenda-card`, `.empty-card`
- `.settings-card`, `.settings-row`, `.status-badge`, `.path-label`, `.auth-prompt`
- `.auth-current-status`, `.auth-wizard-page`, `.auth-step-actions`, `.auth-helper-actions`
- `.auth-form`, `.auth-path-row`, `.field-label`, `.text-entry`
- `.title`, `.agenda-header`, `.month-title`, `.event-title`
- `.muted`, `.subtle`, `.accent`
- `.weekday`, `.date-cell`, `.day`
- `.event-date`, `.event-time`, `.pill`
- `.nav-button`, `.action-button`, `.close-button`

The packaged default theme is also installed at:

```text
/usr/share/waybar-google-calendar/themes/apple-light.css
```

## Environment

- `GCAL_DAYS`: deprecated; accepted for older configs.
- `GCAL_CALENDAR`: calendar name or ID filter for agenda.
- `GCAL_TIMEZONE`: IANA timezone override for agenda.
- `GCAL_CACHE_TTL`: cache freshness in seconds, default `300`.
- `GCAL_FETCH_TIMEOUT`: Google API request/auth timeout in seconds, default `25`.
- `WAYBAR_GCAL_CLIENT_SECRET`: OAuth client secret JSON path.
- `WAYBAR_GCAL_THEME`: CSS file appended after the built-in theme.

## Packaging

The AUR package draft lives in `packaging/aur`.
Tag pushes can publish it automatically through the `Publish AUR` GitHub
Actions workflow when the `AUR_SSH_PRIVATE_KEY` repository secret is configured
with the full multiline private key.
The workflow validates AUR SSH access before pushing and can initialize the AUR
git repository on the first publish.

For manual publishing, create the release tag first, then run:

```bash
cd packaging/aur
updpkgsums
makepkg --printsrcinfo > .SRCINFO
```
