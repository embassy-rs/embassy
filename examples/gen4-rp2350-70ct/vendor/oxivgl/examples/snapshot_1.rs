#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Snapshot 1 — Capture and display a rotated, scaled widget snapshot
//!
//! Simplified: interactive snapshot retake on press/release omitted —
//! event user_data passing not yet wrapped. Star images replaced with
//! colored squares (compiled star asset not available).

use oxivgl::{
    layout::{FlexAlign, FlexFlow},
    snapshot::Snapshot,
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Image, Obj, WidgetError},
};

#[derive(Default)]
struct Snapshot1 {
    _snapshot_img: Option<Image<'static>>,
    _container: Option<Obj<'static>>,
    _item0: Option<Obj<'static>>,
    _item1: Option<Obj<'static>>,
    _item2: Option<Obj<'static>>,
    _item3: Option<Obj<'static>>,
    // snapshot declared LAST → dropped last, satisfying LVGL's
    // pointer-lifetime requirement (spec §3.1).
    _snapshot: Option<Snapshot>,
}

/// Colors for the four squares inside the container.
const ITEM_COLORS: [u32; 4] = [0xe74c3c, 0x2ecc71, 0x3498db, 0xf39c12];

impl View for Snapshot1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        // Shared background style for the root screen container.
        let screen_style = Style::new(|s| {
            s.bg_color_hex(0xadd8e6).bg_opa(255);
        });
        container.add_style(&screen_style, Selector::DEFAULT);

        // Image widget that will display the snapshot (source set later).
        let snapshot_img = Image::new(container)?;
        snapshot_img.center();

        // Container: 180×180, centered, flex row-wrap, radius 50.
        let container = Obj::new(container)?;
        container.size(180, 180).center();
        // Shared style: radius 50, dark background, pad 5.
        let container_style = Style::new(|s| {
            s.radius(50).bg_color_hex(0x303030).bg_opa(255).pad_all(5);
        });
        container.add_style(&container_style, Selector::DEFAULT);
        container.set_flex_flow(FlexFlow::RowWrap);
        container.set_flex_align(FlexAlign::SpaceEvenly, FlexAlign::Center, FlexAlign::Center);

        // Shared style for the four squares: opaque, no border.
        let item_style = Style::new(|s| {
            s.bg_opa(255).border_width(0);
        });
        // Four colored squares inside the container. The shared style carries
        // the common opacity/border; each item gets its own bg color.
        let item0 = Obj::new(&container)?;
        item0.size(50, 50).add_style(&item_style, Selector::DEFAULT);
        item0.add_style(&Style::new(|s| { s.bg_color_hex(ITEM_COLORS[0]); }), Selector::DEFAULT);
        let item1 = Obj::new(&container)?;
        item1.size(50, 50).add_style(&item_style, Selector::DEFAULT);
        item1.add_style(&Style::new(|s| { s.bg_color_hex(ITEM_COLORS[1]); }), Selector::DEFAULT);
        let item2 = Obj::new(&container)?;
        item2.size(50, 50).add_style(&item_style, Selector::DEFAULT);
        item2.add_style(&Style::new(|s| { s.bg_color_hex(ITEM_COLORS[2]); }), Selector::DEFAULT);
        let item3 = Obj::new(&container)?;
        item3.size(50, 50).add_style(&item_style, Selector::DEFAULT);
        item3.add_style(&Style::new(|s| { s.bg_color_hex(ITEM_COLORS[3]); }), Selector::DEFAULT);

        // Take snapshot of the container (ARGB8888).
        let snapshot = Snapshot::take_widget(&container).ok_or(WidgetError::LvglNullPointer)?;

        // Configure the snapshot image: 0.5× scale (128), 30° rotation (300).
        snapshot_img.set_src_snapshot(&snapshot);
        snapshot_img.set_scale(128);
        snapshot_img.set_rotation(300);
        snapshot_img.center();

                self._snapshot_img = Some(snapshot_img);
        self._container = Some(container);
        self._item0 = Some(item0);
        self._item1 = Some(item1);
        self._item2 = Some(item2);
        self._item3 = Some(item3);
        self._snapshot = Some(snapshot);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Snapshot1::default());
