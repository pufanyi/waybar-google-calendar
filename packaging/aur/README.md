# AUR packaging

This directory contains the AUR package metadata.

## Automatic publishing

The GitHub Actions workflow `.github/workflows/publish-aur.yml` publishes this
package to AUR when an upstream `v*` tag or an AUR `aur-v*` package tag is
pushed.

Required GitHub repository secret:

- `AUR_SSH_PRIVATE_KEY`: an SSH private key whose public key is registered in
  the maintainer's AUR account. Paste the full multiline private key, including
  the `BEGIN` and `END` lines.

Release flow:

1. Update the source version, for example `Cargo.toml`.
2. Commit the source changes.
3. Push a tag such as `v0.0.1`.
4. The workflow updates `pkgver`, uses `pkgrel=1`, runs `updpkgsums`,
   generates `.SRCINFO`, and pushes `PKGBUILD` plus `.SRCINFO` to AUR.

For packaging-only fixes that reuse the same upstream tag, push an AUR package
tag with the incremented package release, for example `aur-v0.0.1-2`. The
workflow publishes `pkgver=0.0.1` and `pkgrel=2` while still downloading the
upstream `v0.0.1` source tarball.

The workflow validates the private key with `ssh-keygen`, checks AUR SSH access
with `ssh -T aur.archlinux.org help`, and initializes the AUR git repository on
the first publish if it does not exist yet.

The workflow can also be run manually with version and `pkgrel` inputs, but the
matching upstream GitHub tag must already exist because the AUR source URL
downloads the upstream tag tarball.

The package disables makepkg LTO with `options=('!lto')` because the Rust
`ring` dependency links C/assembly objects and can fail under makepkg LTO with
undefined `ring_core_*` symbols.

## Manual publishing

Before uploading to AUR:

1. Commit the source changes.
2. Create and push a release tag, for example `v0.0.1`.
3. Run `updpkgsums` in this directory to replace `SKIP` with the real source checksum.
4. Generate `.SRCINFO`:

   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. Copy `PKGBUILD` and `.SRCINFO` into the AUR git repository and push.

OAuth and Google Calendar API access are handled natively by the application.
