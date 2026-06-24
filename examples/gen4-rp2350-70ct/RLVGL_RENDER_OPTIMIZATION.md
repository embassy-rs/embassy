# rlvgl full-frame rendering optimization notes

## Problem

The `rlvgl_widget_demo` renders into a single persistent `800×480` RGB565
framebuffer in PSRAM. A real full-frame repaint therefore writes:

```text
800 × 480 × 2 bytes = 768 KiB
```

over the shared QMI/PSRAM bus while the PIO RGB scan-out continuously reads from
the same framebuffer. That bus traffic is the hard limit for full-screen redraws.

Partial rendering avoids this most of the time. Any future path that reintroduces
a full `render_tree()` during normal interaction will still visibly take longer
than small widget updates.

## Current architecture

- One persistent PSRAM framebuffer.
- PIO RGB scan-out reads continuously from that framebuffer.
- Bounce-buffer refill copies through the cached CPU/XIP path, matching the
  working C reference default.
- Dirty-widget updates draw directly into the PSRAM framebuffer.
- Full-screen redraws render through a `39`-line SRAM tile and then copy each
  tile linearly into PSRAM.
- Dynamic animation redraws only the changing widgets (`bar` + `LED`).
- Touch/button interaction no longer repaints the whole tree. It uses a small
  dirty-widget mask and redraws only `button`, `status`, `bar`, and `LED`.
- Pointer-move samples are dispatched to the widget tree but do not trigger a
  redraw unless press/release state actually changes.
- Full repaint is reserved for startup and future global layout/theme changes.
- The demo logs render timings so regressions are visible on hardware.

## Implemented optimizations

### Dirty-widget rendering

Normal touch/button interaction now avoids full-screen repaint.

The previous coarse logic was:

```rust
if touched {
    render_tree(fb, ui.root());
} else if animated {
    ui.render_dynamic(fb);
}
```

The current logic is:

```rust
if dirty != DirtyWidgets::None {
    ui.render_dirty(fb, dirty);
} else if animated {
    ui.render_dynamic(fb);
}
```

`DemoUi::handle_touch()` returns a `DirtyWidgets` value for press/release changes,
and `DemoUi::render_dirty()` redraws only the affected widgets:

- button
- status label
- bar
- LED

`render_tree()` remains reserved for boot, reset, theme/layout changes, or other
rare cases where the whole screen really changed.

### SRAM-tiled full-frame rendering

Full-screen rendering now uses `render_tree_tiled()` instead of drawing the whole
widget tree directly into PSRAM. It renders `39` scan lines at a time into an
SRAM tile:

```text
800 × 39 × 2 bytes = 62.4 KiB
```

After each tile is drawn, the tile is copied linearly to the corresponding PSRAM
framebuffer rows. This does **not** remove the unavoidable `768 KiB` full-frame
write, but it keeps the complex widget drawing phase off the shared QMI/PSRAM
bus and turns the final framebuffer update into wide sequential writes.

## Hardware measurements

Observed on the gen4-RP2350-70CT board:

- Initial direct full `render_tree()` into PSRAM before tiling: about `468 ms`.
- Initial tiled full `render_tree_tiled()` after the stack-allocation fix: about
  `533 ms`.
- Dirty widget redraw after touch: about `39–49 ms`.

The first tiled measurement is slightly slower than direct PSRAM rendering for
this simple scene, because the widget tree is redrawn once per tile and the demo
still writes the full framebuffer afterwards. The tiled path is therefore mainly
an experimental baseline; dirty-widget rendering remains the important win for
normal interaction.

The dirty redraw is still not free because the selected widgets include larger
filled rectangles and text, but it is an order of magnitude cheaper than a full
frame rewrite and avoids the normal-interaction fullscreen path.

## Why this is the lowest-risk next step

The display timing, DMA scan-out, bounce-buffer refill, bus priority, and
single-framebuffer anti-flicker path are already stable. The dirty-widget and
SRAM-tile changes do not touch the critical scan-out timing path and therefore
have much lower risk than another DMA/PIO/bus experiment.

The win is not making a true full-frame redraw magically cheap. The win is that
normal UI interaction no longer causes full-frame redraws in the first place.

## Remaining options

- Add renderer fast paths for large solid fills and rectangle blits.
  - Prefer writing multiple RGB565 pixels per store where possible.
  - This can improve CPU/cache behavior, but it cannot remove the `768 KiB`
    PSRAM write cost of a real full-frame redraw.
- Add a small dirty-rectangle layer for more precise clipping.
- Add larger bounce-buffer slack if scan-out ever becomes marginal again.
- Reduce large-area invalidation in the UI design itself.