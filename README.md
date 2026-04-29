# Publisher

A modern, open-source page layout application — a professional InDesign alternative for print designers, publishers, and editorial teams.

**Status:** Early concept phase. See the [Wiki](https://github.com/atcen/publisher/wiki) for full documentation.

## What is Publisher?

Publisher is a desktop publishing tool focused on **print-first page layout**. It runs as a native desktop app and in the browser without installation — with real-time collaboration built in.

It is not an all-in-one graphics suite. It works alongside tools like Inkscape (vector) and GIMP (image editing), not instead of them.

## Key Features (planned)

- Print-first layout: DIN paper formats, CMYK, ICC profiles, PDF/X-1a export
- Cross-platform: macOS, Windows, Linux — and in the browser with no install
- Real-time collaboration (Figma-style, CRDT-based)
- Professional typography: OpenType, Variable Fonts, GREP styles, paragraph/character styles
- Master pages, text threading, footnotes, dynamic table of contents
- ePub Fixed Layout and Reflowable export
- IDML import/export for InDesign interop
- Plugin ecosystem

## Stack

| Layer | Technology |
|---|---|
| Core engine | Rust |
| 2D rendering | Vello + WebGPU |
| Desktop shell | Tauri |
| Browser | WASM + WebGPU |
| UI | Svelte |
| Typography | HarfBuzz + FreeType |
| Color | LittleCMS |
| Collaboration | automerge-rs (CRDT) |

## Roadmap

| Milestone | Goal |
|---|---|
| v0.1 | Desktop app, print layout, PDF/X-1a export |
| v0.2 | Browser app, real-time collaboration |
| v0.3 | Interactive PDF, accessible PDF, IDML |
| v0.4 | ePub Fixed Layout, RTL, plugins |
| v0.5 | ePub Reflowable |

Full details: [Roadmap Wiki](https://github.com/atcen/publisher/wiki/Roadmap)

## License

AGPL v3 — see [LICENSE](LICENSE). A commercial license is available for organizations that cannot comply with AGPL terms.

The software is fully free. No feature gates, no limits on the open source version. Revenue comes from the hosted cloud service. See [Business Model](https://github.com/atcen/publisher/wiki/Business-Model).

## Contributing

Issues and PRs follow an atomic workflow: one issue = one concern, one PR = one issue. Every feature issue includes acceptance criteria that map directly to tests (TDD).

See open issues: [github.com/atcen/publisher/issues](https://github.com/atcen/publisher/issues)
