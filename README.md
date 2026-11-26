# nib

```
 ▄███▄
███████
███████
███████
 ██ ██
 ██ ██
  █▄█
   ▀
```

An offline pastebin: one local store for text, reachable from the terminal and from a desktop window. No network, no account, no sync.

**Status: pre-alpha.** The design is written and the mark is drawn; there is no code yet.

## What it will do

- `echo "text" | nib` stores it and prints a four-character ID
- `nib search <term>` finds any paste by its content, with a snippet
- `nib get <id>` prints it back, byte-exact when piped
- optional per-paste passphrase encryption for secrets
- a two-pane desktop app over the same SQLite file
- one launcher when you type `nib` alone

## Docs

- [Design spec](docs/spec.md) — scope, data model, CLI surface, app, packaging
- [CLI splash](docs/design/cli-splash.md) — the screen and its rules
- [Logo](docs/design/logo.md) — the terminal mark, the SVG assets, the ramp
- [Questionnaire](docs/design/questionnaire.md) — the questions the design answers

## Install

Not yet. Later: an unsigned Windows installer, and `cargo install --git` for the CLI. The crate names `nib` and `nib-cli` are taken on crates.io, so there will be no `cargo install nib`.

## License

MIT
