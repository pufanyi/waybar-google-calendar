# waybar-google-calendar

GTK4/Relm4 Google Calendar popup for Waybar.

This project provides a small native Linux desktop popup intended for Waybar's
clock module:

- `waybar-gcal agenda --days 7` shows upcoming Google Calendar events.
- `waybar-gcal month` shows a local month calendar.
- `waybar-gcal auth` starts Google Workspace CLI calendar authentication.
- `waybar-gcal print-theme` prints the built-in CSS theme.

The UI is built with Relm4 on top of GTK4/libadwaita. The Google API integration
is delegated to `googleworkspace-cli` (`gws`) so this project can focus on the
Waybar popup UI, cache behavior, and desktop packaging.

## Dependencies

Runtime:

- `gtk4`
- `libadwaita`
- `googleworkspace-cli`

Build:

- `rust`
- `cargo`

On Arch Linux:

```bash
sudo pacman -S gtk4 libadwaita googleworkspace-cli rust
```

## Build

```bash
cargo build --release
```

## Authentication

```bash
waybar-gcal auth
```

This runs:

```bash
gws auth login --services calendar --readonly
```

## Waybar

Use `examples/waybar-clock.json` as a starting point:

```json
{
  "clock": {
    "on-click": "waybar-gcal agenda --days 7",
    "on-click-right": "waybar-gcal month"
  }
}
```

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

- `GCAL_DAYS`: default agenda range in days.
- `GCAL_CACHE_TTL`: cache freshness in seconds, default `300`.
- `GCAL_FETCH_TIMEOUT`: `gws` fetch timeout in seconds, default `25`.
- `WAYBAR_GCAL_THEME`: CSS file appended after the built-in theme.

## Packaging

The AUR package draft lives in `packaging/aur`.

Before publishing, create a release tag and run:

```bash
cd packaging/aur
updpkgsums
makepkg --printsrcinfo > .SRCINFO
```
