//! GFXMMU — graphics MMU for LTDC framebuffers.
//!
//! Maps a virtual framebuffer address space (used by LTDC) onto up to four physical
//! memory buffers using a line LUT. Typical use is a contiguous RGB framebuffer in
//! one physical buffer while LTDC reads through the GFXMMU aperture.

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::interrupt::typelevel::Interrupt;
use crate::pac::gfxmmu::Gfxmmu as Regs;
#[cfg(gfxmmu_v2)]
use crate::pac::gfxmmu::vals::Bm192;
use crate::{Peri, interrupt, rcc};

/// GFXMMU block size in bytes.
pub const BLOCK_SIZE: u32 = 16;

/// GFXMMU driver error.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// AHB master error reported in status.
    MasterError,
    /// Buffer overflow reported in status.
    BufferOverflow,
}

/// Physical buffer description for one GFXMMU buffer slot (0..3).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BufferConfig {
    /// CPU address of the physical buffer (must be 16-byte aligned).
    pub base_addr: u32,
}

impl BufferConfig {
    /// Encode `base_addr` into BCR register fields.
    fn encode(self) -> (u32, u16) {
        let pbo = (self.base_addr >> 4) & 0x0007_ffff;
        let pbba = ((self.base_addr >> 22) & 0x01ff) as u16;
        (pbo, pbba)
    }
}

/// Driver configuration applied at init.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Use 192 blocks per line instead of 256.
    pub block_mode_192: bool,
    /// Enable the cache unit.
    ///
    /// Not available on N6 (`gfxmmu_n6`): the block has no cache controller,
    /// so this field doesn't exist for that variant.
    #[cfg(gfxmmu_v2)]
    pub cache_enable: bool,
    /// Default 32-bit fill value for unmapped blocks.
    pub default_value: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            block_mode_192: true,
            #[cfg(gfxmmu_v2)]
            cache_enable: true,
            default_value: 0,
        }
    }
}

/// One LUT line entry.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LutLine {
    /// Line is mapped through the LUT.
    pub enabled: bool,
    /// First valid block index within the line.
    pub first_block: u8,
    /// Last valid block index within the line.
    pub last_block: u8,
    /// Line offset in blocks from the physical buffer base (block 0 of this line).
    pub line_offset_blocks: u32,
}

/// GFXMMU driver.
pub struct Gfxmmu<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Gfxmmu<'d, T> {
    /// Create and configure GFXMMU.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let regs = T::regs();
        regs.dvr().write(|w| w.set_dv(config.default_value));
        regs.cr().modify(|w| {
            #[cfg(gfxmmu_v2)]
            {
                w.set_bm(
                    0,
                    if config.block_mode_192 {
                        Bm192::_192blocksPerLine
                    } else {
                        Bm192::_256blocksPerLine
                    },
                );
                w.set_ce(config.cache_enable);
            }
            #[cfg(gfxmmu_n6)]
            {
                w.set_bs(config.block_mode_192);
                w.set_ate(true);
            }
        });

        Self { _peri: peri }
    }

    /// Configure one of the four physical buffer slots.
    pub fn set_buffer(&mut self, index: usize, config: BufferConfig) {
        assert!(index < 4);
        let (pbo, pbba) = config.encode();
        T::regs().bcr(index).modify(|w| {
            w.set_pbo(pbo);
            w.set_pbba(pbba);
        });
    }

    /// Program one LUT line.
    pub fn set_lut_line(&mut self, line: usize, entry: LutLine) {
        assert!(line < 1024);
        T::regs().lutl(line).write(|w| {
            w.set_en(entry.enabled);
            w.set_fvb(entry.first_block);
            w.set_lvb(entry.last_block);
        });
        T::regs().luth(line).write(|w| w.set_lo(entry.line_offset_blocks));
    }

    /// Configure a contiguous linear framebuffer in buffer slot 0.
    ///
    /// `width_px` and `bytes_per_pixel` define the active line width. Lines `0..height`
    /// are mapped sequentially in the physical buffer.
    pub fn configure_linear_framebuffer(
        &mut self,
        buffer: BufferConfig,
        width_px: u16,
        height: u16,
        bytes_per_pixel: u8,
        block_mode_192: bool,
    ) {
        self.set_buffer(0, buffer);

        let bytes_per_line = width_px as u32 * bytes_per_pixel as u32;
        let blocks_per_line =
            ((bytes_per_line + BLOCK_SIZE - 1) / BLOCK_SIZE).min(if block_mode_192 { 192 } else { 256 }) as u8;

        for line in 0..height as usize {
            self.set_lut_line(
                line,
                LutLine {
                    enabled: true,
                    first_block: 0,
                    last_block: blocks_per_line.saturating_sub(1),
                    line_offset_blocks: (line as u32) * blocks_per_line as u32,
                },
            );
        }
    }

    /// Flush the cache (blocks until complete).
    ///
    /// Not available on N6 (`gfxmmu_n6`): the block has no cache controller.
    #[cfg(gfxmmu_v2)]
    pub fn flush_cache(&mut self) {
        let regs = T::regs();
        regs.ccr().modify(|w| w.set_ff(true));
        while regs.ccr().read().ff() {}
    }

    /// Invalidate the cache (blocks until complete).
    ///
    /// Not available on N6 (`gfxmmu_n6`): the block has no cache controller.
    #[cfg(gfxmmu_v2)]
    pub fn invalidate_cache(&mut self) {
        let regs = T::regs();
        regs.ccr().modify(|w| w.set_fi(true));
        while regs.ccr().read().fi() {}
    }

    /// Read and clear sticky error flags.
    pub fn take_status(&mut self) -> Result<(), Error> {
        let regs = T::regs();
        let sr = regs.sr().read();
        let mut err = Ok(());

        regs.fcr().write(|w| {
            for i in 0..4 {
                if sr.bof(i) {
                    w.set_cbof(i, true);
                    err = Err(Error::BufferOverflow);
                }
            }
            if sr.amef() {
                w.set_camef(true);
                err = Err(Error::MasterError);
            }
        });

        err
    }
}

/// GFXMMU interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let _ = T::regs();
        // Errors are cleared via [`Gfxmmu::take_status`].
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Regs;
}

/// GFXMMU instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// GFXMMU interrupt.
    type Interrupt: Interrupt;
}

foreach_interrupt!(
    ($inst:ident, gfxmmu, GFXMMU, GLOBAL, $irq:ident) => {
        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> Regs {
                crate::pac::$inst
            }
        }
    };
);
