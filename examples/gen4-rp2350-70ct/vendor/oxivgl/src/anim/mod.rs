// SPDX-License-Identifier: MIT OR Apache-2.0
//! Animation descriptors, path functions, and timeline management.

mod anim;
mod anim_timeline;

pub use anim::{
    ANIM_REPEAT_INFINITE, Anim, AnimHandle, anim_path_bounce, anim_path_ease_in, anim_path_ease_in_out,
    anim_path_ease_out, anim_path_linear, anim_path_overshoot, anim_set_arc_value, anim_set_bar_value, anim_set_height,
    anim_set_pad_column, anim_set_pad_row, anim_set_size, anim_set_slider_value, anim_set_translate_x,
    anim_set_scale_rotation, anim_set_width, anim_set_x, anim_set_y,
};
pub use anim_timeline::{ANIM_TIMELINE_PROGRESS_MAX, AnimTimeline};
