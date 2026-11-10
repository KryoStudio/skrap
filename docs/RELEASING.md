# Releasing a version

Releases are built by GitHub Actions only, from the public code: nobody builds a release on their own PC. That is the lesson of CCleaner 2017, where a compromised build machine shipped malware to 2.27 million PCs.

## Once: the update signing key

The in-app updater only installs an update signed with Skrap's key. The public half is in [`app/src-tauri/tauri.conf.json`](../app/src-tauri/tauri.conf.json) (`plugins.updater.pubkey`); the private half never enters the repository.

1. The key pair was made with `npx tauri signer generate -w %USERPROFILE%\.tauri\skrap-updater.key` (in `app/`). Keep a copy of `skrap-updater.key` somewhere safe and offline: **without it, installed copies of Skrap can never be updated again.**
2. In the repository: **Settings → Secrets and variables → Actions → New repository secret**, name `TAURI_SIGNING_PRIVATE_KEY`, value the content of `skrap-updater.key`. If the key has a password, add it as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
3. Turn on two-factor authentication on the GitHub account.

## Every version

```bash
node scripts/version.mjs set 0.2.0
```

writes the version in `Cargo.toml`, `app/package.json` and `app/src-tauri/tauri.conf.json`. Then:

1. Add a `## 0.2.0` section at the top of [`CHANGELOG.md`](../CHANGELOG.md), and the same notes, short, in every language, at the top of [`app/src/lib/changelog.ts`](../app/src/lib/changelog.ts): the app shows them once after the update ("What's new").
2. `node scripts/version.mjs check`, then commit ("Version 0.2.0").
3. Tag and push: `git tag v0.2.0 && git push && git push origin v0.2.0`.
4. The [Release workflow](../.github/workflows/release.yml) checks that the tag matches the version, builds the installer, signs the update files, and drafts a GitHub release with `Skrap_0.2.0_x64-setup.exe`, its `.sig`, `latest.json`, the dependency list (SBOM), the installer's SHA-256 and, once configured, its VirusTotal report.
5. Edit the draft: the first paragraph of the release notes is what the update dialog shows ("A new version is ready…" when there is none), so write it for players, in one or two sentences. Then **Publish**.

From that moment, every installed Skrap that looks for updates finds it.

## What users see

- At launch (unless turned off in the settings), Skrap reads `latest.json` from the latest published release. When the version is newer, a dialog comes up in the middle of the screen: the version, the first paragraph of the notes, **Update** or **Later**.
- **Later** puts that version off for a day; a small "Update 0.2.0" reminder stays in the title bar. **Settings → Check now** asks at any time.
- **Update** downloads the installer, checks its signature against the public key, installs it without questions (`passive` mode) and restarts Skrap.
- Restarted in the new version, Skrap shows "What's new in 0.2.0" once, from `changelog.ts`.

A draft release is invisible to the updater: nothing reaches users before **Publish**. To pull a bad version, publish a newer fixed one (the updater only moves forward).

## Code signing and antivirus

Windows code signing (SignPath) and VirusTotal scans are optional and off until configured: see [`SIGNING.md`](SIGNING.md). Until then, Windows shows SmartScreen's warning on the downloaded installer; updates made from inside the app are not affected.
