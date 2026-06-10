// SPDX-License-Identifier: MIT OR Apache-2.0
use oxivgl_sys::*;

use super::anim::Anim;

/// Owning wrapper around `lv_anim_timeline_t*`. Calls `lv_anim_timeline_delete` on drop.
pub struct AnimTimeline {
    handle: *mut lv_anim_timeline_t,
}

impl core::fmt::Debug for AnimTimeline {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnimTimeline").finish_non_exhaustive()
    }
}

impl AnimTimeline {
    /// Create a new empty animation timeline.
    pub fn new() -> Self {
        let handle = unsafe { lv_anim_timeline_create() };
        assert!(!handle.is_null(), "lv_anim_timeline_create returned NULL");
        Self { handle }
    }

    /// Add an animation at the given start time (ms).
    pub fn add(&mut self, start_time: u32, anim: &Anim<'_>) -> &mut Self {
        unsafe { lv_anim_timeline_add(self.handle, start_time, &anim.inner) };
        self
    }

    /// Start timeline playback. Returns total duration in ms.
    pub fn start(&self) -> u32 {
        unsafe { lv_anim_timeline_start(self.handle) }
    }

    /// Pause timeline playback.
    pub fn pause(&self) {
        unsafe { lv_anim_timeline_pause(self.handle) }
    }

    /// Enable/disable reverse playback.
    pub fn set_reverse(&self, reverse: bool) {
        unsafe { lv_anim_timeline_set_reverse(self.handle, reverse) }
    }

    /// Set timeline progress (0 to ANIM_TIMELINE_PROGRESS_MAX).
    pub fn set_progress(&self, progress: u16) {
        unsafe { lv_anim_timeline_set_progress(self.handle, progress) }
    }

    /// Return the raw timeline pointer.
    pub fn handle(&self) -> *mut lv_anim_timeline_t {
        self.handle
    }
}

impl Drop for AnimTimeline {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { lv_anim_timeline_delete(self.handle) };
        }
    }
}

/// `LV_ANIM_TIMELINE_PROGRESS_MAX`
pub const ANIM_TIMELINE_PROGRESS_MAX: u16 = LV_ANIM_TIMELINE_PROGRESS_MAX as u16;
