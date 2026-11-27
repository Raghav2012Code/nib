# Nib — the terminal app

`nib` with no arguments opens a full-screen application. Piped input still goes straight to the store, so scripts never see it.

## Screens

| Screen | Reached by | Keys |
|---|---|---|
| Home | `nib` | `↑↓ j k` move · `↵` open · `n` new · `/` find · `q` quit |
| Add | `n` or "Add a paste" | type · `Ctrl+S` save · `Esc` cancel |
| Find | `/` or "Find a paste" | type · `↑↓` move · `↵` open · `Esc` back |
| Recent | menu | `↑↓` move · `↵` open · `Esc` back |
| Preview | `↵` on any list | `↵` copy · `↑↓` scroll · `u` unlock · `Esc` back |

`Ctrl+C` quits from anywhere. `q` only quits from Home, so it stays a letter inside a text field.

## Layout

Six bands, top to bottom: header, rule, body, rule, footer, status.

**Scaling.** The interface never grows wider than its column, but the column steps up with the window — 90 columns below 120, 100 up to 169, 120 from 170 — and the mark doubles when the window is at least 30 rows tall and the column is at least 100 wide. A fullscreen terminal gets a bigger hero and a wider frame, not a postcard in a void, and below 120 columns nothing changes from the layout that reads well at 80×24. Home centres its content vertically when there is spare height; Add, Find, Recent and Preview stay top-aligned. The recent list is narrower than the column and centred in it, so ages sit near their names instead of at the far edge.

Home opens on the mark, centred, with one line under it. The mark and its line belong to Home only: opening Add, Find, Recent or Preview replaces them with that screen, so the focused view has nothing above it but the header.

```
 nib                                  C:\Users\you\AppData\Roaming\nib\nib.db
────────────────────────────────────────────────────────────────────────────
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

                     Offline pastebin for your terminal.

                 › Add a paste     write it, Ctrl+S saves
                   Find a paste    search the text, pick a hit
                   Recent pastes   the last ten, open one

  recent · 2
   k3x9  fn parse_args() -> Args {                                    2m
   r8ce  deploy key                                                   1d

────────────────────────────────────────────────────────────────────────────
 ↑↓ move · ↵ open · n new · / find · q quit
  saved k3x9
```

- The header shows the resolved store path, shortened from the left, never cut in the middle.
- The footer names only the keys that work on that screen.
- The status line carries one transient message — `saved k3x9`, `copied k3x9`, `unlocked`, or an error — and clears itself.
- Below 60×24 the app draws one centred line naming the current and required size, and nothing else.

## Add

A full screen with the cursor in a real editor: wrapping, selection, undo, bracketed paste. `Ctrl+S` saves through the same store call the CLI verbs use, returns to Home, refreshes the list and shows `saved <id>`. `Esc` on a draft asks once; `Esc` on an empty draft just goes back. The status line shows the character count and the 64 KB ceiling while you type.

## Find

The query is a visible, editable field. Results update as you type, each with its id and the matched snippet, the match in the ember accent. An out-of-order response can never replace a newer query — each request carries a sequence number and stale answers are dropped.

## Preview

The paste fills the width and wraps. Enter copies it; the footer confirms with `copied <id>`. Long pastes scroll with `↑↓` and `PgUp`/`PgDn`.

A sealed paste never shows ciphertext. It shows a passphrase field; a wrong passphrase is an inline error that keeps the field, and the right one reveals the text in place. `Esc` leaves the field first, then the screen.

## Colour

- Accent: `#e07a45` on dark terminals, `#b4531f` on light ones, chosen from `COLORFGBG`'s background index.
- Muted text is the terminal's grey 245; the error colour is `#f97066`.
- `NO_COLOR` turns every colour off; the selection stays visible because the marker and bold carry it, not the colour.

## Why a layout engine

The previous splash computed padding by hand. Two menu rows collided — `Add a paste paste, Ctrl+Z to finish`, `Find a pastesearch the text` — and a border drifted off its row. Layout is now calculated by ratatui, and the tests assert that label and description are separated at the minimum width. The splash module is deleted.
