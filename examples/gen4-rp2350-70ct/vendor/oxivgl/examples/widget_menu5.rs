#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Menu 5 — Settings menu with sidebar
//!
//! A full settings UI with sidebar navigation, sections, separators,
//! sliders, switches, and a sidebar-toggle switch. Root back button
//! shows a message box.
//!
//! Simplified: the sidebar toggle switch is identified by
//! `VALUE_CHANGED` event code (it is the only interactive switch
//! whose events bubble to the screen).

use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    style::{color_brightness, color_darken, Selector, Style},
    symbols,
    view::{NavAction, View},
    widgets::{
        AsLvHandle, Child, Image, Label, LabelLongMode, Menu, Msgbox, Obj, Part, Slider,
        Switch, WidgetError,
    },
};

#[derive(Default)]
struct WidgetMenu5 {
    menu: Option<Menu<'static>>,
    /// Tracks whether sidebar mode is active (toggled via the switch).
    sidebar_enabled: bool,
}

/// Create a menu container with an optional icon and text label.
fn create_text<'a>(
    parent: &'a impl AsLvHandle,
    icon: Option<&symbols::Symbol>,
    txt: &str,
    variant2: bool,
) -> Child<Obj<'a>> {
    let cont = Menu::cont_create(parent);

    let img_child = if let Some(sym) = icon {
        let img = Image::new(&cont).ok();
        if let Some(ref img) = img {
            img.set_src_symbol(sym);
        }
        img
    } else {
        None
    };

    if let Ok(label) = Label::new(&cont) {
        label.text(txt);
        label.set_long_mode(LabelLongMode::ScrollCircular);
        label.set_flex_grow(1);

        if variant2 {
            if let Some(ref img) = img_child {
                img.add_flag(ObjFlag::FLEX_IN_NEW_TRACK);
                img.swap(&label);
            }
        }
        core::mem::forget(label);
    }
    if let Some(img) = img_child {
        core::mem::forget(img);
    }

    cont
}

/// Create a menu container with icon, text, and a slider underneath.
fn create_slider(
    parent: &impl AsLvHandle,
    icon: &symbols::Symbol,
    txt: &str,
    min: i32,
    max: i32,
    val: i32,
) {
    let cont = create_text(parent, Some(icon), txt, true);

    if let Ok(slider) = Slider::new(&cont) {
        slider.set_flex_grow(1);
        slider.set_range(min, max);
        slider.set_value(val);
        core::mem::forget(slider);
    }
}

/// Create a menu container with icon, text, and a switch.
fn create_switch(parent: &impl AsLvHandle, icon: &symbols::Symbol, txt: &str, checked: bool) {
    let cont = create_text(parent, Some(icon), txt, false);

    if let Ok(sw) = Switch::new(&cont) {
        if checked {
            sw.add_state(ObjState::CHECKED);
        }
        sw.bubble_events();
        core::mem::forget(sw);
    }
}

impl View for WidgetMenu5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let menu = Menu::new(container)?;

        // Darken menu background slightly (theme detection)
        let bg = menu.get_style_bg_color(Part::Main);
        let darkened = if color_brightness(bg) > 127 {
            color_darken(bg, 10)
        } else {
            color_darken(bg, 50)
        };
        let menu_bg_style = Style::new(|s| {
            s.bg_color(darkened);
        });
        menu.add_style(&menu_bg_style, Selector::DEFAULT);

        menu.set_mode_root_back_button(true);
        menu.bubble_events();
        menu.size(320, 240).center();

        let header_pad = menu.get_main_header().get_style_pad_left(Part::Main);
        let header_pad_style = Style::new(|s| {
            s.pad_hor(header_pad);
        });

        // ── Sub-pages ──────────────────────────────────────────────────

        // Mechanics
        let sub_mechanics = menu.page_create(None);
        sub_mechanics.add_style(&header_pad_style, Selector::DEFAULT);
        Menu::separator_create(&sub_mechanics);
        let section = Menu::section_create(&sub_mechanics);
        create_slider(&section, &symbols::SETTINGS, "Velocity", 0, 150, 120);
        create_slider(&section, &symbols::SETTINGS, "Acceleration", 0, 150, 50);
        create_slider(&section, &symbols::SETTINGS, "Weight limit", 0, 150, 80);

        // Sound
        let sub_sound = menu.page_create(None);
        sub_sound.add_style(&header_pad_style, Selector::DEFAULT);
        Menu::separator_create(&sub_sound);
        let section = Menu::section_create(&sub_sound);
        create_switch(&section, &symbols::AUDIO, "Sound", false);

        // Display
        let sub_display = menu.page_create(None);
        sub_display.add_style(&header_pad_style, Selector::DEFAULT);
        Menu::separator_create(&sub_display);
        let section = Menu::section_create(&sub_display);
        create_slider(&section, &symbols::SETTINGS, "Brightness", 0, 150, 100);

        // Software info
        let sub_sw_info = menu.page_create(None);
        sub_sw_info.add_style(&header_pad_style, Selector::DEFAULT);
        let section = Menu::section_create(&sub_sw_info);
        create_text(&section, None, "Version 1.0", false);

        // Legal info
        let sub_legal = menu.page_create(None);
        sub_legal.add_style(&header_pad_style, Selector::DEFAULT);
        let section = Menu::section_create(&sub_legal);
        for _ in 0..15 {
            create_text(
                &section,
                None,
                "This is a long long long long long long long long long text, if it is long enough it may scroll.",
                false,
            );
        }

        // About
        let sub_about = menu.page_create(None);
        sub_about.add_style(&header_pad_style, Selector::DEFAULT);
        Menu::separator_create(&sub_about);
        let section = Menu::section_create(&sub_about);
        let cont = create_text(&section, None, "Software information", false);
        menu.set_load_page_event(&cont, &sub_sw_info);
        let cont = create_text(&section, None, "Legal information", false);
        menu.set_load_page_event(&cont, &sub_legal);

        // Menu mode
        let sub_menu_mode = menu.page_create(None);
        sub_menu_mode.add_style(&header_pad_style, Selector::DEFAULT);
        Menu::separator_create(&sub_menu_mode);
        let section = Menu::section_create(&sub_menu_mode);
        create_switch(&section, &symbols::AUDIO, "Sidebar enable", true);

        // ── Root page (sidebar) ────────────────────────────────────────

        let root_page = menu.page_create(Some("Settings"));
        root_page.add_style(&header_pad_style, Selector::DEFAULT);

        let section = Menu::section_create(&root_page);
        let cont = create_text(&section, Some(&symbols::SETTINGS), "Mechanics", false);
        menu.set_load_page_event(&cont, &sub_mechanics);
        let cont = create_text(&section, Some(&symbols::AUDIO), "Sound", false);
        menu.set_load_page_event(&cont, &sub_sound);
        let cont = create_text(&section, Some(&symbols::SETTINGS), "Display", false);
        menu.set_load_page_event(&cont, &sub_display);

        create_text(&root_page, None, "Others", false);
        let section = Menu::section_create(&root_page);
        let cont = create_text(&section, None, "About", false);
        menu.set_load_page_event(&cont, &sub_about);
        let cont = create_text(&section, Some(&symbols::SETTINGS), "Menu mode", false);
        menu.set_load_page_event(&cont, &sub_menu_mode);

        menu.set_sidebar_page(&root_page);

        // Simulate click on first sidebar item to load initial page
        if let Some(sidebar) = menu.get_cur_sidebar_page() {
            if let Some(first_section) = sidebar.get_child(0) {
                if let Some(first_cont) = first_section.get_child(0) {
                    first_cont.send_event(EventCode::CLICKED);
                }
            }
        }

        self.menu = Some(menu);
        self.sidebar_enabled = true;
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref menu) = self.menu {
            // Root back button → show message box
            if event.code() == EventCode::CLICKED
                && menu.back_button_is_root(&event.target())
            {
                if let Ok(mbox) = Msgbox::new(None::<&Obj<'_>>) {
                    mbox.add_title("Hello");
                    mbox.add_text("Root back btn click.");
                    mbox.add_close_button();
                    core::mem::forget(mbox);
                }
            }

            // Sidebar toggle switch
            if event.code() == EventCode::VALUE_CHANGED {
                let target = event.target();
                let checked = target.has_state(ObjState::CHECKED);

                if checked && !self.sidebar_enabled {
                    self.sidebar_enabled = true;
                    menu.clear_page();
                    // Re-activate sidebar: click first item
                    if let Some(sidebar) = menu.get_cur_sidebar_page() {
                        if let Some(first_section) = sidebar.get_child(0) {
                            if let Some(first_cont) = first_section.get_child(0) {
                                first_cont.send_event(EventCode::CLICKED);
                            }
                        }
                    }
                } else if !checked && self.sidebar_enabled {
                    self.sidebar_enabled = false;
                    menu.clear_sidebar();
                    menu.clear_history();
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetMenu5::default());
