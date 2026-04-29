# Gemini Context: Publisher

## Project Overview
Publisher is a modern, open-source, print-first desktop publishing application—a professional alternative to Adobe InDesign. It is designed to run as a native desktop app (Tauri) and in the browser (WASM + WebGPU), featuring real-time collaboration via CRDTs.

**Key Technologies:**
- **Core Engine:** Rust (compiles to native and WASM)
- **2D Rendering:** Vello + WebGPU/WGPU
- **Desktop Shell:** Tauri
- **Browser Build:** WASM + WebGPU
- **UI Layer:** Svelte (for panels and dialogs; canvas is Vello-rendered)
- **Typography:** HarfBuzz + FreeType + Parley
- **Color Management:** LittleCMS
- **Collaboration:** `automerge-rs` (CRDT)
- **Backend:** Rust (Axum) + S3-compatible storage

## Architecture
The project is organized as a Rust workspace with the following structure:
- `crates/core/`: Document model and layout engine (zero platform dependencies).
- `crates/typography/`: Text shaping and font handling.
- `crates/color/`: Color space management and ICC profiles.
- `crates/renderer/`: Vello-based rendering abstraction.
- `crates/export-pdf/`: PDF/X-1a and PDF/X-4 export logic.
- `crates/export-epub/`: ePub export logic.
- `crates/collab/`: CRDT and synchronization logic.
- `apps/desktop/`: Tauri application shell.
- `apps/web/`: Svelte frontend (shared between desktop and web).
- `backend/`: Axum-based collaboration and storage server.

## Building and Running
The project is organized as a Rust workspace.

- **Build all crates:** `cargo build --workspace`
- **Run tests:** `cargo test --workspace`
- **Run backend:** `cargo run -p publisher-backend`

## Development Conventions
- **TDD (Test-Driven Development):** Every feature must have acceptance tests defined before implementation.
- **Atomic Workflows:** One issue = one concern, one PR = one issue.
- **Zero Platform Dependencies in Core:** `crates/core` must compile to both native and WASM targets without changes.
- **CRDT-First:** All document mutations must go through the `automerge-rs` layer.
- **No Feature Gates:** The open-source version must remain fully featured.
- **Licensing:** Dual-licensed under AGPL v3 and a Commercial License.

## Key Resources
- **Full Wiki:** `wiki_repo/` (local clone) or [online](https://github.com/atcen/publisher/wiki)
- **Document Model Spec:** `wiki_repo/Document-Model.md`
- **Architecture Decisions:** `wiki_repo/Architecture-Decisions.md`
