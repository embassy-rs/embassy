// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests — exercise widgets against a real (headless) LVGL instance.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test integration
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

// Tests exercise the deprecated inline style setters to verify they still work.
#![allow(deprecated)]

#[path = "../common/mod.rs"]
mod common;

mod anim;
mod diag;
mod draw;
mod event;
mod group;
mod keypad;
mod misc;
mod navigator_modal;
mod navigator_toast;
mod obj;
mod observer;
mod pointer;
mod style;
mod timer;
mod widgets_container;
mod widgets_display;
mod widgets_input;
