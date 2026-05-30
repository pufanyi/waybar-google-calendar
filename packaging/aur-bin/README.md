# AUR binary packaging

This directory contains the AUR metadata for `waybar-google-calendar-bin`.

The `Publish AUR` GitHub Actions workflow builds the release binary, packages
it into a GitHub Release asset, runs `updpkgsums`, generates `.SRCINFO`, and
pushes this package to AUR.

The package tag format is shared with the source package:

```text
v<pkgver>-<pkgrel>
```

For example, `v0.0.1-2` publishes:

- `waybar-google-calendar` with `pkgver=0.0.1`, `pkgrel=2`
- `waybar-google-calendar-bin` with `pkgver=0.0.1`, `pkgrel=2`

The binary package downloads:

```text
https://github.com/pufanyi/waybar-google-calendar/releases/download/v<pkgver>-<pkgrel>/waybar-google-calendar-bin-<pkgver>-<pkgrel>-x86_64.tar.zst
```

`waybar-google-calendar-bin` provides and conflicts with
`waybar-google-calendar`.
