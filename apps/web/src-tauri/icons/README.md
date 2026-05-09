# Tauri icons

`tauri.conf.json` references icons in this directory by these names:

- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

To generate them all from a single source PNG (1024×1024 recommended):

```sh
cd apps/web
npm run tauri icon path/to/source-icon.png
```

Tauri's CLI fans the source out into every required platform-specific size and format
in this folder. `tauri dev` will run without these icons; `tauri build` requires them.

A starter source `source.png` should be a 1024×1024 PNG of the giff logo on its red
background. You can render one from `apps/web/static/logo.svg` with any SVG-to-PNG tool
(e.g. `rsvg-convert apps/web/static/logo.svg -w 1024 -h 1024 -o source.png`).
