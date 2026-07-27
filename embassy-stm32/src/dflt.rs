//! Shared configuration types for ADF and MDF digital filter peripherals.

pub use crate::pac::adf::vals::{
    Acqmod, Bssel, Cckdir, Cckdiv, Ccken, Cicmod, Ckgmod, Datsrc, Rxfifo, Scksrc, Sitfmod,
};

/// Clock generator configuration shared by ADF and MDF.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClockConfig {
    /// Divider applied to the kernel clock to produce the processing clock.
    ///
    /// Output frequency is `kernel_clock / (procdiv + 1)`.
    pub procdiv: u8,
    /// Divider applied to the CCK output clock.
    pub cckdiv: Cckdiv,
    /// Drive CCK0 as output (typically the PDM bit clock).
    pub cck0_output: bool,
    /// Drive CCK1 as output.
    pub cck1_output: bool,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            procdiv: 0,
            cckdiv: Cckdiv::Div1,
            cck0_output: true,
            cck1_output: false,
        }
    }
}

/// Serial interface (SITF) configuration.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SitfConfig {
    /// Serial interface mode.
    pub mode: Sitfmod,
    /// Serial clock source.
    pub clock_source: Scksrc,
    /// Manchester/SPI threshold (must be >= 4 in SPI mode).
    pub threshold: u8,
}

impl Default for SitfConfig {
    fn default() -> Self {
        Self {
            mode: Sitfmod::NormalSpi,
            clock_source: Scksrc::Cck0,
            threshold: 4,
        }
    }
}

/// Digital filter configuration.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FilterConfig {
    /// Bitstream routed into the filter.
    pub bitstream: Bssel,
    /// CIC decimation ratio: actual ratio is `cic_decimation + 1`, minimum 2.
    pub cic_decimation: u16,
    /// CIC output gain / shift (see RM Table "Possible gain values").
    pub scale: u8,
    /// Acquisition mode.
    pub acquisition_mode: Acqmod,
    /// RX FIFO threshold for DMA requests.
    pub fifo_threshold: Rxfifo,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            bitstream: Bssel::Bs0R,
            cic_decimation: 31,
            scale: 0b100000,
            acquisition_mode: Acqmod::AsynchronousContinuous,
            fifo_threshold: Rxfifo::HalfFull,
        }
    }
}

/// Extract a signed 24-bit sample from a DMA word read from DFLTDR.
#[inline]
pub fn sample_from_dma_word(word: u32) -> i32 {
    let raw = (word >> 8) & 0x00FF_FFFF;
    if raw & 0x0080_0000 != 0 {
        (raw | 0xFF00_0000) as i32
    } else {
        raw as i32
    }
}

/// Convert raw DMA words into signed PCM samples.
#[inline]
pub fn samples_from_dma_words(raw: &[u32], out: &mut [i32]) {
    for (sample, word) in out.iter_mut().zip(raw.iter()) {
        *sample = sample_from_dma_word(*word);
    }
}
