# AUR packaging

This directory is a starting point for the AUR package.

Before uploading to AUR:

1. Commit the source changes.
2. Create and push a release tag, for example `v0.1.0`.
3. Run `updpkgsums` in this directory to replace `SKIP` with the real source checksum.
4. Generate `.SRCINFO`:

   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. Copy `PKGBUILD` and `.SRCINFO` into the AUR git repository and push.

OAuth and Google Calendar API access are handled natively by the application.
