# Issue #41: Text Frame Types (Point vs. Area Text)

## Description
Currently, Publisher only supports fixed-size frames (`Frame`). In professional DTP tools like Illustrator, there is a distinction between:
- **Point Text**: The frame automatically expands or contracts to fit the text content. Scaling the frame scales the text (adjusting font size).
- **Area Text**: The text flows within a fixed-size frame and wraps when it hits the boundaries. Scaling the frame changes the flow/wrapping, not the text size.

We need to implement this distinction to allow for more flexible typography layouts (e.g., headlines that auto-expand vs. body copy that flows).

## Proposed Changes
1. **Core (`crates/core`)**:
   - Update `TextFrame` to include a `TextFrameType` enum (Point, Area).
   - Implement logic in the layout engine to calculate the bounding box for "Point" text.
   - Add a method to `TextFrame` to scale font size to a target width (direct calculation: $S_{new} = S_{old} * W_{target} / W_{current}$).

2. **UI (`apps/web`)**:
   - Add a toggle in the "Properties" panel to switch between Point and Area text.
   - Update the frame resizing handles to behave differently based on the type (Point text scales font size, Area text changes frame bounds).

## Acceptance
- [ ] `TextFrame` struct has a `frame_type` property.
- [ ] "Point Text" frames automatically size themselves to fit their content.
- [ ] Scaling a "Point Text" frame in the UI updates the font size of the content.
- [ ] "Area Text" continues to wrap text within fixed bounds.
- [ ] Switching from "Area" to "Point" text preserves the current text but recalculates the frame size.
- [ ] Existing tests pass and new tests for "Point Text" are added.
