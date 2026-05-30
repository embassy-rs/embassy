//! Filter Math Accelerator

mod from_adc;

pub use dsp_fixedpoint::Q16;
use embassy_hal_internal::{Peri, PeripheralType};
pub use from_adc::FromAdc;

use crate::{peripherals, rcc};

/// FMAC driver
pub struct Fmac<'d, T: Instance> {
    peri: Peri<'d, T>,
}

/// FMAC instance
trait SealedInstance {
    /// Get access to FMAC registers
    fn regs() -> crate::pac::fmac::Fmac;

    /// Read value from RDATA
    fn read_result(&self) -> u16 {
        Self::regs().rdata().read().res()
    }

    fn read_q16(&self) -> Q16<15> {
        Q16::new(self.read_result() as i16)
    }

    fn write_input(&self, x: u16) {
        Self::regs().wdata().write(|w| w.set_wdata(x));
    }

    fn write_q16(&self, x: Q16<15>) {
        self.write_input(x.inner as u16);
    }

    fn rdata() -> *const u32 {
        Self::regs().rdata().as_ptr() as *const u32
    }

    fn wdata() -> *mut u32 {
        Self::regs().wdata().as_ptr() as *mut u32
    }
}

/// FMAC instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral {}

/// Output mode
///
/// Wrapping vs Saturating
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OutputMode {
    /// Wrap result when it does not fit into a Q1.15
    Wrapping,
    /// Saturate result when it does not fit into a Q1.15
    Saturating,
}

/// Set of values (input or output) that can be either prefilled or not
pub enum Data<'a> {
    /// Prefill with values
    Prefilled {
        /// Prefill data. The capacity of the buffer will be set to the length of this data
        data: &'a [Q16<15>],
    },
    /// Do not prefill. Output may be delayed until data is filled the regular way
    Empty {
        /// Capacity of buffer
        capacity: usize,
    },
}

impl<'a> Data<'a> {
    fn capacity(&self) -> usize {
        match self {
            Data::Prefilled { data } => data.len(),
            Data::Empty { capacity } => *capacity,
        }
    }
}

enum Func {
    /// Preload X1 values (inputs)
    LoadX1 = 1,
    /// Load X2 values (parameters)
    LoadX2 = 2,
    /// Preload Y values (outputs)
    LoadY = 3,
    /// Convolution/FIR
    ///
    ///  
    Fir = 8,
    Iir = 9,
}

/// Read/write method for output and input
#[derive(Clone, Debug, PartialEq)]
pub enum AccessMethod {
    /// Polled access
    Poll,
    /// Interrupt
    Interrupt,
    /// DMA
    Dma,
}

/// Gain
#[derive(Copy, Clone, Debug)]
pub enum Gain {
    /// 1x gain
    X1,
    /// 2x gain
    X2,
    /// 4x gain
    X4,
    /// 8x gain
    X8,
    /// 16x gain
    X16,
    /// 32x gain
    X32,
    /// 64x gain
    X64,
    /// 128x gain
    X128,
}

/// FMAC configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// Set output clamping behaviour
    pub output_mode: OutputMode,
    /// Set how the output will be read
    pub read_method: AccessMethod,
    /// Set how the input will be written
    pub write_method: AccessMethod,
}

/// The input buffer is full
#[derive(Clone, Debug)]
pub struct ErrorInputFull;

impl<'d, T: Instance> Fmac<'d, T> {
    fn new(
        peri: Peri<'d, T>,
        config: Config,
        x1: Option<&[Q16<15>]>,
        x2_ff: &[Q16<15>],
        x2_fb: &[Q16<15>],
        y: Data,
    ) -> Self {
        assert!(2 * (x2_ff.len() + x2_fb.len()) + y.capacity() <= 256);
        if let Some(data) = x1 {
            assert_eq!(data.len(), x2_ff.len() + x2_fb.len());
        }
        rcc::enable_and_reset::<T>();

        let mut this = Self { peri };

        // TODO: Consider adding watermark support. If so {x1,x2,y}_buf_size will need to be enlarged
        let x2_len = x2_ff.len() as u8 + x2_fb.len() as u8;
        let x1_len = x2_len;
        let y_len = y.capacity() as u8;

        T::regs().x1bufcfg().write(|w| {
            w.set_full_wm(0); // 0: Threshold = 1 // Must be 0 when using DMA to write X1
            w.set_x1_base(0);
            w.set_x1_buf_size(x1_len);
        });

        T::regs().x2bufcfg().write(|w| {
            w.set_x2_base(x1_len);
            w.set_x2_buf_size(x2_len);
        });

        T::regs().ybufcfg().write(|w| {
            w.set_empty_wm(0); // 0: Threshold = 1 // Must be 0 when using DMA to read Y
            w.set_y_base(x1_len + x2_len);
            w.set_y_buf_size(y_len);
        });

        if let Some(data) = x1 {
            this.load(Func::LoadX1, data, &[]);
        }

        this.load(Func::LoadX2, x2_ff, x2_fb);

        if let Data::Prefilled { data } = y {
            this.load(Func::LoadY, data, &[]);
        }

        T::regs().cr().write(|w| {
            w.set_clipen(config.output_mode == OutputMode::Saturating);
            w.set_dmaren(config.read_method == AccessMethod::Dma);
            w.set_dmawen(config.write_method == AccessMethod::Dma);
            w.set_rien(config.read_method == AccessMethod::Interrupt);
            w.set_wien(config.write_method == AccessMethod::Interrupt);
            w.set_satien(false);
            w.set_ovflien(false);
            w.set_unflien(false);
        });

        this
    }

    /// Convolution/FIR filter
    ///
    /// y = gain * (feedforward ⋅ input)
    ///
    /// Note: The 0th elements are the newest, and the n-1nth is the oldest
    pub fn fir(
        peri: Peri<'d, T>,
        config: Config,
        input: Option<&[Q16<15>]>,
        feedforward: &[Q16<15>],
        gain: Gain,
    ) -> Self {
        let p = feedforward.len() as u8;
        let mut this = Self::new(peri, config, input, feedforward, &[], Data::Empty { capacity: 1 });
        this.func(Func::Fir, p, 0, gain as u8);
        this
    }

    /// IIR filter
    ///
    /// y = gain * (feedforward ⋅ input) + (feedback ⋅ previous_outputs)
    ///
    /// Note: The 0th elements are the newest, and the n-1nth is the oldest
    pub fn iir(
        peri: Peri<'d, T>,
        config: Config,
        input: Option<&[Q16<15>]>,
        feedforward: &[Q16<15>],
        feedback: &[Q16<15>],
        output: Data,
        gain: Gain,
    ) -> Self {
        assert!(feedback.len() < feedforward.len());
        let mut this = Self::new(peri, config, input, feedforward, feedback, output);
        this.func(Func::Iir, feedforward.len() as u8, feedback.len() as u8, gain as u8);
        this
    }

    /// 2p2z controller
    pub fn controller_2p2z(
        peri: Peri<'d, T>,
        config: Config,
        input: Option<&[Q16<15>; 3]>,
        a: [Q16<15>; 2],
        b: [Q16<15>; 3],
        output: Option<[Q16<15>; 2]>,
        gain: Gain,
    ) -> Self {
        // TODO: Invert a?
        let output = match &output {
            Some(data) => Data::Prefilled { data },
            None => Data::Empty { capacity: 2 },
        };
        Self::iir(peri, config, input.map(|x| &x[..]), &b, &a, output, gain)
    }

    /// 3p3z controller
    pub fn controller_3p3z(
        peri: Peri<'d, T>,
        config: Config,
        input: Option<&[Q16<15>; 4]>,
        a: [Q16<15>; 3],
        b: [Q16<15>; 4],
        output: Option<[Q16<15>; 3]>,
        gain: Gain,
    ) -> Self {
        let output = match &output {
            Some(data) => Data::Prefilled { data },
            None => Data::Empty { capacity: 3 },
        };
        Self::iir(peri, config, input.map(|x| &x[..]), &b, &a, output, gain)
    }

    fn func(&mut self, func: Func, p: u8, q: u8, r: u8) {
        debug_assert!(!T::regs().param().read().start());
        T::regs().param().write(|w| {
            w.set_func(func as u8);
            w.set_p(p);
            w.set_q(q);
            w.set_r(r);
            w.set_start(true);
        });
    }

    // Assumes lengths are correct
    fn load(&mut self, func: Func, d_n: &[Q16<15>], d_m: &[Q16<15>]) {
        self.func(func, d_n.len() as u8, d_m.len() as u8, 0);
        for x in d_n {
            self.peri.write_q16(*x);
        }
        for x in d_m {
            self.peri.write_q16(*x);
        }
        debug_assert!(!T::regs().param().read().start());
    }

    /// Read output value
    pub fn read(&mut self) -> Option<Q16<15>> {
        match T::regs().sr().read().yempty() {
            true => None,
            false => Some(self.peri.read_q16()),
        }
    }

    /// Write input
    pub fn write(&mut self, x: Q16<15>) -> Result<(), ErrorInputFull> {
        match T::regs().sr().read().x1full() {
            true => Err(ErrorInputFull),
            false => {
                self.peri.write_q16(x);
                Ok(())
            }
        }
    }
}

foreach_interrupt!(
    ($inst:ident, fmac, $block:ident, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::fmac::Fmac {
                crate::pac::$inst
            }
        }
    };
);
