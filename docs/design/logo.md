# Nib — the mark

A fountain-pen nib drawn as pixels: steel body, a slit that splits it into two tines, and one ember block at the point. It is the CLI logo, the app icon and the README logo. One artwork, three renderings.

## Shape

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
```

- 9 columns × 10 rows of half-block cells, each cell two square pixels.
- Stepped round crown, straight body, then the slit opens and the tines step in to a point.
- No breather hole. The slit starts with a half-cell notch (`▀`), which at this size reads as the hole.
- The last row is `▀▀█▀▀`: the tines stop at mid-cell and the ember runs a full cell, so the point touches the slit above and hangs one pixel below the tines.

## Colour

The body runs a steel ramp top to bottom; the point is ember. Row colours, in order:

| Row | Cell | Colour |
|---|---|---|
| 1 | `  ▄███▄` | `#d7ecfe` |
| 2 | `▄███████▄` | `#bcdcf7` |
| 3 | `█████████` | `#a6c9ec` |
| 4 | `█████████` | `#93b7dd` |
| 5 | `█████████` | `#82a8ce` |
| 6 | `████▀████` | `#7199bf` |
| 7 | `████ ████` | `#628bb0` |
| 8 | ` ███ ███` | `#547da2` |
| 9 | `  ██ ██` | `#487094` |
| 10 | `  ▀▀█▀▀` | `#3d5e80`, point `#e07a45` |

One warm pixel in the whole mark: the point.

## Files

| File | Use |
|---|---|
| `assets/nib-cli.svg` | the mark at 9×20 units, dark surfaces only — the light top disappears on white |
| `assets/nib-appicon.svg` | 512×512 tile, `#141413` with `rx=112`: README hero, table icons, app icon |
| `assets/nib-mark.svg` | previous smooth outline, kept only if a single-colour mark is needed |
| `assets/nib-wordmark.svg` | stroke-drawn `nib`, for wide lockups |

The terminal rendering and both SVGs are generated from the same 9×20 pixel grid, so they match block for block. The SVG blocks carry a 1-unit seam so the pixel grid stays visible.

## Lockup

Mark centred, `n i b` in bold one row below it, one cell of tracking, `#e8e6e1`, then the tagline in muted ink. The mark carries the weight; the wordmark only names it.

## Rules

- The ramp is the mark. Do not flatten it to one colour, and do not add a second warm colour.
- On a light surface use the tile, never `nib-cli.svg`.
- Below 60×24 the splash drops the mark and prints `nib` alone.
- `NO_COLOR` keeps the shape; the point is geometry, not colour.

Implemented in `crates/nib-cli/src/splash.rs` (terminal) and `assets/*.svg` (everywhere else).
