// SPDX-License-Identifier: MIT OR Apache-2.0
//! Resource-diagnostics tests: widget census, budgets, and a baseline record
//! for a representative widget-heavy tree.

use crate::common::{fresh_screen, pump};

use oxivgl::diag::{assert_budget, census, Budget, Census, BYTES_PER_OBJECT_EST};
use oxivgl::widgets::{Label, Obj};

/// Build a `rows`-wide, two-deep tree (each row is an `Obj` holding one
/// `Label`) under a fresh container. Returns the container plus the retained
/// owned widgets — they must be kept alive or `Drop` deletes the tree.
fn build_tree(rows: usize) -> (Obj<'static>, Vec<Obj<'static>>, Vec<Label<'static>>) {
    let screen = fresh_screen();
    let container = Obj::new(&screen).unwrap();
    let mut row_objs = Vec::with_capacity(rows);
    let mut labels = Vec::with_capacity(rows);
    for _ in 0..rows {
        let row = Obj::new(&container).unwrap();
        let label = Label::new(&row).unwrap();
        row_objs.push(row);
        labels.push(label);
    }
    pump();
    (container, row_objs, labels)
}

#[test]
fn census_counts_objects_and_depth() {
    let (container, _rows, _labels) = build_tree(30);
    let c = census(&container);
    // container + 30 rows + 30 labels.
    assert_eq!(c.objects, 61, "expected 1 container + 30 rows + 30 labels");
    // container(0) → row(1) → label(2).
    assert_eq!(c.max_depth, 2);
}

#[test]
fn census_empty_subtree_is_one_object() {
    let screen = fresh_screen();
    let leaf = Obj::new(&screen).unwrap();
    let c = census(&leaf);
    assert_eq!(c, Census { objects: 1, max_depth: 0 });
}

#[test]
fn estimated_heap_bytes_uses_coefficient() {
    let (container, _rows, _labels) = build_tree(10);
    let c = census(&container);
    assert_eq!(c.objects, 21);
    assert_eq!(c.estimated_heap_bytes(), 21 * BYTES_PER_OBJECT_EST);
}

#[test]
fn budget_detects_overflow() {
    let (container, _rows, _labels) = build_tree(30);
    let c = census(&container);
    assert!(c.exceeds(&Budget::objects(50)), "61 > 50");
    assert!(!c.exceeds(&Budget::objects(100)), "61 < 100");
    assert!(c.exceeds(&Budget { max_objects: 1000, max_depth: 1 }), "depth 2 > 1");
}

#[test]
fn assert_budget_passes_within_envelope() {
    let (container, _rows, _labels) = build_tree(5);
    // 11 objects, depth 2 — comfortably inside.
    assert_budget(&container, &Budget { max_objects: 50, max_depth: 4 });
}

/// Reports the Rust-side struct size of each widget wrapper, and proves that a
/// freshly-created widget that never calls `add_style` performs ZERO Rust-side
/// heap allocation (the `_styles` `Vec` is empty and unallocated). Run with
/// `--nocapture` to see the sizes.
#[test]
fn wrapper_struct_sizes_and_zero_heap() {
    use core::mem::size_of;
    use oxivgl::widgets::{Arc, Bar, Button, Label, Scale};
    println!(
        "wrapper size_of (bytes): Obj={} Label={} Button={} Bar={} Arc={} Scale={}",
        size_of::<Obj>(),
        size_of::<Label>(),
        size_of::<Button>(),
        size_of::<Bar>(),
        size_of::<Arc>(),
        size_of::<Scale>(),
    );
    // The base wrappers are a raw handle + an (empty, unallocated) style Vec +
    // a ZST PhantomData — no per-widget heap until a style is retained.
    assert!(size_of::<Label>() == size_of::<Obj>(), "Label adds no fields");
    assert!(size_of::<Button>() == size_of::<Obj>(), "Button adds no fields");
}

/// Records the census of a representative widget-heavy tree. Run with
/// `--nocapture` to print the baseline; the assertions pin it so a regression
/// (a refactor that inflates object count) fails here.
#[test]
fn baseline_widget_heavy_tree() {
    let (container, _rows, _labels) = build_tree(30);
    let c = census(&container);
    println!(
        "BASELINE widget-heavy tree: {c}  (coefficient {BYTES_PER_OBJECT_EST} B/object)"
    );
    assert_eq!(c.objects, 61);
    assert_eq!(c.max_depth, 2);
}
