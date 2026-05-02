# PR: Spot Color Support (#18)

## Description
This PR implements full-stack support for Spot Colors (Volltonfarben). It enables professional print workflows by allowing named, device-independent colors with CMYK alternates and tinting.

## Changes

### Core & Color Engine (`crates/color`, `crates/core`)
- **Spot Color Conversion:** Updated `ColorEngine::convert_core_color` to support `Color::Spot`. 
    - Uses `alternate_cmyk` and `tint` to calculate an RGB preview.
    - Implemented a naive mathematical fallback for CMYK->RGB conversion to ensure previews work even without loaded ICC profiles.
- **TDD:** Added unit tests in `crates/color` to verify accurate spot-to-rgb conversion (including tinting).

### Desktop App (`apps/desktop`)
- **Tauri Command:** Updated the existing `convert_color` command to support all core color types (RGB, CMYK, Spot), allowing the frontend to request accurate color conversions from the Rust core.

### Web UI (`apps/web`)
- **Swatch Management:**
    - Created `SwatchModal.svelte` for editing swatch details (Name, Color Type, Values).
    - Integrated the new modal into `ModalManager.svelte` and `App.svelte`.
- **UI Enhancements:**
    - Added a visual indicator (orange dot) for Spot colors in the swatches panel.
    - Improved `getSwatchColor` in the sidebar to handle `Spot` and `Cmyk` types with proper tinting for the UI preview.

## Verification Results

### Automated Tests
- `cargo test -p publisher-color`: **PASSED**
- `npm --prefix apps/web run check`: **PASSED**

### Manual Verification
- Created a new Spot color "PANTONE 185 C".
- Adjusted the tint and verified the UI color update.
- Switched between RGB, CMYK, and Spot modes.
- Verified that the "orange dot" indicator appears correctly.

## Acceptance Criteria Check
- [x] `Color::Spot` is handled by the ColorEngine.
- [x] CMYK alternate and Tint are used for preview.
- [x] Swatch editing UI supports Spot color fields.
- [x] Spot colors have a visual indicator in the UI.
