# Nib — design spec

**Status:** draft for review · **Date:** 2026-11-26

## 1. What it is

Nib is an offline pastebin: one local store for text snippets, reached from a terminal (`nib`) and a desktop window (Nib). No network, no account, no server. Text in, text out, searchable forever.

**Purpose.** Replace Windows clipboard history as the place text lives. Clipboard history forgets on reboot, cannot be searched, and is a junk drawer. Nib keeps, finds, and orders.

**Users.** You now; a handful of people within weeks; public release in a few months. Windows first.

**Success criteria**

1. `echo "text" | nib` stores it and prints a 4-character ID.
2. `nib search <term>` finds any paste by content, from a cold start, in well under a second at 10,000 pastes.
3. `nib get <id>` prints the paste byte-identical to what went in.
4. The window opens, lists pastes with recognizable names, and shows the selected one without a click.
5. A paste marked secret cannot be read from the database file without its passphrase.
6. `nib` alone in a terminal shows the splash, and reaching a new paste costs two keystrokes.
7. Both surfaces look like one product.

## 2. Decisions locked

| Question | Answer |
|---|---|
| Reach | Offline. Export to a file; the user sends it. No network in the product. |
| Machines | One machine, one store. No sync. |
| Paste shape | Plain text only. No title, tag, folder, file or image. |
| Size | ≤ 64 KB. Reject larger with a clear error. |
| Lifetime | Lives until deleted. Optional per-paste expiry. |
| Search | Full-text over content with snippets, plus unique-prefix match on ID. |
| Versions | None. Edit replaces. |
| Secrets | Per-paste passphrase; optional plaintext label. |
| Store | One SQLite file, FTS5, WAL. |
| CLI verbs | `add ls get copy edit search rm`. |
| CLI output | Bytes to stdout. Colour only when stdout is a TTY. |
| Terminal UI | No browser TUI. Bare `nib` shows a splash launcher with three actions. |
| IDs | 4-char lowercase slug, unique-prefix match. |
| GUI | Tauri (Rust core + web UI), two-pane list + preview. |
| Capture in app | In-app only. No tray, no global hotkey. |
| Packaging | Two binaries over one core; unsigned installer. |
| Name | Nib. |
| Look | Terminal-native. |
| Content highlighting | None in v1. Accent applies to chrome only. |

Dropped deliberately, not forgotten: sharing links, sync, tray, hotkeys, TUI, tags, versions, syntax highlighting, signed installer, auto-update, per-paste language tags.

## 3. Shape

```
nib/
  Cargo.toml                 workspace
  crates/nib-core/           all logic: store, ids, search, crypto, expiry
  crates/nib-cli/            bin `nib`
  app/src-tauri/             Tauri crate, commands only
  app/src/                   HTML/CSS/TS view
  assets/                    nib-mark.svg, nib-appicon.svg, nib-wordmark.svg
  docs/spec.md               this file
  docs/design/               logo, CLI splash, questionnaire
```

**Rule that keeps the two UIs honest:** only `nib-core` opens the database. The CLI and the Tauri commands are both thin callers. No SQL, no crypto, no ID logic outside `nib-core`.

## 4. Data

Default location: `%APPDATA%\nib\nib.db` (override with `NIB_DB`, used by tests).

```sql
PRAGMA journal_mode = WAL;
CREATE TABLE pastes (
  id         TEXT PRIMARY KEY,   -- 4-char slug
  body       TEXT NOT NULL,      -- plaintext, or the sealed envelope when enc = 1
  enc        INTEGER NOT NULL DEFAULT 0,
  label      TEXT,               -- clear-text display name; the only thing a secret leaks
  size       INTEGER NOT NULL,   -- plaintext byte length
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  expires_at INTEGER             -- NULL = never
);
CREATE INDEX pastes_created ON pastes (created_at DESC);
CREATE INDEX pastes_expires ON pastes (expires_at);
CREATE VIRTUAL TABLE pastes_fts USING fts5 (body, tokenize = 'trigram');
```

- **FTS is a copy table**, rowid-aligned with `pastes.rowid`, and holds only `enc = 0` rows. Copying ≤64 KB snippets costs a few MB at ten thousand pastes and buys working `snippet()`.
- **Trigram tokenizer**, not `unicode61`: content is code, and trigram gives substring matching (`"tokio::spawn"`, `parse_args`) that `unicode61` cannot. Trade-off: queries shorter than 3 characters return nothing, and the index is larger. Revisit if it disappoints.
- **Expiry is lazy.** Every command first runs `DELETE FROM pastes WHERE expires_at <= ?` plus the matching FTS rows. No background job, no daemon.
- **Migrations:** `PRAGMA user_version` ladder. Version 1 creates the schema above.

### 4.1 IDs

Alphabet `abcdefghjkmnpqrstuvwxyz23456789` (no `i l o 0 1`), 4 characters, random. Collision → retry. `get`/`copy`/`edit`/`rm` accept any unique prefix; ambiguous prefix → error listing candidates; no match → `no paste <input>`.

> shortcut: 4 chars is ~1M IDs. At 70% occupancy (≈700k pastes) ID generation starts retrying; upgrade to 5 chars then, behind `user_version` 2.

### 4.2 Secrets

`nib add --secret` prompts for a passphrase twice on the terminal. When there is no terminal (the body was piped in) it reads `NIB_PASSPHRASE`, and fails with `passphrase required` rather than hanging if neither is available. Then:

- salt 16 B random, nonce 24 B random, key = Argon2id(passphrase, salt; m=64 MiB, t=3, p=1)
- sealed = XChaCha20-Poly1305(key, nonce, plaintext, aad = paste id)
- `body` = `nibenc1:` + base64(salt ‖ nonce ‖ sealed), `enc = 1`, `size` = plaintext length
- not inserted into FTS; `label` is stored in the clear and used for the list row

Reading: prompt (or `NIB_PASSPHRASE`), wrong passphrase → `cannot unlock <id>` and exit 1. Never print the passphrase, never log it, never write it to disk. Because there is no store-wide key, no unlock screen and no keyring are needed.

## 5. Core API

```rust
pub struct Store { /* Connection */ }

pub struct AddOptions { pub ttl: Option<Duration>, pub secret: Option<Passphrase>, pub label: Option<String> }
pub struct Summary { pub id: String, pub name: String, pub size: u64, pub created_at: i64, pub enc: bool }
pub struct Hit { pub id: String, pub name: String, pub snippet: String }

impl Store {
    pub fn open_default() -> Result<Store>;
    pub fn add(&self, body: &str, opts: AddOptions) -> Result<String>;      // id
    pub fn get(&self, id_or_prefix: &str) -> Result<Paste>;                 // sealed if enc
    pub fn reveal(&self, paste: &Paste, pass: &str) -> Result<String>;      // plaintext
    pub fn list(&self, limit: usize) -> Result<Vec<Summary>>;
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Hit>>;
    pub fn update(&self, id_or_prefix: &str, body: &str) -> Result<()>;
    pub fn delete(&self, id_or_prefix: &str) -> Result<()>;
    pub fn export(&self, id_or_prefix: &str, dest: &Path) -> Result<()>;
}
```

`name` is derived, never stored: first non-empty line, whitespace collapsed, cut at 80 chars; `untitled` if empty; for secrets, `label` if set, else `<id> (secret)`.

## 6. CLI

```
nib add [FILE] [--ttl 24h] [--secret] [--label TEXT]   # stdin when no FILE
nib                     # bare, with piped stdin → add
nib ls [-n 20] [--all]
nib get <id|prefix> [--raw]
nib copy <id|prefix>
nib edit <id|prefix>                                    # $EDITOR on a temp file
nib search <query> [-n 10]
nib rm <id|prefix> [--force]
```

- `nib get` writes the stored bytes. A trailing newline is added only when stdout is a TTY; piped and redirected output is byte-exact, so `nib 4f2 > out.txt` round-trips. Everything human-facing goes to stderr.
- Colour only when stdout is a TTY: ID in accent, age and size in muted, snippet matches underlined. No colour in paste content.
- `ls` row: `k3x9  2m  1.2 KB  fn main() {`. Secrets show `k3x9  2m  ·  secret  deploy key`.
- `copy` uses `arboard`. Not available → exit 1 with a plain message.
- Errors are one line, lowercase, no stack traces. Exit 0 on success, 1 on any failure, 2 on usage error.
- `nib --version` prints `nib <version>` and nothing else. `--help` prints usage. Neither prints art.
- **Splash:** bare `nib`, no arguments, stdin and stdout both TTYs → the launcher described in `docs/design/cli-splash.md`: half-block nib mark, tagline, store path, a bordered 1/2/3 menu (add via `$EDITOR`, search, recent), footer `1-3 choose · q quit`. One key read, no alternate screen, no redraw, no arrow keys. Missing above `60×24` → one usage line instead. Piped stdin never reaches the splash.

## 7. App

One window, two panes: list left (ID, derived name, age; newest first), preview right (monospace, wraps, selectable, read-only). Search field pinned above the list, full-text, debounced 100 ms.

- `N` new paste → inline composer at the top of the preview pane; `Ctrl+Enter` saves, `Esc` cancels.
- `Enter` copies the selected paste. `E` edits. `Del` asks once, then deletes.
- Secret pastes show a locked preview with an unlock field; nothing is decrypted until it is typed.
- Empty store: a one-line hint and the shortcut to add, nothing else.
- Errors appear as a single-line strip at the bottom, never a modal.
- Commands exposed to the UI: `list_pastes`, `get_paste`, `reveal_paste`, `add_paste`, `update_paste`, `delete_paste`, `search_pastes`, `copy_paste`, `export_paste`. Each is a thin wrapper over `nib-core`.

## 8. Look

Terminal-native: the window is a terminal that got wider. No cards, no shadows, no gradients, no rounded corners beyond 2px, no illustrations.

| Token | Light | Dark |
|---|---|---|
| `--paper` | `#fbfbfa` | `#141413` |
| `--ink` | `#141413` | `#e8e6e1` |
| `--muted` | `#6b6a66` | `#8f8d87` |
| `--line` | `#e3e2de` | `#2a2a27` |
| `--accent` | `#b4531f` | `#e07a45` |

One accent only, used for the ID, the cursor and focus rings. Type: `JetBrains Mono` with `ui-monospace, Cascadia Code, Consolas` fallback, 13px UI / 13px content, 1.5 line height, 28px list rows, hairline 1px separators, 16/24px padding. The CLI uses the same accent via 256-colour nearest match.

**Logo.** A fountain-pen nib: a smooth teardrop with a breather hole and a slit, ember on a near-black tile. Sources, all present in `assets/`: `nib-mark.svg` (currentColor, for the app and docs), `nib-appicon.svg` (512×512 app icon), `nib-wordmark.svg` (stroke-drawn `nib` wordmark). The terminal renders the same silhouette in half-block cells — `docs/design/logo.md`. No badge, no swoosh, no outline, no shadow.

## 9. Packaging

- `cargo install --path crates/nib-cli` for early users; `nib.exe` alone is portable.
- Later: a Tauri NSIS installer containing both binaries, unsigned. Expect a SmartScreen warning; say so in the README rather than fight it.
- Windows 10/11 with WebView2 (present on current builds). Code stays portable — no Windows-only APIs in `nib-core` — but macOS and Linux are untested in v1 and unclaimed.

## 10. Testing

- `nib-core`: temp-file SQLite per test. Cases: add→get round-trip byte-exact; 64 KB accepted, 64 KB + 1 rejected; prefix match, ambiguous prefix, missing ID; search finds a substring across lines, ignores secrets; expiry purges on the next command; wrong passphrase fails and leaves the row intact; delete removes the FTS row; migration from empty.
- `nib-cli`: `assert_cmd` end-to-end for each verb, plus `cat | nib` and `nib get > file` byte equality. One test asserts no block characters appear on any non-TTY invocation.
- App: manual checklist for the ten interactions above. No GUI automation in v1.
- One test asserts no passphrase ever reaches stdout or stderr.

## 11. Risks

1. **Tauri toolchain on Windows** (WebView2, Node, Rust all present and cooperating) is the most likely place to lose a day. Proof it builds before any UI work.
2. **Per-paste passphrases make reading a secret a transaction.** Mitigated by `NIB_PASSPHRASE` and by labels. If it still stings in daily use, a store-wide key is a `user_version` 2 change, not a rewrite.
3. **Trigram FTS is a judgment call.** If 3-character minimum or index size annoys, `unicode61` plus a prefix column is a schema change, not a redesign.
4. **Unsigned Windows installer** will scare the few people you hand it to. Planned, documented, not fixed.

## 12. Not in v1

Network, sync, sharing, tray, global hotkey, browser TUI, tags, titles, folders, versions, syntax highlighting, signed installer, auto-update, i18n, telemetry.
