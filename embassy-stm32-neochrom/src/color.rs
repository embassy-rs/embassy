//! RGBA color in the format expected by NemaGFX clear/fill APIs.

use crate::ffi::nema_gfx::{NEMA_A8, NEMA_L8, NEMA_RGB565, NEMA_RGBA4444, NEMA_RGBA8888};

/// Opaque 32-bit RGBA color (`0xAARRGGBB`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rgba8888(pub u32);

impl Rgba8888 {
    /// Construct from 8-bit RGBA channels.
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    /// Solid red.
    pub const RED: Self = Self::new(0xFF, 0x00, 0x00, 0xFF);
    /// Solid green.
    pub const GREEN: Self = Self::new(0x00, 0xFF, 0x00, 0xFF);
    /// Solid blue.
    pub const BLUE: Self = Self::new(0x00, 0x00, 0xFF, 0xFF);
    /// Opaque black.
    pub const BLACK: Self = Self::new(0x00, 0x00, 0x00, 0xFF);
    /// Opaque white.
    pub const WHITE: Self = Self::new(0xFF, 0xFF, 0xFF, 0xFF);

    /// Value passed to [`crate::ffi::nema_gfx::nema_clear`].
    #[inline]
    pub const fn bits(self) -> u32 {
        self.0
    }
}

/// NemaGFX texture color formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ColorFormat {
    /// 32-bit RGBA8888 (4 bytes per pixel).
    Rgba8888 = 0,
    /// 16-bit RGB565 (2 bytes per pixel).
    Rgb565 = 4,
    /// 16-bit RGBA4444 (2 bytes per pixel).
    Rgba4444 = 5,
    /// 8-bit Alpha mask (1 byte per pixel).
    A8 = 8,
    /// 8-bit Grayscale (1 byte per pixel).
    L8 = 11,
}

impl ColorFormat {
    /// NemaGFX texture format constant for [`crate::ffi::nema_gfx::nema_bind_dst_tex`].
    pub const fn nema_format(self) -> u32 {
        match self {
            Self::Rgba8888 => NEMA_RGBA8888,
            Self::Rgb565 => NEMA_RGB565,
            Self::Rgba4444 => NEMA_RGBA4444,
            Self::A8 => NEMA_A8,
            Self::L8 => NEMA_L8,
        }
    }
}
