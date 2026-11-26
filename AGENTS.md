# AGENTS.md

Nib is an offline pastebin: one SQLite store, a CLI (`nib`) and a Tauri window over it. No network code anywhere. Read `docs/spec.md` before changing behaviour; it is the contract.

## Layout

| Path | What lives there |
|---|---|
| `crates/nib-core/` | All logic: store, ids, search, sealing, expiry. The only crate that opens SQLite. |
| `crates/nib-cli/` | `bin nib`. Verbs, stream rules, splash. |
| `app/` | Tauri 2 + Vite + plain TypeScript. `src-tauri` is excluded from the root workspace. |
| `app/src/mock.ts` | Dev-only IPC stand-in so the UI runs in a browser. Must never ship. |
| `assets/` | Mark SVGs. `docs/design/` explains them. |
| `docs/spec.md` | Behaviour contract. `docs/plan.md` tracks phases. |

## Commands

```powershell
cargo test                       # core + CLI, 22 tests
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo install --path crates/nib-cli
cd app; npm run tauri dev        # the window
cd app; npm run build            # tsc + vite, must pass before any app commit
cargo check --manifest-path app/src-tauri/Cargo.toml
```

`NIB_DB` overrides the store path (use a temp file in every manual check, never the real `%APPDATA%\nib\nib.db`). `NIB_PASSPHRASE` supplies a passphrase when there is no terminal.

## Invariants

- Only `nib-core` opens the database. The CLI and the Tauri commands are thin callers; no SQL, no crypto, no id logic outside the core.
- Sealed bodies never enter the FTS index; only their label does.
- IDs are 4 characters from `abcdefghjkmnpqrstuvwxyz23456789`, resolvable by unique prefix.
- A paste is at most 64 KB. Reject, never truncate.
- Output contract: pastes to stdout, chrome to stderr, a trailing newline only when stdout is a TTY.
- The window is keyboard-first and modal-free: errors and confirmations appear in the footer strip.

## Testing

- New non-trivial logic (a branch, a loop, a parser, anything touching money, security or data loss) leaves a test.
- Core tests use a temp-file database per test; CLI tests use `assert_cmd` with `NIB_DB` set.
- Never write to the real store from a test.
- `cargo test` at the root does not build the app. Run `npm run build` and the `cargo check` above when touching `app/`.

## Style

- Terminal-native: monospace everywhere, hairlines, 2px radius, no shadows, no gradients, no illustrations.
- Tokens live in `app/src/styles.css` and nowhere else: `--paper`, `--ink`, `--muted`, `--line`, `--accent`, `--danger`. Accent is `#b4531f` on light, `#e07a45` on dark.
- Accent marks identity (ids, selection, focus). It is not a decoration colour.
- Copy is sentence case, specific, and never apologises.

## Environment traps

- The toolchain must be MSVC (`stable-x86_64-pc-windows-msvc`). `rusqlite` compiles bundled SQLite through `cl.exe`, and Tauri does not support the GNU target. A missing Visual C++ workload shows up as `link.exe not found`; a half-installed Windows SDK shows up as `LNK1181: cannot open input file 'dbghelp.lib'`.
- This machine's `I:` drive is nearly full. `target/` and `app/src-tauri/target/` are junctions to `F:\cargo\...` and are not in git. If a build fails with `ENOSPC` or "not enough space", recreate them:
  ```powershell
  Remove-Item target -Recurse -Force
  New-Item -ItemType Junction -Path target -Target F:\cargo\nib-root
  ```
  and the same for `app/src-tauri/target` → `F:\cargo\nib-app`.
- Vite dev needs `TEMP`/`TMP` on a drive with room (`F:\Temp\opencode`), otherwise `npm install` reports `nospc` even when the project's own drive looks free.

## Do not commit

`target/`, `node_modules/`, `dist/`, `*.db*`, `.openchamber/`, `.playwright-mcp/`, or anything from a real store.

## Before a release

1. `Select-String dist/assets/*.js -Pattern 'mockApi|ssh-ed25519'` must find nothing.
2. `cargo fmt --all --check`, clippy with `-D warnings`, `cargo test`, `npm run build`.
3. Bump `version` in `Cargo.toml`, `app/src-tauri/Cargo.toml`, `app/package.json` and `app/src-tauri/tauri.conf.json` together.
4. `cd app; npm run tauri build` produces an unsigned NSIS installer; note the SmartScreen warning in release notes.
