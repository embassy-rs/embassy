// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL built-in icon symbols (Font Awesome subset).
//!
//! Each constant is a NUL-terminated UTF-8 byte slice matching the `LV_SYMBOL_*`
//! defines from `lv_symbol_def.h`. Pass to [`List::add_button`](crate::widgets::List::add_button)
//! or [`Obj::style_bg_image_src_symbol`](crate::widgets::Obj::style_bg_image_src_symbol).

/// LVGL symbol constant — a NUL-terminated UTF-8 byte slice.
///
/// Use [`as_ptr`](Symbol::as_ptr) to get a `*const c_char` for LVGL APIs.
pub struct Symbol(&'static [u8]);

impl Symbol {
    /// Raw C string pointer. The symbol is statically allocated and NUL-terminated.
    pub fn as_ptr(&self) -> *const core::ffi::c_char {
        self.0.as_ptr() as *const core::ffi::c_char
    }
}

impl core::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Show hex bytes for non-ASCII symbol content
        write!(f, "Symbol({:?})", core::str::from_utf8(&self.0[..self.0.len().saturating_sub(1)]))
    }
}

/// Audio icon.
pub const AUDIO: Symbol = Symbol(b"\xEF\x80\x81\0");
/// Video icon.
pub const VIDEO: Symbol = Symbol(b"\xEF\x80\x88\0");
/// List icon.
pub const LIST: Symbol = Symbol(b"\xEF\x80\x8B\0");
/// OK / checkmark icon.
pub const OK: Symbol = Symbol(b"\xEF\x80\x8C\0");
/// Close / X icon.
pub const CLOSE: Symbol = Symbol(b"\xEF\x80\x8D\0");
/// Power icon.
pub const POWER: Symbol = Symbol(b"\xEF\x80\x91\0");
/// Settings / gear icon.
pub const SETTINGS: Symbol = Symbol(b"\xEF\x80\x93\0");
/// Home icon.
pub const HOME: Symbol = Symbol(b"\xEF\x80\x95\0");
/// Download icon.
pub const DOWNLOAD: Symbol = Symbol(b"\xEF\x80\x99\0");
/// Drive icon.
pub const DRIVE: Symbol = Symbol(b"\xEF\x80\x9C\0");
/// Refresh icon.
pub const REFRESH: Symbol = Symbol(b"\xEF\x80\xA1\0");
/// Mute icon.
pub const MUTE: Symbol = Symbol(b"\xEF\x80\xA6\0");
/// Volume mid icon.
pub const VOLUME_MID: Symbol = Symbol(b"\xEF\x80\xA7\0");
/// Volume max icon.
pub const VOLUME_MAX: Symbol = Symbol(b"\xEF\x80\xA8\0");
/// Image icon.
pub const IMAGE: Symbol = Symbol(b"\xEF\x80\xBE\0");
/// Tint icon.
pub const TINT: Symbol = Symbol(b"\xEF\x81\x83\0");
/// Previous icon.
pub const PREV: Symbol = Symbol(b"\xEF\x81\x88\0");
/// Play icon.
pub const PLAY: Symbol = Symbol(b"\xEF\x81\x8B\0");
/// Pause icon.
pub const PAUSE: Symbol = Symbol(b"\xEF\x81\x8C\0");
/// Stop icon.
pub const STOP: Symbol = Symbol(b"\xEF\x81\x8D\0");
/// Next icon.
pub const NEXT: Symbol = Symbol(b"\xEF\x81\x91\0");
/// Eject icon.
pub const EJECT: Symbol = Symbol(b"\xEF\x81\x92\0");
/// Left arrow icon.
pub const LEFT: Symbol = Symbol(b"\xEF\x81\x93\0");
/// Right arrow icon.
pub const RIGHT: Symbol = Symbol(b"\xEF\x81\x94\0");
/// Plus icon.
pub const PLUS: Symbol = Symbol(b"\xEF\x81\xA7\0");
/// Minus icon.
pub const MINUS: Symbol = Symbol(b"\xEF\x81\xA8\0");
/// Eye open icon.
pub const EYE_OPEN: Symbol = Symbol(b"\xEF\x81\xAE\0");
/// Eye close icon.
pub const EYE_CLOSE: Symbol = Symbol(b"\xEF\x81\xB0\0");
/// Warning icon.
pub const WARNING: Symbol = Symbol(b"\xEF\x81\xB1\0");
/// Shuffle icon.
pub const SHUFFLE: Symbol = Symbol(b"\xEF\x81\xB4\0");
/// Up arrow icon.
pub const UP: Symbol = Symbol(b"\xEF\x81\xB7\0");
/// Down arrow icon.
pub const DOWN: Symbol = Symbol(b"\xEF\x81\xB8\0");
/// Loop icon.
pub const LOOP: Symbol = Symbol(b"\xEF\x81\xB9\0");
/// Directory / folder icon.
pub const DIRECTORY: Symbol = Symbol(b"\xEF\x81\xBB\0");
/// Upload icon.
pub const UPLOAD: Symbol = Symbol(b"\xEF\x82\x93\0");
/// Call / phone icon.
pub const CALL: Symbol = Symbol(b"\xEF\x82\x95\0");
/// Cut icon.
pub const CUT: Symbol = Symbol(b"\xEF\x83\x84\0");
/// Copy icon.
pub const COPY: Symbol = Symbol(b"\xEF\x83\x85\0");
/// Save icon.
pub const SAVE: Symbol = Symbol(b"\xEF\x83\x87\0");
/// Bars / hamburger menu icon.
pub const BARS: Symbol = Symbol(b"\xEF\x83\x89\0");
/// Envelope icon.
pub const ENVELOPE: Symbol = Symbol(b"\xEF\x83\xA0\0");
/// Charge / lightning icon.
pub const CHARGE: Symbol = Symbol(b"\xEF\x83\xA7\0");
/// Paste icon.
pub const PASTE: Symbol = Symbol(b"\xEF\x83\xAA\0");
/// Bell icon.
pub const BELL: Symbol = Symbol(b"\xEF\x83\xB3\0");
/// Keyboard icon.
pub const KEYBOARD: Symbol = Symbol(b"\xEF\x84\x9C\0");
/// GPS / navigation icon.
pub const GPS: Symbol = Symbol(b"\xEF\x84\xA4\0");
/// File icon.
pub const FILE: Symbol = Symbol(b"\xEF\x85\x9B\0");
/// WiFi icon.
pub const WIFI: Symbol = Symbol(b"\xEF\x87\xAB\0");
/// Battery full icon.
pub const BATTERY_FULL: Symbol = Symbol(b"\xEF\x89\x80\0");
/// Battery 3/4 icon.
pub const BATTERY_3: Symbol = Symbol(b"\xEF\x89\x81\0");
/// Battery 2/4 icon.
pub const BATTERY_2: Symbol = Symbol(b"\xEF\x89\x82\0");
/// Battery 1/4 icon.
pub const BATTERY_1: Symbol = Symbol(b"\xEF\x89\x83\0");
/// Battery empty icon.
pub const BATTERY_EMPTY: Symbol = Symbol(b"\xEF\x89\x84\0");
/// USB icon.
pub const USB: Symbol = Symbol(b"\xEF\x8A\x87\0");
/// Bluetooth icon.
pub const BLUETOOTH: Symbol = Symbol(b"\xEF\x8A\x93\0");
/// Trash icon.
pub const TRASH: Symbol = Symbol(b"\xEF\x8B\xAD\0");
/// Edit / pencil icon.
pub const EDIT: Symbol = Symbol(b"\xEF\x8C\x84\0");
/// Backspace icon.
pub const BACKSPACE: Symbol = Symbol(b"\xEF\x95\x9A\0");
/// SD card icon.
pub const SD_CARD: Symbol = Symbol(b"\xEF\x9F\x82\0");
/// New line icon.
pub const NEW_LINE: Symbol = Symbol(b"\xEF\xA2\xA2\0");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_as_ptr_nonnull() {
        assert!(!OK.as_ptr().is_null());
        assert!(!CLOSE.as_ptr().is_null());
        assert!(!SETTINGS.as_ptr().is_null());
    }

    #[test]
    fn symbol_as_ptr_nul_terminated() {
        // The byte slice ends with \0; as_ptr() must point to valid memory.
        let ptr = OK.as_ptr();
        // SAFETY: ptr is a valid static NUL-terminated byte slice.
        let len = unsafe { core::ffi::CStr::from_ptr(ptr) }.to_bytes().len();
        assert!(len > 0, "symbol bytes should be non-empty before NUL");
    }

    #[test]
    fn symbol_debug_fmt() {
        let s = format!("{:?}", OK);
        assert!(!s.is_empty());
    }

    #[test]
    fn symbol_all_nonnull() {
        let syms: &[&[u8]] = &[
            AUDIO.0, VIDEO.0, LIST.0, OK.0, CLOSE.0, POWER.0, SETTINGS.0,
            HOME.0, DOWNLOAD.0, DRIVE.0, REFRESH.0, MUTE.0, VOLUME_MID.0,
            VOLUME_MAX.0, IMAGE.0, TINT.0, PREV.0, PLAY.0, PAUSE.0, STOP.0,
            NEXT.0, EJECT.0, LEFT.0, RIGHT.0, PLUS.0, MINUS.0, EYE_OPEN.0,
            EYE_CLOSE.0, WARNING.0, SHUFFLE.0, UP.0, DOWN.0, LOOP.0,
            DIRECTORY.0, UPLOAD.0, CALL.0, CUT.0, COPY.0, SAVE.0, BARS.0,
            ENVELOPE.0, CHARGE.0, PASTE.0, BELL.0, KEYBOARD.0, GPS.0,
            FILE.0, WIFI.0, BATTERY_FULL.0, BATTERY_3.0, BATTERY_2.0,
            BATTERY_1.0, BATTERY_EMPTY.0, USB.0, BLUETOOTH.0, TRASH.0,
            EDIT.0, BACKSPACE.0, SD_CARD.0, NEW_LINE.0,
        ];
        for s in syms {
            assert!(!s.is_empty(), "symbol slice should not be empty");
            assert_eq!(*s.last().unwrap(), 0u8, "symbol should be NUL-terminated");
        }
    }
}
