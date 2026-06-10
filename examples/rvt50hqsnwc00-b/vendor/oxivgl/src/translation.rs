// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL translation/i18n support.
//!
//! Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.
//!
//! Register static translation packs, then switch languages at runtime.
//! Labels created with [`Label::set_translation_tag`](crate::widgets::Label::set_translation_tag) auto-update when
//! the language changes.
//!
//! # Example
//!
//! ```no_run
//! use oxivgl::translation::{self, StaticCStr as S};
//!
//! static LANGS: [S; 3] = [S::from_cstr(c"en"), S::from_cstr(c"de"), S::NULL];
//! static TAGS: [S; 2] = [S::from_cstr(c"hello"), S::NULL];
//! static TRANS: [S; 2] = [S::from_cstr(c"Hello"), S::from_cstr(c"Hallo")];
//!
//! translation::add_static(&LANGS, &TAGS, &TRANS);
//! translation::set_language(c"de");
//! ```

use core::ffi::{CStr, c_char};
use oxivgl_sys::*;

/// A `*const c_char` wrapper that is `Sync` + `Send`.
///
/// Use for `'static` translation arrays. The pointer must refer to a
/// compile-time string literal (`c"..."` or `&'static CStr`).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct StaticCStr(*const c_char);

// SAFETY: the pointer targets compile-time string literals which are
// inherently immutable and 'static — safe to share across threads.
unsafe impl Sync for StaticCStr {}
unsafe impl Send for StaticCStr {}

impl StaticCStr {
    /// Create from a `&'static CStr` literal.
    pub const fn from_cstr(s: &'static CStr) -> Self {
        Self(s.as_ptr())
    }
    /// NULL sentinel for array termination.
    pub const NULL: Self = Self(core::ptr::null());
}

/// Register a translation pack from static NULL-terminated arrays.
///
/// - `languages`: e.g. `&[S::from_cstr(c"en"), S::from_cstr(c"de"), S::NULL]`
/// - `tags`: NULL-terminated tag names
/// - `translations`: flattened by language (tags × languages)
///
/// All arrays must be `'static`. LVGL stores the pointers directly.
pub fn add_static(
    languages: &'static [StaticCStr],
    tags: &'static [StaticCStr],
    translations: &'static [StaticCStr],
) {
    // SAFETY: StaticCStr is repr(transparent) over *const c_char.
    // All pointers are 'static (enforced by the type system + 'static borrow).
    // Arrays are NULL-terminated per API contract.
    unsafe {
        lv_translation_add_static(
            languages.as_ptr().cast(),
            tags.as_ptr().cast(),
            translations.as_ptr().cast(),
        );
    }
}

/// Set the active language. All widgets with translation tags will update.
///
/// `lang` must match one of the registered language strings (e.g. `c"en"`).
pub fn set_language(lang: &CStr) {
    // SAFETY: lang is a valid NUL-terminated CStr. LVGL copies the string
    // internally via lv_strdup (see lv_translation_set_language in
    // lv_translation.c), so no lifetime requirement on the caller.
    unsafe { lv_translation_set_language(lang.as_ptr()) };
}

/// Get the currently active language, or `None` if none set.
pub fn get_language() -> Option<&'static CStr> {
    // SAFETY: lv_translation_get_language returns a static pointer or NULL.
    let ptr = unsafe { lv_translation_get_language() };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(ptr) })
    }
}

/// Look up the translation for `tag` in the current language.
///
/// Falls back to the first registered language if the current language has no
/// entry, then falls back to the tag string itself if no pack contains the tag.
/// Never returns `None` — LVGL guarantees a non-NULL result.
pub fn translate(tag: &CStr) -> &'static CStr {
    // SAFETY: lv_translation_get returns a pointer to a string in the
    // static translation pack or the tag itself — both are 'static.
    let ptr = unsafe { lv_translation_get(tag.as_ptr()) };
    // LVGL never returns NULL — falls back to the tag string.
    unsafe { CStr::from_ptr(ptr) }
}

/// A dynamic translation pack. Created by [`add_dynamic`].
pub struct DynamicPack {
    inner: *mut lv_translation_pack_t,
}

/// A tag descriptor within a dynamic pack. Created by [`DynamicPack::add_tag`].
pub struct TagDsc {
    inner: *mut lv_translation_tag_dsc_t,
}

/// Create a new empty dynamic translation pack and register it with LVGL.
///
/// Add languages via [`DynamicPack::add_language`] and tags via
/// [`DynamicPack::add_tag`] before calling [`set_language`].
pub fn add_dynamic() -> DynamicPack {
    // SAFETY: allocates a new empty pack via LVGL's allocator.
    let ptr = unsafe { lv_translation_add_dynamic() };
    assert!(!ptr.is_null(), "lv_translation_add_dynamic returned NULL");
    DynamicPack { inner: ptr }
}

impl DynamicPack {
    /// Add a language to this dynamic pack.
    ///
    /// All languages should be added before any tags. The `lang` string is
    /// copied by LVGL internally.
    pub fn add_language(&self, lang: &CStr) {
        // SAFETY: pack pointer valid (from add_dynamic); LVGL copies the string.
        unsafe { lv_translation_add_language(self.inner, lang.as_ptr()) };
    }

    /// Add a named tag to this dynamic pack.
    ///
    /// Returns a [`TagDsc`] that can be used with [`DynamicPack::set_translation`]
    /// to attach per-language translation strings.
    pub fn add_tag(&self, tag_name: &CStr) -> TagDsc {
        // SAFETY: pack pointer valid; LVGL copies the string.
        let ptr = unsafe { lv_translation_add_tag(self.inner, tag_name.as_ptr()) };
        assert!(!ptr.is_null(), "lv_translation_add_tag returned NULL");
        TagDsc { inner: ptr }
    }

    /// Set the translation for `tag` in the language at `lang_idx`.
    ///
    /// `lang_idx` is zero-based and corresponds to the order in which languages
    /// were added via [`DynamicPack::add_language`]. The `text` string is copied
    /// by LVGL internally.
    pub fn set_translation(&self, tag: &TagDsc, lang_idx: u32, text: &CStr) {
        // SAFETY: pack and tag pointers valid; LVGL copies the string.
        unsafe {
            lv_translation_set_tag_translation(self.inner, tag.inner, lang_idx, text.as_ptr());
        };
    }
}
