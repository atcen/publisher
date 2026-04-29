# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Publisher is an open-source, print-first desktop publishing application — an InDesign alternative. It runs as a native desktop app (Tauri) and in the browser (WASM + WebGPU), with real-time collaboration via CRDT.

Full concept documentation: https://github.com/atcen/publisher/wiki

## Architecture

```
publisher/
├── crates/
│   ├── core/           # Document model, layout engine (no UI, no platform code)
│   ├── typography/     # HarfBuzz + FreeType + Parley integration
│   ├── color/          # LittleCMS, CMYK/RGB/ICC, spot colors
│   ├── renderer/       # Vello abstraction (native GPU + WebGPU)
│   ├── export-pdf/     # PDF/X-1a, PDF/X-4, interactive PDF
│   ├── export-epub/    # ePub Fixed Layout + Reflowable
│   ├── collab/         # automerge-rs CRDT, document sync
│   └── wasm/           # WASM bindings for browser build
├── apps/
│   ├── desktop/        # Tauri app shell
│   └── web/            # Svelte frontend (shared between desktop and browser)
└── backend/            # Axum server (auth, S3 storage, collaboration WebSocket)
```

**Key constraint:** `crates/core` must have zero platform dependencies — no filesystem, no GPU, no UI. It must compile to both native and WASM targets unchanged.

**Document model:** The authoritative spec is in the [Document Model wiki page](https://github.com/atcen/publisher/wiki/Document-Model). All coordinates are stored in points (pt) internally; units conversion happens at the UI boundary.

**Rendering:** Canvas rendering uses Vello (not the web UI framework). Svelte handles panels, dialogs, and toolbars only. Never put layout logic in Svelte.

**Collaboration:** The document model is CRDT-backed via automerge-rs from day one. All document mutations must go through the CRDT layer, never directly mutate structs.

## Development Principles

- **TDD:** Write acceptance tests before implementing a feature. Every issue on GitHub includes acceptance criteria that map directly to tests.
- **Atomic PRs:** One issue = one PR. No bundling unrelated changes.
- **No feature gates:** The AGPL v3 open source build must be fully featured. No `#[cfg(feature = "pro")]` on core functionality.

## License

AGPL v3 (open source) + Commercial License (dual licensing). See [Business Model wiki](https://github.com/atcen/publisher/wiki/Business-Model).
