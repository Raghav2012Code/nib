## Problem Statement

`nib` with no arguments draws a poster and then behaves like a script.

Concretely, from the 0.1.2 splash:

- The menu rows collide. `Add a paste paste, Ctrl+Z to finish` repeats the label's own words, `Find a pastesearch the text` has no separator at all, and the right border drifts off its row, leaving a stray `│` floating inside the box.
- The menu cannot be scrolled. Only the number keys select anything, and there is no cursor on screen to move.
- Picking "Add a paste" does not open a screen. The banner stays, the terminal echoes what you type underneath it, and the only way to finish is Ctrl+Z — after which the id is printed into the wreckage of the banner.
- There is no way back. The screen never changes, so there is nothing to return to.

The result is a poster with three hotkeys, not an application, and it cannot be shipped as one.

## Solution

Replace the splash with a full-screen terminal application built on a layout engine.

`nib` opens **Home**: header, a menu you move through with ↑/↓ or `j`/`k`, a recent-pastes panel, and a footer that names only the keys that work there. **Enter** opens the selected screen. **Add** is a real screen with the cursor in an editor and nothing else competing for keys; **Ctrl+S** saves, returns to Home, refreshes the list and shows the new id. **Esc** always goes back. **Find** is a search field with live results. **Preview** shows the paste and copies it with Enter. Sealed pastes unlock in place, with the passphrase field inside the preview.

Layout is computed by the layout engine, so no padded string can collide, and no row can overflow its frame. The identity is unchanged: pixel nib, steel and ember, monospace, hairlines.

```
┌ Home ────────────────────────────────────────────────────────────┐
│ nib                                        ~\AppData\Roaming\…\nib.db │
│                                                                  │
│ › Add a paste      write it, Ctrl+S saves                        │
│   Find a paste     search the text, pick a hit                   │
│   Recent pastes    the last ten, open one                        │
│                                                                  │
│ ───────────────────────────────────────────────────────────────  │
│ k3x9   2m    fn parse_args() -> Args {                           │
│ m7qa   47m   kubectl get pods -A --field-selector …               │
│                                                                  │
│ ↑↓ move · ↵ open · n new · / find · q quit          saved k3x9   │
└──────────────────────────────────────────────────────────────────┘
```

## User Stories

1. As a terminal user, I want `nib` alone to open a full-screen application, so that I am inside the tool instead of looking at a banner.
2. As a terminal user, I want a visible selection I can move with ↑/↓, so that I do not have to memorise number keys.
3. As a terminal user, I want `j`/`k` to move the selection too, so that my hands stay on the home row.
4. As a terminal user, I want Enter to open the highlighted item, so that what is marked is what I get.
5. As a terminal user, I want Esc to return to the screen I came from, so that I can never get lost.
6. As a terminal user, I want Home to list recent pastes with id, first line and age, so that I recognise what I stored.
7. As a terminal user, I want the store path visible on Home, so that I know which database I am writing to.
8. As a writer, I want "Add a paste" to open a focused editor, so that every keystroke goes into the paste.
9. As a writer, I want arrows, Home/End and selection inside that editor, so that it behaves like an editor rather than a raw read.
10. As a writer, I want Ctrl+S to save, return me to Home and show the new id, so that the loop closes in one keystroke.
11. As a writer, I want Esc on a non-empty draft to ask once, so that a stray key cannot destroy text.
12. As a writer, I want the character count and the 64 KB ceiling while I type, so that save time is not a surprise.
13. As a searcher, I want Find to open an editable search field, so that I can see and change my query.
14. As a searcher, I want results as I type, with the matched snippet, so that I can pick without leaving the screen.
15. As a searcher, I want ↑/↓ through results and Enter to preview, so that search and reading are one flow.
16. As a reader, I want a full-width preview that wraps, so that long lines never run off.
17. As a reader, I want Enter on a preview to copy it, so that getting the text out is one key.
18. As a reader, I want a visible copy confirmation, so that I know it happened.
19. As a reader, I want sealed pastes to ask for the passphrase inside the preview, so that unlocking never dumps me to a shell.
20. As a reader, I want a wrong passphrase to show an inline error and keep focus, so that retrying is immediate.
21. As a user, I want each footer to list only the keys that work on that screen, so that no key is ever advertised and dead.
22. As a user, I want `q` to quit only from Home, so that `q` in a field stays a letter.
23. As a user, I want the terminal restored on quit or crash, so that my cursor and scrollback are intact.
24. As a user, I want a clear message below 60×24, so that I know why nothing drew.
25. As a user, I want the ember accent and monospace type kept, so that the terminal matches the window and the docs.
26. As a user without colour, I want `NO_COLOR` respected, so that the app works in monochrome.
27. As a scripter, I want `echo x | nib` to keep storing silently, so that pipelines never become interactive.
28. As a user, I want the menu label and its description separated by at least two spaces on every row, so that no two words can touch.
29. As a user, I want no line to exceed the frame, so that borders never drift.

## Implementation Decisions

- **Layout engine.** Adopt `ratatui` on the existing `crossterm`. Every screen is a layout; no string padding and no manual border arithmetic. This deletes the class of bug behind the collisions, not just the two visible ones.
- **Editor.** `tui-textarea` for the compose screen: cursor, wrapping, selection, undo and paste. Hand-rolling a second editor is not worth owning.
- **Pure state machine.** A module owns the state as plain data — screen, menu index, draft, query, hits, preview, passphrase, toast, back screen — and consumes key events, returning effects: `Save`, `Search`, `Copy`, `Reveal`, `Quit`. It never touches a terminal or the store, so it is testable without either.
- **Screens.** `Home`, `Add`, `Search`, `Recent`, `Preview`. Add is a screen, not an overlay. Preview is reached from Home, Recent or Search and remembers where it came from.
- **Event loop.** Poll with a short timeout so toasts expire and relative ages refresh without a keypress; redraw on change or tick.
- **Terminal lifecycle.** Alternate screen and raw mode on entry, restored on drop, and a panic hook that restores before printing, so a crash never leaves a broken terminal.
- **Keymap.** One table per screen inside the state machine. Global: `Esc` back, `Ctrl+C` quit. Home: `↑↓ j k`, `Enter`, `n`, `/`, `q`. Add: typing, `Ctrl+S`, `Esc`. Search: typing, `↑↓`, `Enter`, `Esc`. Recent: `↑↓`, `Enter`, `Esc`. Preview: `Enter` copy, `u` unlock when sealed, `Esc` back.
- **Size guard.** Below 60×24, draw one centred message with current and required size; no partial UI.
- **Store access.** Unchanged. The state machine asks `nib-core` for list, search, add and reveal through the calls the verbs already use. No schema change.
- **Entry point.** `nib` with a terminal opens the app; `nib` with piped stdin still stores and prints an id. Verbs are untouched.
- **Retirement.** The splash module is deleted. Its mark, palette and copy move into Home.
- **Home layout.** Header row, menu block, hairline, recent list, footer. Menu rows are a three-column table (marker, label, description), which is what guarantees separation and truncation.

## Testing Decisions

- **Test behaviour, not bytes.** A good test sends a key event and asserts the next screen, the selected index, the draft, or the effect. It never asserts ANSI.
- **State machine tests:** arrow and `j`/`k` clamping; Enter opening the right screen; Esc returning to the correct screen; typing reaching the editor; `Ctrl+S` emitting `Save` with the exact body; empty save refused with a status; Esc on a draft asking once, `y` discarding, `n` staying; search emitting one effect per keystroke with an increasing sequence; `q` quitting only from Home and staying a letter in the editor.
- **Render tests** with ratatui's test backend, asserting on cells: the selected row carries the marker; label and description are separated by at least two spaces; no line exceeds the area; the footer matches the screen's keymap; 59×23 draws only the size message.
- **Seams.** Two: the existing `nib-core` API (already covered) and the new pure state machine (all new tests). Render tests drive the same state machine through the test backend, so they add no third seam.
- **Prior art.** `crates/nib-cli/tests` uses `assert_cmd` for the non-interactive verbs; those stay and the TUI adds no terminal-dependent test.
- **Manual checklist** for the one thing tests cannot see: run in Windows Terminal and conhost at 80×25 and 140×40; quit with `q`, interrupt with Ctrl+C; confirm the cursor returns.

## Definition of Done

- `nib` opens Home; ↑/↓, `j`/`k`, Enter, `n`, `/`, `q` all behave as specified.
- Add → Ctrl+S → Home with the new id visible and the paste at the top of the list. Nothing echoes under the banner; there is no Ctrl+Z anywhere.
- No menu row collides, and no line exceeds the frame at any size at or above 60×24.
- `echo x | nib` still stores and prints an id with no interface.
- `cargo test` covers the transitions above; `cargo clippy -- -D warnings` is clean.
- The splash module is gone from the tree.

## Out of Scope

- Mouse support.
- Config files, themes, user keybindings.
- Tags, titles, folders, history, syntax highlighting.
- The desktop window, the installer and the store schema.
- Search ranking changes.

## Further Notes

The trigger is concrete: the splash computed padding by hand (`{label:<12}`, `inner - plain.len()`), so two rows collided and a border drifted. A layout engine makes that impossible rather than fixing it once.

The copy needs the same discipline: a description must not begin with the words its label already used. "Add a paste paste, Ctrl+Z to finish" is what that looks like.
