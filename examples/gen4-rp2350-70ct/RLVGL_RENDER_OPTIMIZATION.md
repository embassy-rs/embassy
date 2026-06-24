# rlvgl full-frame rendering optimization notes

## Problem

The `rlvgl_widget_demo` renders into a single persistent `800×480` RGB565
framebuffer in PSRAM. A real full-frame repaint therefore writes:

```text
800 × 480 × 2 bytes = 768 KiB
```

over the shared QMI/PSRAM bus while the PIO RGB scan-out continuously reads from
the same framebuffer. That bus traffic is the hard limit for full-screen redraws.

Partial rendering avoids this most of the time, but any path that still calls a
full `render_tree()` will visibly take longer than small widget updates.

## Current architecture

- One persistent PSRAM framebuffer.
- PIO RGB scan-out reads continuously from that framebuffer.
- Bounce-buffer refill copies through the cached CPU/XIP path, matching the
  working C reference default.
- `rlvgl` draws directly into the PSRAM framebuffer.
- Dynamic animation already redraws only the changing widgets.
- Full repaint is still used for coarse UI changes.

## Main optimization idea

Avoid full-screen repaint for normal touch/button interaction.

Instead of this coarse logic:

```rust
if touched {
    render_tree(fb, ui.root());
} else if animated {
    ui.render_dynamic(fb);
}
```

normal touch handling should mark only the affected widgets dirty and redraw
those widgets:

- button
- status label
- bar
- LED

`render_tree()` should remain reserved for boot, reset, theme/layout changes, or
other rare cases where the whole screen really changed.

## Suggested implementation order

1. Change `DemoUi::handle_touch()` so it returns what changed.
   - For example, return a small dirty bitmask instead of only mutating state.
   - Button interaction should mark only the button/status/bar/LED area dirty.
2. Replace touch-triggered full repaint with `DemoUi::render_dirty()`.
   - `render_dirty()` redraws only the widgets selected by the dirty mask.
   - `render_dynamic()` can either stay separate or become a special dirty-mask
     case for the animated widgets.
3. Keep full repaint rare.
   - Use `render_tree()` at startup.
   - Use it only for global visual changes such as a layout/theme reset.
4. If full repaint is still too slow, add renderer fast paths.
   - Optimize large solid fills and rectangle blits.
   - Prefer writing multiple RGB565 pixels per store where possible.
   - This can improve CPU/cache behavior, but it cannot remove the `768 KiB`
     PSRAM write cost of a real full-frame redraw.

## Why this is the lowest-risk next step

The display timing, DMA scan-out, bounce-buffer refill, bus priority, and
single-framebuffer anti-flicker path are already stable. Optimizing the UI dirty
logic does not touch the critical scan-out timing path and therefore has much
lower risk than another DMA/PIO/bus experiment.

The expected win is not making a true full-frame redraw magically cheap. The win
is making normal UI interaction stop causing full-frame redraws in the first
place.

## Later options

- Add a small dirty-rectangle layer for more precise clipping.
- Add larger bounce-buffer slack if scan-out ever becomes marginal again.
- Render complex full-screen scenes into SRAM-sized tiles, then copy the tile to
  PSRAM; this may improve locality, but still writes the same total framebuffer
  size.
- Reduce large-area invalidation in the UI design itself.