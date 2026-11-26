# Local Pastebin — discovery questionnaire

**Purpose:** settle the decisions that change the architecture, so the spec is not built on guesses.

**From:** you (owner), **To:** you (the one holding the answers), **How your answers will be used:** they become the design spec; the spec becomes the build plan.

## Context

We're building an **offline** pastebin in Rust: a CLI and a desktop app over one local store, plus a mark telling them apart nowhere and joining them everywhere. Nothing in `I:\projects\local-pastebin` yet. Two UI front-ends over one store means the store boundary matters more than either UI: if it isn't a small shared core, the two will drift.

Assumed unless you correct it: single user, single machine, Windows-first, text pastes, no network at all, no crypto beyond what the disk already has.

Answer what you care about, skip what you don't. **"I don't know" is a valid answer** and I'll pick. Time: 10 minutes, no deadline.

## 1. What it is for

### 1.1 Who runs this?
Just you on your machine, or do other people install it too?

_Why this matters: distribution and tolerance for setup steps live or die on this answer._

>
### 1.2 What do you do today instead, and what annoys you about it?
Notepad file? Windows clipboard history? GitHub Gists? A folder of `.txt`? Name the specific annoyance.

_Why this matters: this is the real spec. Everything the replacement does must be faster than the thing you're escaping._

>
### 1.3 Does a paste ever leave the machine?
Copy to clipboard, export to file, hand to a colleague, paste into a browser — or is "offline" absolute, read-back-only?

>
### 1.4 One machine only, or the same store synced across two?
If two: same OS on both, or Windows + Linux/macOS?

>

## 2. Pastes and storage

### 2.1 What can a paste be?
Only plain text? Or also: language tag for syntax highlighting, title, tags, files, images?

>
### 2.2 How big do pastes get, at most?
Typical is bytes; the ceiling is what decides whether the list and search stay instant.

>
### 2.3 Does a paste expire, or live until deleted?
A TTL / "burn after read" mode, or simply manual delete?

>
### 2.4 Do you need to find a paste by its content, not just its name?
Full-text search with snippets? Fuzzy on title/ID only? Both?

>
### 2.5 Does a paste have versions?
Edit in place and lose the old, or keep history of edits?

>
### 2.6 Encryption at rest?
Password on the store, or is "BitLocker + a locked laptop" good enough?

_Why this matters: it decides the store format and kills or allows FTS5 search over plaintext._

>
### 2.7 Storage format — pick your poison:
- **A. One SQLite file** (`rusqlite` + FTS5): instant search, WAL so CLI and app write safely at once, one file to back up. Opaque to `grep`, needs a tool to read.
- **B. A folder of `.md`/`.txt` files + an index**: greppable, syncs with anything, you can fix it by hand. You maintain the index and the search yourself.
- **C. Both**: files as truth, SQLite as rebuildable cache. Costs the most code, buys the most freedom.

>
## 3. CLI surface

### 3.1 Which verbs must exist on day one?
`pb` add / list / get / copy / search / delete / edit / tags — what else is essential, what's noise?

>
### 3.2 How do you read one back?
Straight to stdout (pipeable), syntax-highlighted to the terminal, or into a pager?

>
### 3.3 Do you want an interactive terminal browser (TUI: list, preview, search, copy)?
Or plain subcommands only — `pb ls`, `pb get 4f2a`?

_Why this matters: a TUI is a second whole UI to build and design. Fine to want; not free._

>
### 3.4 ID scheme: short slug (`pb get k3x9`), incrementing number (`pb get 12`), or content hash?

>
### 3.5 Piping in and out must work both ways — `cat log.txt | pb -` and `pb 12 > out.txt`. Anything else you'd pipe?

>
## 4. Desktop app

### 4.1 Which layer for the GUI?
- **A. Native Rust (Slint or Iced)**: one binary, no web stack, no WebView. Less design control; Slint is the more designer-friendly of the two.
- **B. Tauri (Rust core + HTML/CSS UI)**: pixel-perfect design control, web ecosystem, but needs WebView2 on Windows and a JS toolchain to build.
- **C. No app at first**: CLI + a `pb serve` local web view in the browser, app later.

_Why this matters: it's the single most expensive choice here and it's very hard to reverse later._

>
### 4.2 What's on screen the moment it opens?
Two-pane list + preview? Search box first? Whatever was last open?

>
### 4.3 Which interactions are must-have in the app?
Create, copy, search, delete, tag/folder, edit — and do you want a **global hotkey / tray icon** ("capture the clipboard from any app")?

_Why this matters: tray + hotkey is a real feature with real platform work, not a checkbox._

>
### 4.4 One binary or two?
`pb` for terminal and `pb --gui` for the window, or two separate executables sharing a core crate?

>
## 5. Identity

### 5.1 What's the name?
`local-pastebin` is the folder, not a product. Name drives the binary name, app ID, wordmark and logo. Do you have one, or want three options from me?

>
### 5.2 Where must the logo survive?
Terminal at 16-ish cells in monochrome (ANSI/ASCII), 16×16 tray icon, 256×256 app icon, and a wide wordmark — all from one mark?

>
### 5.3 Design feel, pick a direction:
- **A. Terminal-native**: monospace, high contrast, one accent, keyboard-first, no chrome.
- **B. Warm paper**: cream background, serif headings, ink-black text, feels like a notebook.
- **C. Dark IDE**: deep charcoal, muted syntax colors, dense rows.
- **D. Industrial**: hairlines, uppercase micro-labels, no rounded corners.

>
## 6. Build and ship

### 6.1 How does it get installed?
`cargo install` only? Signed `.msi`? Portable `.exe` on a USB stick?

>
### 6.2 Must the CLI and app install as one thing, or can the CLI be usable alone?

>
## Anything else?

What did I not ask that will annoy you later if it's missing?

>
