# AUR packaging

This directory contains the AUR package metadata.

## Automatic publishing

The GitHub Actions workflow `.github/workflows/publish-aur.yml` publishes this
source package and the `packaging/aur-bin` binary package to AUR when a package
tag in the form `v<pkgver>-<pkgrel>` is pushed.

Required GitHub repository secret:

- `AUR_SSH_PRIVATE_KEY`: an SSH private key whose public key is registered in
  the maintainer's AUR account. Paste the full multiline private key, including
  the `BEGIN` and `END` lines.

Release flow:

1. Update the source version, for example `Cargo.toml`.
2. Commit the source changes.
3. Push a package tag such as `v0.0.1-1`.
4. The workflow builds the binary release asset, updates both AUR package
   directories, runs `updpkgsums`, generates `.SRCINFO`, and pushes `PKGBUILD`
   plus `.SRCINFO` to AUR.

For packaging-only fixes that reuse the same application version, push another
package tag with the incremented release, for example `v0.0.1-2`. The workflow
publishes `pkgver=0.0.1` and `pkgrel=2`.

The workflow validates the private key with `ssh-keygen`, checks AUR SSH access
with `ssh -T aur.archlinux.org help`, and initializes each AUR git repository
on first publish if it does not exist yet.

The workflow can also be run manually with version and `pkgrel` inputs, but the
matching package tag must already exist because the AUR source URL downloads
that tag tarball.

The package disables makepkg LTO with `options=('!lto')` because the Rust
`ring` dependency links C/assembly objects and can fail under makepkg LTO with
undefined `ring_core_*` symbols.

The binary package lives in `packaging/aur-bin` and downloads a GitHub Release
asset built by the workflow.

## Manual publishing

Before uploading to AUR:

1. Commit the source changes.
2. Create and push a package tag, for example `v0.0.1-1`.
3. Run `updpkgsums` in this directory to replace `SKIP` with the real source checksum.
4. Generate `.SRCINFO`:

   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. Copy `PKGBUILD` and `.SRCINFO` into the matching AUR git repository and push.
   Repeat the same process from `packaging/aur-bin` for the binary package.

OAuth and Google Calendar API access are handled natively by the application.
