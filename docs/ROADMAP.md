# Roadmap

Five phases, each closed by a gate: a measurable bar to clear before moving on. No dates: they depend on the time left next to Krak.

Phase 3 is the real public launch. Everything after it ships through updates, one module at a time.

## Phase 0 — Foundations

The repository, the engine and the app shell, so that every later module only adds rules and screens.

- [x] Cargo workspace: `skrap-core` (rules, scan, clean, Reserve, protected paths), `skrap-win` (Win32), `skrap-rules` (built-in rules), `skrap-cli`
- [x] Rule format (TOML), with risk levels, explanations in every language, "skip while this app runs"
- [x] Hard-coded protected folders that no rule can reach
- [x] The Reserve: moved, not deleted, on the same disk; restore; purge after 7 days or when the disk runs low
- [x] Tauri 2 + SvelteKit app: title bar, sidebar, home, scan, results, clean, Reserve, settings
- [x] Signed in-app updates, offered in the middle of the screen at launch, then "What's new" once updated
- [x] French and English
- [x] CI (format, lints, tests, type checks, build, dependency advisories) and the release workflow
- [ ] The GitHub repository, its `TAURI_SIGNING_PRIVATE_KEY` secret, and a first tagged release

**Gate:** a build from a tag installs, updates itself from a newer tag, scans a real PC in under 5 seconds and restores everything it put in the Reserve.

## Phase 1 — The core

- [ ] System: Windows and user temp files, Windows Update and Delivery Optimization caches, error reports and dumps, logs, thumbnail and icon caches, Recycle Bin, Windows.old (with a warning)
- [ ] Browsers: Chrome, Edge, Firefox, Brave, Opera, Vivaldi, Zen. Caches by default; cookies and history only if ticked, with a list of sites to stay signed in to
- [ ] Gaming: DirectX, NVIDIA, AMD and Intel shader caches; Steam, Epic, Battle.net, Riot, EA and Ubisoft caches; driver installer leftovers (`C:\NVIDIA`, `C:\AMD`); game logs and dumps
- [ ] Apps: Discord, Spotify, Teams, Slack, VS Code, then hundreds more through BleachBit's CleanerML definitions
- [ ] Windows restore point before any system change
- [ ] Every rule tested on a French Windows

**Gate:** the whole rule set runs on three real gaming PCs (one French Windows) without a single file outside its rule touched, and the Reserve brings back everything.

## Phase 2 — Private beta

- [ ] The admin helper: a small elevated process that only accepts a closed list of commands over a named pipe
- [ ] Disk analysis: visual map of the space, largest files and folders, read straight from the MFT
- [ ] Crash reports written locally, sent only if the user chooses to
- [ ] Ghost games: folders of uninstalled games still on disk
- [ ] Tray icon with a quick scan (Safe rules only), low-disk alert under 10% (a scan is offered, never started on its own)

**Gate:** 50 testers from the Krak community, zero broken PC, zero data loss reported.

## Phase 3 — Public launch

- [ ] Code signing (SignPath Foundation) and the Microsoft Store
- [ ] 12 languages, like Krak
- [ ] Website, Discord channel, press kit
- [ ] VirusTotal check and Microsoft submission of every release

**Gate:** a signed release with no SmartScreen warning, and no antivirus engine flagging it.

## Phase 4 — Waves

Each module ships with a preview and an undo.

- [ ] Duplicates, similar images, empty files and folders, broken shortcuts (`czkawka_core`)
- [ ] Uninstaller: Win32, Store and Steam apps, in bulk, then their leftovers
- [ ] Startup: apps at startup, scheduled tasks, browser extensions, with their estimated impact
- [ ] Windows privacy: telemetry, ads, Copilot, Recall, suggestions. Each setting explained, a "Balanced" preset, all reversible
- [ ] Developer caches: npm, pnpm, Cargo, pip, Gradle; `node_modules` and `target` folders of abandoned projects
- [ ] Scheduled cleaning: a quiet weekly scan and clean on Safe rules only, with a summary notification
- [ ] Command line and portable version: `skrap scan --profile gaming --dry-run`
- [ ] Community rules: new signed rules downloaded without updating the app
