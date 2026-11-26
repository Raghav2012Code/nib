# Nib — implementation plan

Five phases, in order. Each one ends in something runnable, so nothing is written blind.

## Phase 0 — toolchain

The machine currently has `stable-x86_64-pc-windows-gnu` and no `cl.exe`. Both the store and the app need MSVC.

1. Install VS Build Tools with the "Desktop development with C++" workload. Needs admin; ~3–6 GB. `winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools"` works.
2. `rustup toolchain install stable-x86_64-pc-windows-msvc` then `rustup default stable-x86_64-pc-windows-msvc`.
3. Prove it: a throwaway crate with `rusqlite = { features = ["bundled"] }` must compile and run a query.

**Done when:** `cargo build` and the SQLite smoke test pass on the MSVC toolchain. If this fails, the rest is blocked and the pure-Rust store becomes the fallback — decide then, not later.

**Done 2026-11-26.** Build Tools 2022 installed with the C++ workload and Windows SDK 10.0.26100; `rustup default stable-x86_64-pc-windows-msvc`; SQLite compiles and the full suite is green. One trap worth remembering: while the SDK was still installing, every link failed with `LNK1181: cannot open input file 'dbghelp.lib'`. That is an incomplete SDK, not a broken toolchain — wait for `Windows Kits\10\Lib\<version>\um\x64\dbghelp.lib` to exist.

## Phase 1 — core

Files: `Cargo.toml` (workspace), `crates/nib-core/{Cargo.toml, src/{lib,store,ids,crypto,error}.rs}`.

1. Schema v1 and the `PRAGMA user_version` ladder; `open_default()` reading `%APPDATA%\nib\nib.db`, `NIB_DB` override for tests.
2. IDs: 4-char alphabet, collision retry, unique-prefix resolution with an ambiguity error.
3. `add`/`get`/`list`/`update`/`delete`, 64 KB rejection, derived display name.
4. FTS5 `trigram` copy table: insert/delete in the same transaction as the row; `search` returning `snippet()`; sealed rows never indexed, labels indexed.
5. Lazy expiry purge on every open.
6. Crypto: Argon2id (m=64 MiB, t=3, p=1) + XChaCha20-Poly1305 with the id as AAD, `nibenc1:` envelope, `reveal`, wrong-passphrase error.

Tests (temp file DB per test): round-trip byte-exact, 64 KB boundary, prefix and ambiguity, search across lines, sealed paste not searchable by body, label searchable, expiry purge, wrong passphrase leaves the row, delete clears FTS, migration from empty.

**Done when:** `cargo test -p nib-core` is green on Windows and Ubuntu runners.

## Phase 2 — CLI

Files: `crates/nib-cli/{Cargo.toml, src/{main,args,render,splash}.rs}`.

1. `clap` surface: `add`, `ls`, `get`, `copy`, `edit`, `search`, `rm`, plus bare `nib` reading piped stdin.
2. Stream rules: paste to stdout, chrome to stderr; newline only on a TTY; colour only on a TTY; exit 0/1/2.
3. `--secret` prompting, `NIB_PASSPHRASE` fallback, fail rather than hang.
4. `copy` via `arboard`; `edit` via `$EDITOR` on a temp file.
5. Splash: the 22-line screen, the mark's 10 rows, one `crossterm` key read, `1/2/3/q`, `60×24` fallback, never on a pipe.

Tests: `assert_cmd` per verb, `cat x | nib`, `nib <id> > file` byte equality, no block characters on non-TTY output.

**Done when:** every verb is usable by hand and `nib` alone in a terminal shows the splash.

## Phase 3 — app

Files: `app/` (Tauri v2), `app/src-tauri/src/{main,commands}.rs`, `app/src/{index.html,main.ts,style.css}`.

1. `npm create tauri-app` skeleton, Vite, TS, no framework; confirm it builds before touching the UI.
2. Commands as thin wrappers over `nib-core`: list, get, reveal, add, update, delete, search, copy, export.
3. Two panes, search field debounced 100 ms, `N`/`Enter`/`E`/`Del` shortcuts, secret unlock field, empty state, single-line error strip.
4. The design tokens from spec §8: `--paper`, `--ink`, `--muted`, `--line`, `--accent`, JetBrains Mono with fallbacks, 28px rows, hairlines, no shadows.

**Done when:** the ten-interaction checklist in spec §10 passes by hand.

**Done 2026-11-26.** Tauri 2.12, Vite 8, plain TypeScript. `src-tauri` is excluded from the root workspace so `cargo test` at the root stays fast; the app builds through `npm --prefix app run tauri dev`. Nine commands wrap `nib-core`, with no SQL or crypto outside the core. The window follows the OS theme. `app/src/mock.ts` is a dev-only stand-in for the IPC so the UI can be inspected in a plain browser; Vite removes it from production builds (checked: the built bundle contains no mock data).

Verified: the window opens with the title `nib`, the first `list_pastes` call creates the store, the two panes render from real IPC data, the search field, composer, seal fields, unlock panel and delete confirmation all work, and the icons are generated from `assets/nib-appicon.svg`.

Not verified by me: the native window's own pixels — a desktop screenshot captured the wrong window and exposed unrelated content, so it was deleted rather than repeated. The UI itself was verified in Chromium against the same CSS and TypeScript.

## Phase 4 — ship

1. CI workflow: `fmt --check`, `clippy -D warnings`, `test` on `windows-latest` and `ubuntu-latest`. **Done**, plus a third job that installs the app, builds the frontend with `tsc` and checks the Tauri crate on Windows.
2. `tauri build` NSIS installer containing both binaries, unsigned; README notes the SmartScreen warning.
3. Tag `v0.1.0`; release notes list what works and what is deliberately missing.

**Done when:** the installer installs on a clean Windows install and `nib` works from a fresh PowerShell.

**Remaining:** steps 2 and 3.

**Done 2026-11-26.** `npm run tauri build` produced `nib_0.1.0_x64-setup.exe` (2.09 MiB, NSIS, current-user, unsigned); `cargo build --release -p nib-cli` produced a portable `nib.exe` (2.64 MiB). Both are attached to the `v0.1.0` release.

Verified on this machine: the CLI stored and read back a paste; the release window opened with the title `nib`; the installer ran silently, landed in `%LOCALAPPDATA%\nib` as `nib-app.exe` and `uninstall.exe`, registered `nib 0.1.0` in the uninstall list, launched, created its store, and then removed itself and its registry entry cleanly. Not verified: a truly clean Windows install, and signing.

The installer does not edit `PATH`; the CLI ships beside it as a portable binary instead. Putting a directory into a user's `PATH` from NSIS is the one place an installer can quietly break a machine, and a pre-alpha does not earn that risk.

## Order and why

Core before CLI, CLI before app. The core is where the risky decisions live (FTS schema, crypto envelope); the CLI exercises all of it with the smallest possible surface; the app is a view and should not be able to hide a broken store. The alternative — app first — spends the most time on the part that can be rebuilt cheapest.

## Risks that would change this plan

- **VS Build Tools install fails or is refused.** Stop at Phase 0 and pick the pure-Rust store; that is a different plan, not a variant of this one.
- **Trigram search disappoints** (index size, 3-character floor). The schema change is contained in `crates/nib-core/src/store.rs` plus a migration; it does not reach the CLI or the app.
- **Tauri v2 scaffolding fights the workspace.** Fall back to `app/` as a standalone crate outside the workspace members, still calling `nib-core` by path.
