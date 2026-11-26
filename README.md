<p align="center">
  <img src="assets/nib-appicon.svg" width="128" height="128" alt="nib">
</p>

<p align="center"><b>Text lives here. Nothing leaves the machine.</b></p>

<p align="center">
  github.com/Raghav2012Code/nib — source and spec.<br>
  Liked nib? <a href="https://github.com/Raghav2012Code/nib">Star it there</a>, one click is enough.
</p>

<p align="center">
  <a href="https://raghav2012code.github.io/nib/"><img src="https://img.shields.io/badge/docs-read%20the%20spec-b4531f?style=flat-square" alt="Docs"></a>
  <a href="docs/spec.md"><img src="https://img.shields.io/badge/spec-in%20the%20repo-3d5e80?style=flat-square" alt="Spec in the repo"></a>
  <a href="https://github.com/Raghav2012Code/nib/actions/workflows/ci.yml"><img src="https://github.com/Raghav2012Code/nib/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/status-pre--alpha-8f8d87?style=flat-square" alt="Pre-alpha">
  <img src="https://img.shields.io/badge/license-MIT-3d5e80?style=flat-square" alt="MIT">
</p>

A native Windows app with a terminal beside it, both over one local store. Windows 10 and 11 first. Plain text, up to 64 KB a paste.

**Pre-alpha.** 0.1.0 ships the CLI and the window; nothing is code-signed yet.

## What it is

- **Local.** One SQLite file on your disk. Nothing is uploaded.
- **Searchable.** FTS5 over the text of every paste, matches shown as snippets.
- **Yours.** MIT, no account, no relay, no telemetry, no network calls.

## In the terminal

```
echo "kubectl get pods -A" | nib   # 4f2a
nib search "get pods"              # by content
nib 4f2a > run.txt                 # byte-exact
```

`nib` alone opens a three-line launcher: add, find, recent. No full-screen TUI, no arrow keys, no modes.

## In the window

Two panes: the list on the left, the selected paste on the right. `/` searches, `j` and `k` move, `N` writes, `E` edits, `Enter` copies, `Del` asks once and deletes, `Ctrl+E` exports. Sealed pastes unlock in place. The window follows your OS theme.

## Secrets

Any paste can be sealed with its own passphrase — Argon2id plus XChaCha20-Poly1305. A sealed paste is never indexed, so search cannot leak it, and its label is the only thing stored in the clear.

## Docs

The same pages are published at **https://raghav2012code.github.io/nib/** — built from the Markdown in `docs/`, not a copy of it.

- [Design spec](docs/spec.md) — scope, data model, CLI surface, app, packaging
- [Plan](docs/plan.md) — phases and what each one proved
- [CLI splash](docs/design/cli-splash.md) — the screen, the mark, the rules
- [Logo](docs/design/logo.md) — the mark and the SVG assets
- [Questionnaire](docs/design/questionnaire.md) — the questions the design answers
- [AGENTS.md](AGENTS.md) — how to work in this repo

## Install

Download from [releases](https://github.com/Raghav2012Code/nib/releases):

- **The window** — `nib_0.1.0_x64-setup.exe`. Current-user install, no admin. Unsigned, so SmartScreen warns: "More info", then "Run anyway".
- **The terminal** — `nib.exe`, portable. Drop it anywhere on your `PATH`.

The installer does not touch your `PATH`; that is deliberate.

From source:

```
cargo install --path crates/nib-cli
cd app && npm install && npm run tauri dev
```

The crate names `nib` and `nib-cli` are taken on crates.io, so there will be no `cargo install nib`.

## License

MIT
