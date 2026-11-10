# Contributing

Thanks for wanting to make Skrap better. The most useful help, by far: **cleaning rules** for the apps and games you know, tested on your own PC.

## Before anything

- Skrap's promise is that nothing leaves without being seen, and that what matters can come back. A change that weakens it will not be merged, however useful it looks.
- Skrap never adds a registry cleaner, a "RAM booster", a health score, ads, telemetry or an account. These are not missing features.
- A security problem (Skrap touching a file it should not) goes through [SECURITY.md](SECURITY.md), not a public issue.

## Proposing a rule

1. Read [`docs/RULES.md`](docs/RULES.md).
2. Add the rule to the right file in [`rules/`](rules), with its texts in English and French. When unsure, pick `review`: it keeps files in the Reserve.
3. Check it: `cargo test -p skrap-rules`, then `cargo run -p skrap-cli -- scan --all --lang fr` to see what it finds on your PC (nothing is deleted).
4. In the pull request, say which app version you tested, on which Windows and in which language, and what happens once the files are gone (does the app rebuild them? does it log you out?).

Not ready to write TOML? Open a **Cleaning rule** issue with the folder and what is inside.

## Code

Requirements and commands are in the [README](README.md#building-from-source). Before a pull request:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && npm run check
```

- Rust: the engine (`crates/skrap-core`) stays free of Windows APIs; what it needs from the system comes through its `Context`. Anything that deletes or moves a file needs a test on a temporary folder.
- Interface: texts go in `app/src/lib/locales/en.ts` first, then every other language. Follow the tone: direct, honest, never alarming, and in French, tutoiement.
- Motion: things appear from the left and leave to the right, slanted like the logo; with Windows animations off, simple fades.
- Commits: one change per commit, a subject that says what it does ("Keep the Reserve on the disk the file came from"), and a body that says why.

By contributing, you agree that your work is released under the [GPL-3.0](LICENSE). The Skrap name and logo stay Kryo Studio's: see [TRADEMARKS.md](TRADEMARKS.md).
