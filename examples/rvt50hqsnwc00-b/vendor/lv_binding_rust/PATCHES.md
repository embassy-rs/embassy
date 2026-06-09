# Vendored `lv_binding_rust` master

Snapshot of [`lvgl/lv_binding_rust`](https://github.com/lvgl/lv_binding_rust)
at commit `d83b374` (master HEAD as of 2025-12) with the following
modifications:

## Pruned upstream content

- `examples/`, `lvgl-sys/vendor/lvgl/{demos,docs,examples,scripts,env_support,tests}`,
  `lvgl-sys/vendor/lv_drivers/` removed to keep the vendored tree small —
  none of them are needed to compile this Embassy example, and they would
  otherwise add ~80 MB to the repo.

## Source patches

- `lvgl/src/lv_core/style.rs` — replace `self.raw.clone().into_raw()` (method
  syntax that newer rustc rejects, since `Box::into_raw` is an associated
  function only) with `Box::into_raw(self.raw.clone())`. Should be sent
  upstream.

The lvgl C source under `lvgl-sys/vendor/lvgl/src/` is unchanged from
upstream LVGL 8 (commit `2b56e042`).
