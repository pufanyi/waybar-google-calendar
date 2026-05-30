# AUR packaging

This directory contains the AUR package metadata.

## Automatic publishing

The GitHub Actions workflow `.github/workflows/publish-aur.yml` publishes this
package to AUR when a `v*` tag is pushed.

Required GitHub repository secret:

- `AUR_SSH_PRIVATE_KEY`: an SSH private key whose public key is registered in
  the maintainer's AUR account. Paste the full multiline private key, including
  the `BEGIN` and `END` lines.

Release flow:

1. Update the source version, for example `Cargo.toml`.
2. Commit the source changes.
3. Push a tag such as `v0.0.1`.
4. The workflow updates `pkgver`, resets `pkgrel` to `1`, runs `updpkgsums`,
   generates `.SRCINFO`, and pushes `PKGBUILD` plus `.SRCINFO` to AUR.

The workflow validates the private key with `ssh-keygen`, checks AUR SSH access
with `ssh -T aur.archlinux.org help`, and initializes the AUR git repository on
the first publish if it does not exist yet.

The workflow can also be run manually with a version input, but the matching
GitHub tag must already exist because the AUR source URL downloads the tag
tarball.

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
