# Writing a cleaning rule

Every rule lives in [`rules/`](../rules), one TOML file per module, and is reviewed like code. The tests in `crates/skrap-rules` load them all: a rule that does not parse, misses a language or aims at a user folder never reaches a release.

```toml
[[rule]]
id = "gaming.nvidia.shader_cache"         # category.app.what: a-z, 0-9, _
category = "gaming"                        # system, browsers, gaming, apps
risk = "review"                            # safe, review (expert: later)
group = { en = "NVIDIA", fr = "NVIDIA" }   # the app or place, shown above its rules
name = { en = "Shader cache", fr = "Cache de shaders" }
explain = { en = "Shaders the NVIDIA driver compiled for your games. Rebuilt on their own, but the next launch of each game can stutter for a moment.", fr = "…" }
paths = [
  '%LOCALAPPDATA%\NVIDIA\DXCache',
  '%LOCALAPPDATALOW%\NVIDIA\PerDriverVersion\DXCache',
]
running = ["steam.exe"]                    # optional: skipped while one of these runs
older_than_hours = 24                      # optional: only files untouched this long
exclude = ["*.lock"]                       # optional: names or full patterns to leave alone
admin = true                               # optional: usually needs administrator rights
```

Use single quotes for paths: TOML keeps the backslashes as they are.

## Risk levels

| Risk | Examples | Ticked | Once cleaned |
| --- | --- | --- | --- |
| `safe` | temporary files, browser caches, dumps | yes | deleted: it rebuilds itself |
| `review` | shader caches (the next launch can stutter), Windows.old, blue screen dumps | no | kept 7 days in the Reserve |
| `expert` | services, registry, Windows features | no, and hidden | restore point and `.reg` export first (phase 2) |

When in doubt, pick `review`. A cleaner that breaks a PC once is dead.

## Paths

- A path starts with a folder variable: `%TEMP%`, `%TMP%`, `%LOCALAPPDATA%`, `%LOCALAPPDATALOW%`, `%APPDATA%`, `%PROGRAMDATA%`, `%USERPROFILE%`, `%WINDIR%`, `%SYSTEMDRIVE%`, `%PROGRAMFILES%`, `%PROGRAMFILES(X86)%`, `%STEAM%` (Steam's folder, from the registry). A variable this PC does not have makes the path name nothing.
- A folder means everything inside it; the folder itself stays. Folders emptied by the clean below it go too.
- `*` and `?` work in any part: `User Data\*\Cache` reaches every browser profile. Matching ignores case, like Windows.
- Right after the variable comes a named folder, never a wildcard (`%LOCALAPPDATA%\*\Cache` is refused), so that no rule can sweep a whole profile. `%TEMP%` and `%TMP%` are the exception: they are junk as a whole.
- Junctions, symbolic links and cloud placeholders are never followed nor cleaned.

## What no rule can reach

Documents, Desktop, Pictures, Videos, Music, Downloads, Saved Games, OneDrive, Dropbox, Google Drive, iCloud Drive and the Krak library, wherever Windows says they are. The list is in [`crates/skrap-win`](../crates/skrap-win/src/lib.rs) and checked by [`crates/skrap-core/src/protect.rs`](../crates/skrap-core/src/protect.rs) on every path, at scan time and again at clean time. A target that holds one of them is left out whole.

## Texts

`group`, `name` and `explain` are written in every language the app speaks (English and French today); English is required and is what a missing language shows. `explain` is one or two sentences in plain words: what it is, and what happens once it is gone. Direct and honest, never alarming:

- Yes: "Copies of pages and images from the sites you visited. Downloaded again as you browse; you stay signed in."
- No: "Dangerous junk slowing down your PC!"

## Checking a rule

```bash
cargo test -p skrap-rules
cargo run -p skrap-cli -- scan --all --lang fr   # what it finds on your PC; nothing is deleted
```

Test a new rule on a French Windows too: folder names can differ, and a rule that finds nothing is better than one that finds the wrong thing.
