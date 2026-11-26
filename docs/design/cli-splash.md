# Nib — CLI splash

Structure borrowed from the reference "bin there" screen: centred wordmark, tagline, one factual line, a single bordered action box, a keybind footer, all on a dark terminal. Wording, mark and palette are Nib's own.

## When it appears

Exactly one case: `nib` with no arguments, stdin and stdout both TTYs.

Never on `--help`, `--version`, any subcommand, or when stdin is piped — `echo x | nib` still just stores and prints an ID.

## Screen

Design width 56 columns; everything is centred in the real width, the box spans 56.

```
                          ▄███▄
                        ▄███████▄
                        █████████
                        █████████
                        █████████
                        ████▀████
                        ████ ████
                         ███ ███
                          ██ ██
                          ▀▀█▀▀

                          n i b

              Offline pastebin for your terminal.
           text in · searchable · stays on this disk

                 store   %APPDATA%\nib\nib.db

┌──────────────────────────────────────────────────────┐
│ › 1  Add a paste    type in your editor, then save   │
│   2  Find a paste   search the text, pick a hit      │
│   3  Recent pastes  the last ten, copy one           │
└──────────────────────────────────────────────────────┘

                  1-3 choose  ·  q quit
```

- **Mark:** the nib from `docs/design/logo.md` and `assets/nib-cli.svg`: 9×10 cells, steel ramp top to bottom, one ember cell at the point. Below it `n i b` in bold, letter-spaced. The mark is the logo; the wordmark names it.
- **Tip:** the tines stop half a cell above the point, so the ember block is the lowest thing on the screen.
- **`store` line** replaces the reference's `server` line: there is no server. It shows the resolved database path, nothing else.
- **Menu** is a launcher over existing verbs:
  1. `add` → `$EDITOR` on a temp file, save on close, print the ID.
  2. `search` → `search › ` prompt, five hits with snippets, `1-5` prints, `c` copies.
  3. recent → last ten, `1-9` prints, `c` copies.
- **Footer lists only keys that work.** `1 2 3` choose, `q` quits. No arrow keys, no redraw.
- No alternate screen. Print once, read one key, act, let output scroll normally.

## Colour

| Situation | Mark | Body |
|---|---|---|
| truecolor TTY | steel ramp, one colour per row; point `#e07a45` | `n i b` in `#e8e6e1` bold; `·`, `store`, footer in `#8f8d87` |
| 256-colour TTY | nearest colour per row, same order | same, nearest match |
| `NO_COLOR` set | plain, `n i b` bold | plain |
| terminal under 60×24 | `nib` in one line, no mark, no box | one usage line |

The mark is the only place the ramp appears; the point is the only warm pixel. The `›` and the menu numbers carry the same ember. Nothing else is coloured.

## Rules for implementation

- Hand-written ANSI strings; `crossterm` only for raw mode and a single key read.
- No `ratatui`, no alternate screen, no layout framework. The screen is about 20 lines of `println!`.
- Glyphs used: `█ ▀ ▄` (U+2588, U+2580, U+2584) for the mark, `›`, `·`, `┌ ─ ┐ │ └ ┘`. All are standard in Windows Terminal, ConPTY and common Linux/macOS terminals.
- Output of the chosen action follows the stream rules: paste to stdout, chrome to stderr.
