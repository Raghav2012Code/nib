# Nib — CLI mark

The CLI logo is the same nib as the app icon: a rounded bulb, a breather hole, a slit down to a sharp point, one flat ember. Terminal version is half-block cells; SVG version is `assets/nib-cli.svg`.

## Mark

```
  ▄███▄
▄███████▄
█████████
████ ████
████ ████
▀███ ███▀
 ███ ███
  ██ ██
  ▀▀ ▀▀
    █
```

9 columns × 10 rows, each cell two square pixels (`▀ ▄ █`). The gap from row 4 down is the slit; the first gap cell is the breather hole. The last row is the point, one cell wide, sitting below the tines.

## Colour

One colour: `#e07a45` on `#141413`. No ramp, no gradient.

The earlier steel ramp is gone. The icon is flat ember and the terminal mark now matches it, so the two are the same logo rather than two takes on it. The dark background does the shading.

| Situation | Mark | Body |
|---|---|---|
| truecolor TTY | `#e07a45` | `·`, `store`, footer in `#8f8d87` |
| 256-colour TTY | nearest to `#e07a45` | same, nearest match |
| `NO_COLOR` set | plain `█ ▀ ▄`, no colour | plain |
| terminal under 60×24 | `nib` in one line, no mark | one usage line |

## Files

| File | Use |
|---|---|
| `assets/nib-cli.svg` | the CLI mark, ember, transparent background |
| `assets/nib-mark.svg` | currentColor mark, inherits text colour |
| `assets/nib-appicon.svg` | 512×512 app icon, ember on a `#141413` tile |
| `assets/nib-wordmark.svg` | stroke-drawn `nib`, for wide lockups |

`nib-cli.svg`, `nib-mark.svg` and `nib-appicon.svg` share one outline; only the fill and the tile differ.

## Lockup

Mark centred, `n i b` in bold one row below it, one cell of tracking, `#e8e6e1`, then the tagline in muted ink. The mark carries the weight; the wordmark only names it.

## Rules

- One colour. Do not reintroduce a ramp, an outline or a highlight.
- Below 60×24 the mark is dropped and the splash prints `nib` alone.
- `NO_COLOR` keeps the shape; the point is geometry, not colour.
- The desktop app uses the SVG, never this grid.
