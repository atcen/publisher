## Overview
This PR delivers the core 'v0.1 Print Foundation', significantly extending the document model, typography, and color systems while introducing professional UI workflows.

### Fixed Issues & Implementation Details

#### Reliability & Platform
- **Auto-save and crash recovery (Fixes #43)**
    - Background auto-save every 60s (configurable).
    - Recovery detection on startup with user prompt.
    - Platform-appropriate storage paths (~/Library/Application Support/... or AppData).
- **Preferences dialog (Fixes #53)**
    - New 'Einstellungen' modal (Theme, Units, Intervals).
    - Immediate dynamic updates without restart.
    - Persistent disk storage.
- **Recent files (Fixes #42)**
    - Tracking of last 10 documents.
    - Submenu integration in 'Datei' menu.

#### Layout & Productivity
- **Parent Pages (Fixes #6)**
    - Recursive hierarchy rendering (Parent based on Parent).
    - Interactive object overrides (click parent frame to clone/edit locally).
- **Frame/Content Independence (Fixes #9)**
    - Independent transforms for frames vs. images.
    - Professional cropping/scaling modes.
- **Magnetic Snapping (Fixes #39)**
    - Rust-based SnapEngine for high performance.
    - Snapping to Margins, Columns, Guides, and other Objects.
    - Grid Presets (12-column, Golden Ratio, etc.).
- **GREP Find & Replace (Fixes #49)**
    - Full Regex support for document-wide search.
    - 'Replace All' and 'Find Next/Prev' functionality.
- **Paste in Place (Fixes #45)**
    - Preserved coordinates during copy-paste.
    - 'Text-only' paste support.

#### Typography & Color
- **Variable Fonts & Kerning (Fixes #14, #15)**
    - OpenType variation axes support (Weight, Width, etc.).
    - Metric and algorithmic Optical kerning modes.
- **Professional Color (Fixes #17, #18)**
    - CMYK, RGB, and named Spot Colors with tints.
    - LittleCMS (lcms2) integration for bidirectional ICC conversion.

### Structural Refactoring
- **Frontend Decoupling:** Replaced the 'God Component' App.svelte with modular Svelte 5 Stores (docStore, uiStore, prefsStore) and extracted UI components.
- **Logic Push-down:** Moved heavy computations (snapping, alignment, distribution) to Rust (crates/core).

### Identified Gaps & Follow-up Issues
During implementation, several areas were identified for future refinement:
- **Style Editor UI:** Currently uses placeholders. Created #88.
- **ICC Profile Management:** Need a UI to manage .icc files. Created #89.
- **CRDT Migration:** Frontend should move from JSON snapshots to incremental Automerge patches. Created #90.
- **Global Unit Binding:** Need a generic 'UnitInput' component for consistent conversion. Created #91.
