# Nib — CLI mark

A fountain-pen nib drawn directly in half-block cells: 7 columns × 8 rows, each cell holding two square pixels (`▀ ▄ █`). The strings below are the mark; nothing is merged at runtime.

## Mark

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

- The crown is a half-block dome; the body runs full width for three rows, then the tines step in.
- The last two rows are the point. The tines end in a `▄` gap, and one `▀` sits centred in the row below it: a single ember pixel that sticks out half a cell past the tines, so the nib comes to a real point instead of a flat end.
- No breather hole. At this width it reads as a dent; the slit carries the shape.

## Size

7 columns × 8 rows: narrower than the first draft, and the only cell that hangs below the body is the point itself.

## Colour

| Row | Colour | Role |
|---|---|---|
| 0 | `#d7ecfe` | crown |
| 1 | `#bcdcf7` | body |
| 2 | `#a2c6ea` | body |
| 3 | `#8fb3d9` | body |
| 4 | `#7aa1c8` | tines |
| 5 | `#6a8fb5` | tines |
| 6 | `#4f7299` tines, `#e07a45` gap | tines + point |
| 7 | `#e07a45` | point, protruding |

Six cells of steel, then ember at the writing point. The protruding cell is the sharpest pixel in the mark, which is why it is the one that is orange.

## Lockup

Mark centred, `n i b` in bold one row below the point, one cell of tracking, `#e8e6e1`, then the tagline in muted ink. The mark carries the weight; the wordmark only names it.

## Assets

The terminal mark and the SVG marks are the same silhouette:

| File | Use |
|---|---|
| `assets/nib-mark.svg` | currentColor mark for the app window, docs, README |
| `assets/nib-appicon.svg` | 512×512 app icon, ember on `#141413` |
| `assets/nib-wordmark.svg` | stroke-drawn `nib`, for wide lockups |

## Rules

- Never recolour a row wholesale; the ramp is the mark, and the last two rows are two colours by design.
- Below 60×24 the mark is dropped and the splash prints `nib` alone.
- `NO_COLOR`: plain half-blocks, no colour; the point still reads because it is geometry, not colour.
- Desktop app uses `assets/nib-mark.svg`, the same silhouette with smooth curves, never this grid.
