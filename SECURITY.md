# Security

Skrap deletes files. A flaw in it can cost someone their data, so security reports come first.

## Reporting a problem

Please do not open a public issue for a security problem or for anything that made Skrap touch a file it should not have. Report it privately instead, through GitHub: **Security → Report a vulnerability** on this repository.

Say what happened, on which Windows version and language, with which Skrap version, and, if you can, the rule involved (`skrap-cli rules` lists them). You will get an answer within a few days, and a fix ships as an update as soon as it is ready. You will be credited in the release notes unless you prefer not to be.

## What counts

- Skrap deleted, moved or changed a file outside what the results showed.
- Skrap reached a protected folder: Documents, Desktop, Pictures, Videos, Music, Downloads, Saved Games, OneDrive, Dropbox, Google Drive, iCloud Drive, or the Krak library.
- A file put in the Reserve could not be restored.
- The update check, or an update, could be made to install something that is not an official Skrap release.
- Anything that lets another program use Skrap to delete files.

## How releases are protected

Releases are built only by GitHub Actions from this repository, and in-app updates are signed: an update that is not signed with Skrap's key is refused. The checksum of every installer is in its release notes. See [`docs/RELEASING.md`](docs/RELEASING.md) and [`docs/SIGNING.md`](docs/SIGNING.md).
