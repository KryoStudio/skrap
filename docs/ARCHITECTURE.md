# Architecture

Skrap is a cleaning engine written as a Rust library, and two front ends that use it the same way: the desktop app (Tauri 2 + SvelteKit) and the command line. Only a small elevated helper, later, will do what needs administrator rights.

```
skrap/
├─ crates/
│  ├─ skrap-core     engine: rules, patterns, scan, clean, the Reserve, protected folders
│  ├─ skrap-win      everything Win32: disks, processes, known folders, Recycle Bin, attributes
│  ├─ skrap-rules    the built-in rules (rules/*.toml), embedded and validated
│  └─ skrap-cli      `skrap` on the command line, same engine
├─ rules/            the cleaning rules, one TOML file per module
├─ app/              the desktop app: SvelteKit front end, src-tauri back end
└─ docs/
```

Planned, not there yet: `skrap-helper` (the elevated process, phase 2) and the MFT reader (phase 2).

## The engine

`skrap-core` knows nothing about Windows APIs: everything it needs from the system comes in a `Context` (environment variables, running processes, protected folders, the current time). This keeps it testable on temporary folders, and the same code path for the app, the CLI and the tests.

1. **Rules** (`rule.rs`) are loaded from TOML and validated: unique ids, known risk levels, patterns that start from a known folder variable, texts in English at least.
2. **Patterns** (`pattern.rs`) expand `%LOCALAPPDATA%\NVIDIA\DXCache\*` into real paths. Wildcards (`*`, `?`) can appear in any component (`User Data\*\Cache`); matching is case-insensitive, like Windows.
3. **Scan** (`scan.rs`) walks what each rule targets, in parallel, and never follows a junction or a symbolic link. A rule whose app is running (`running = ["chrome.exe"]`) is reported as skipped, with the reason. Progress is shared through atomics; the app reads it about 30 times a second, whatever the number of files.
4. **Clean** (`clean.rs`) acts on what the scan found and nothing else: a file that changed since the scan is left alone. *Safe* items are deleted (they rebuild themselves), *Review* items go to the Reserve. Every failure is kept with its reason (in use, access denied, gone) so the app can say what it could not clean.
5. **The Reserve** (`reserve.rs`) moves files, it does not delete them. On the system disk it lives in Skrap's local app data folder, on other disks in a hidden `.skrap-reserve` folder at their root: a move on the same disk is instant, whatever the size. Each clean is a batch with a manifest of where every file came from. Batches are freed after 7 days (adjustable), or sooner when the disk runs low.

### What Skrap never touches

`protect.rs` holds the protected folders: Documents, Desktop, Pictures, Videos, Music, Downloads, Saved Games, OneDrive, Dropbox, Google Drive and the Krak library, resolved through Windows' known folders (so a Desktop moved to OneDrive is still protected). A path inside one of them is never scanned nor cleaned, and a rule cannot target a parent of one: the check runs on every path, at scan time and again at clean time. The list is in the code, not in the rules, so no rule can get around it.

## The app

`app/src-tauri` is a thin layer over the engine: Tauri commands start a scan or a clean on a background thread and send their progress as events at 30 Hz at most. The scan's full list of files stays in Rust; the page asks for one rule's files when it opens it.

The front end follows Krak's structure:

| Where | What |
| --- | --- |
| `src/routes/(app)/` | the pages: Clean (home, scan, results, clean), Reserve, Settings |
| `src/lib/*.svelte.ts` | state: settings, scan, Reserve, updates, toasts |
| `src/lib/components/` | title bar, sidebar, dialogs, cards, the scan sweep |
| `src/lib/locales/` | the texts, English as the reference, then French |
| `src/app.css` | the Acide tokens: colors, fonts, motion |

The window has no native frame: the title bar is drawn by the page. The app runs without administrator rights; what needs them is reported, not forced.

### Updates

Releases are built by GitHub Actions only. The in-app updater (Tauri's) downloads `latest.json` from the latest GitHub release and checks every installer against the public key in `tauri.conf.json`: an update that is not signed with Skrap's key is refused.

At launch, when a new version is out, Skrap offers it in the middle of the screen: what's new, *Update* or *Later* (which puts that version off for a day). Once updated and restarted, Skrap shows what changed in this version, once. See [`RELEASING.md`](RELEASING.md).

## Motion

Everything moves fast and the same way: left to right, slanted like the logo. What appears slides in from the left, what is cleaned flies off to the right with a trail, what comes back from the Reserve returns from the right. Micro-interactions 120 to 180 ms, transitions 240 to 320 ms, big moments 800 to 1,200 ms. With Windows animations off, simple fades.
