// SPDX-License-Identifier: MIT OR Apache-2.0
//! Resource diagnostics for widget-heavy UIs.
//!
//! Two complementary signals:
//!
//! 1. [`census`](crate::diag::census) walks a widget subtree and counts objects and nesting depth.
//!    This is the portable, deterministic metric — it counts real `lv_obj`
//!    instances regardless of how the C heap is configured, and every
//!    object-reduction technique (virtualised lists, drawing instead of
//!    instantiating, freeing off-screen trees) moves it directly.
//!
//! 2. [`ResourceProbe`](crate::diag::ResourceProbe) is a pluggable hook for *live* heap and task-stack
//!    figures. There is no portable way to read these — on host they are not
//!    meaningful, and on ESP32 the heap and task stacks are owned by the
//!    application (esp-hal / FreeRTOS), not the library. Apps supply an impl;
//!    the default [`NullProbe`](crate::diag::NullProbe) reports nothing.
//!
//! ## Why not `lv_mem_monitor`?
//!
//! With `LV_USE_STDLIB_MALLOC = LV_STDLIB_CLIB` (the configuration these
//! bindings ship with), LVGL has no internal pool to introspect — it delegates
//! straight to libc `malloc`, so `lv_mem_monitor` reports nothing useful. The
//! figure that actually matters on target is the *system* heap (esp-hal), which
//! a [`ResourceProbe`](crate::diag::ResourceProbe) exposes.

use crate::widgets::Obj;

/// Estimated bytes per `lv_obj`, measured on ESP32-S3 (`lv_obj_t` + `spec_attr`
/// + a small `styles[]`). The audit in
/// `docs/cr-navigator-modal-background-tree-residency.md` puts this at
/// ~120–200 B; we use the midpoint for estimates.
pub const BYTES_PER_OBJECT_EST: u32 = 160;

/// A census of a widget subtree: how many objects it holds and how deep it
/// nests. Both numbers drive heap *and* stack cost — object count dominates
/// heap, nesting depth bounds the recursion depth of layout, style refresh and
/// draw (and therefore the LVGL task's stack high-water mark).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Census {
    /// Total objects in the subtree, including the root.
    pub objects: u32,
    /// Deepest nesting below the root (the root itself is depth 0).
    pub max_depth: u16,
}

impl Census {
    /// Estimated heap footprint in bytes (`objects` × [`BYTES_PER_OBJECT_EST`]).
    ///
    /// A portable estimate, **not** a live reading — styles, draw buffers and
    /// per-object variation are not captured. For the real figure on target,
    /// read a [`ResourceProbe`]. Saturates rather than overflowing.
    pub const fn estimated_heap_bytes(&self) -> u32 {
        self.objects.saturating_mul(BYTES_PER_OBJECT_EST)
    }

    /// Returns `true` if this census exceeds either ceiling of a [`Budget`].
    pub const fn exceeds(&self, budget: &Budget) -> bool {
        self.objects > budget.max_objects || self.max_depth > budget.max_depth
    }
}

impl core::fmt::Display for Census {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} objects, depth {}, ~{} B est",
            self.objects,
            self.max_depth,
            self.estimated_heap_bytes()
        )
    }
}

/// A per-view resource ceiling. Pair with [`Census::exceeds`] (or
/// [`assert_budget`]) to fail loudly in debug builds when a view's widget tree
/// grows past its envelope, instead of silently OOMing a memory-constrained
/// target at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// Maximum allowed object count for the subtree.
    pub max_objects: u32,
    /// Maximum allowed nesting depth.
    pub max_depth: u16,
}

impl Budget {
    /// A budget with the given object ceiling and an unbounded depth.
    pub const fn objects(max_objects: u32) -> Self {
        Self { max_objects, max_depth: u16::MAX }
    }
}

/// Walk `root` and all descendants, returning the subtree [`Census`].
///
/// `root` is typically a screen (`lv_screen_active`) or a container. The walk
/// recurses to `max_depth`; widget trees are normally wide rather than deep, so
/// this stays shallow — see the "flatten the tree" guidance.
pub fn census(root: &Obj) -> Census {
    let mut c = Census::default();
    walk(root, 0, &mut c);
    c
}

fn walk(obj: &Obj, depth: u16, c: &mut Census) {
    c.objects += 1;
    if depth > c.max_depth {
        c.max_depth = depth;
    }
    let n = obj.get_child_count();
    for i in 0..n as i32 {
        if let Some(child) = obj.get_child(i) {
            walk(&child, depth + 1, c);
        }
    }
}

/// In a debug build, panic if `root`'s census exceeds `budget`; no-op in
/// release. Use as a guard right after a view's `create()` so a regression in
/// widget count fails CI/HIL rather than a customer's device.
#[track_caller]
pub fn assert_budget(root: &Obj, budget: &Budget) {
    debug_assert!(
        !census(root).exceeds(budget),
        "widget census {} exceeds budget (max {} objects, depth {})",
        census(root),
        budget.max_objects,
        budget.max_depth,
    );
}

/// A pluggable source of live heap and task-stack figures.
///
/// All methods default to `None` ("unknown"); implement only what a platform
/// can answer. On ESP32 an app typically backs `free_heap_bytes` with
/// `esp_alloc` heap stats and `stack_high_water_bytes` with the FreeRTOS task
/// high-water mark.
pub trait ResourceProbe {
    /// Free system-heap bytes right now, if known.
    fn free_heap_bytes(&self) -> Option<usize> {
        None
    }

    /// Largest contiguous free heap block, if known — a fragmentation
    /// indicator (a low value with high total free means a fragmented heap).
    fn largest_free_block(&self) -> Option<usize> {
        None
    }

    /// Minimum free stack ever observed for the current task (its high-water
    /// mark), in bytes, if known.
    fn stack_high_water_bytes(&self) -> Option<usize> {
        None
    }
}

/// A [`ResourceProbe`] that knows nothing — the host default. Live heap and
/// stack are not meaningful off-target, so every method returns `None`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullProbe;

impl ResourceProbe for NullProbe {}
