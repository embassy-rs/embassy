//! LTDC - LCD-TFT Display Controller
//! See ST application note AN4861: Introduction to LCD-TFT display controller (LTDC) on STM32 MCUs for high level details
//! This module was tested against the stm32h735g-dk using the RM0468 ST reference manual for detailed register information

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use stm32_metapac::ltdc::regs::Dccr;
use stm32_metapac::ltdc::vals::{Bf1, Bf2, Cfuif, Clif, Crrif, Cterrif, Pf, Vbr};

use crate::gpio::{AfType, OutputType, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::{self};
use crate::{peripherals, rcc, Peri};

static LTDC_WAKER: AtomicWaker = AtomicWaker::new();

/// LTDC error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// FIFO underrun. Generated when a pixel is requested while the FIFO is empty
    FifoUnderrun,
    /// Transfer error. Generated when a bus error occurs
    TransferError,
}

/// Display configuration parameters
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LtdcConfiguration {
    /// Active width in pixels
    pub active_width: u16,
    /// Active height in pixels
    pub active_height: u16,

    /// Horizontal back porch (in units of pixel clock period)
    pub h_back_porch: u16,
    /// Horizontal front porch (in units of pixel clock period)
    pub h_front_porch: u16,
    /// Vertical back porch (in units of horizontal scan line)
    pub v_back_porch: u16,
    /// Vertical front porch (in units of horizontal scan line)
    pub v_front_porch: u16,

    /// Horizontal synchronization width (in units of pixel clock period)
    pub h_sync: u16,
    /// Vertical synchronization height (in units of horizontal scan line)
    pub v_sync: u16,

    /// Horizontal synchronization polarity: `false`: active low, `true`: active high
    pub h_sync_polarity: PolarityActive,
    /// Vertical synchronization polarity: `false`: active low, `true`: active high
    pub v_sync_polarity: PolarityActive,
    /// Data enable polarity: `false`: active low, `true`: active high
    pub data_enable_polarity: PolarityActive,
    /// Pixel clock polarity: `false`: falling edge, `true`: rising edge
    pub pixel_clock_polarity: PolarityEdge,
}

/// Edge polarity
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PolarityEdge {
    /// Falling edge
    FallingEdge,
    /// Rising edge
    RisingEdge,
}

/// Active polarity
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PolarityActive {
    /// Active low
    ActiveLow,
    /// Active high
    ActiveHigh,
}

/// LTDC driver.
pub struct Ltdc<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

/// LTDC interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

/// 24 bit color
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RgbColor {
    /// Red
    pub red: u8,
    /// Green
    pub green: u8,
    /// Blue
    pub blue: u8,
}

/// Layer
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LtdcLayerConfig {
    /// Layer number
    pub layer: LtdcLayer,
    /// Pixel format
    pub pixel_format: PixelFormat,
    /// Window left x in pixels
    pub window_x0: u16,
    /// Window right x in pixels
    pub window_x1: u16,
    /// Window top y in pixels
    pub window_y0: u16,
    /// Window bottom y in pixels
    pub window_y1: u16,
}

/// Pixel format
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PixelFormat {
    /// ARGB8888
    ARGB8888 = Pf::ARGB8888 as u8,
    /// RGB888
    RGB888 = Pf::RGB888 as u8,
    /// RGB565
    RGB565 = Pf::RGB565 as u8,
    /// ARGB1555
    ARGB1555 = Pf::ARGB1555 as u8,
    /// ARGB4444
    ARGB4444 = Pf::ARGB4444 as u8,
    /// L8 (8-bit luminance)
    L8 = Pf::L8 as u8,
    /// AL44 (4-bit alpha, 4-bit luminance
    AL44 = Pf::AL44 as u8,
    /// AL88 (8-bit alpha, 8-bit luminance)
    AL88 = Pf::AL88 as u8,
}

impl PixelFormat {
    /// Number of bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::ARGB8888 => 4,
            PixelFormat::RGB888 => 3,
            PixelFormat::RGB565 | PixelFormat::ARGB4444 | PixelFormat::ARGB1555 | PixelFormat::AL88 => 2,
            PixelFormat::AL44 | PixelFormat::L8 => 1,
        }
    }
}

/// Ltdc Blending Layer
#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LtdcLayer {
    /// Bottom layer
    Layer1 = 0,
    /// Top layer
    Layer2 = 1,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        cortex_m::asm::dsb();
        Ltdc::<T>::enable_interrupts(false);
        LTDC_WAKER.wake();
    }
}

impl<'d, T: Instance> Ltdc<'d, T> {
    // Create a new LTDC driver without specifying color and control pins. This is typically used if you want to drive a display though a DsiHost
    /// Note: Full-Duplex modes are not supported at this time
    pub fn new(peri: Peri<'d, T>) -> Self {
        Self::setup_clocks();
        Self { _peri: peri }
    }

    /// Create a new LTDC driver. 8 pins per color channel for blue, green and red
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_pins(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl ClkPin<T>>,
        hsync: Peri<'d, impl HsyncPin<T>>,
        vsync: Peri<'d, impl VsyncPin<T>>,
        b0: Peri<'d, impl B0Pin<T>>,
        b1: Peri<'d, impl B1Pin<T>>,
        b2: Peri<'d, impl B2Pin<T>>,
        b3: Peri<'d, impl B3Pin<T>>,
        b4: Peri<'d, impl B4Pin<T>>,
        b5: Peri<'d, impl B5Pin<T>>,
        b6: Peri<'d, impl B6Pin<T>>,
        b7: Peri<'d, impl B7Pin<T>>,
        g0: Peri<'d, impl G0Pin<T>>,
        g1: Peri<'d, impl G1Pin<T>>,
        g2: Peri<'d, impl G2Pin<T>>,
        g3: Peri<'d, impl G3Pin<T>>,
        g4: Peri<'d, impl G4Pin<T>>,
        g5: Peri<'d, impl G5Pin<T>>,
        g6: Peri<'d, impl G6Pin<T>>,
        g7: Peri<'d, impl G7Pin<T>>,
        r0: Peri<'d, impl R0Pin<T>>,
        r1: Peri<'d, impl R1Pin<T>>,
        r2: Peri<'d, impl R2Pin<T>>,
        r3: Peri<'d, impl R3Pin<T>>,
        r4: Peri<'d, impl R4Pin<T>>,
        r5: Peri<'d, impl R5Pin<T>>,
        r6: Peri<'d, impl R6Pin<T>>,
        r7: Peri<'d, impl R7Pin<T>>,
    ) -> Self {
        Self::setup_clocks();
        new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(hsync, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(vsync, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b0, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b1, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b2, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b3, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b4, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b5, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b6, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(b7, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g0, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g1, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g2, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g3, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g4, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g5, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g6, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(g7, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r0, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r1, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r2, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r3, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r4, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r5, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r6, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        new_pin!(r7, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        Self { _peri: peri }
    }

    /// Initialise and enable the display
    pub fn init(&mut self, config: &LtdcConfiguration) {
        use stm32_metapac::ltdc::vals::{Depol, Hspol, Pcpol, Vspol};
        let ltdc = T::regs();

        // check bus access
        assert!(ltdc.gcr().read().0 == 0x2220); // reset value

        // configure the HS, VS, DE and PC polarity
        ltdc.gcr().modify(|w| {
            w.set_hspol(match config.h_sync_polarity {
                PolarityActive::ActiveHigh => Hspol::ACTIVE_HIGH,
                PolarityActive::ActiveLow => Hspol::ACTIVE_LOW,
            });

            w.set_vspol(match config.v_sync_polarity {
                PolarityActive::ActiveHigh => Vspol::ACTIVE_HIGH,
                PolarityActive::ActiveLow => Vspol::ACTIVE_LOW,
            });

            w.set_depol(match config.data_enable_polarity {
                PolarityActive::ActiveHigh => Depol::ACTIVE_HIGH,
                PolarityActive::ActiveLow => Depol::ACTIVE_LOW,
            });

            w.set_pcpol(match config.pixel_clock_polarity {
                PolarityEdge::RisingEdge => Pcpol::RISING_EDGE,
                PolarityEdge::FallingEdge => Pcpol::FALLING_EDGE,
            });
        });

        // set synchronization pulse width
        ltdc.sscr().modify(|w| {
            w.set_vsh(config.v_sync - 1);
            w.set_hsw(config.h_sync - 1);
        });

        // set accumulated back porch
        ltdc.bpcr().modify(|w| {
            w.set_avbp(config.v_sync + config.v_back_porch - 1);
            w.set_ahbp(config.h_sync + config.h_back_porch - 1);
        });

        // set accumulated active width
        let aa_height = config.v_sync + config.v_back_porch + config.active_height - 1;
        let aa_width = config.h_sync + config.h_back_porch + config.active_width - 1;
        ltdc.awcr().modify(|w| {
            w.set_aah(aa_height);
            w.set_aaw(aa_width);
        });

        // set total width and height
        let total_height: u16 = config.v_sync + config.v_back_porch + config.active_height + config.v_front_porch - 1;
        let total_width: u16 = config.h_sync + config.h_back_porch + config.active_width + config.h_front_porch - 1;
        ltdc.twcr().modify(|w| {
            w.set_totalh(total_height);
            w.set_totalw(total_width)
        });

        // set the background color value to black
        ltdc.bccr().modify(|w| {
            w.set_bcred(0);
            w.set_bcgreen(0);
            w.set_bcblue(0);
        });

        self.enable();
    }

    /// Set the enable bit in the control register and assert that it has been enabled
    ///
    /// This does need to be called if init has already been called
    pub fn enable(&mut self) {
        T::regs().gcr().modify(|w| w.set_ltdcen(true));
        assert!(T::regs().gcr().read().ltdcen())
    }

    /// Unset the enable bit in the control register and assert that it has been disabled
    pub fn disable(&mut self) {
        T::regs().gcr().modify(|w| w.set_ltdcen(false));
        assert!(!T::regs().gcr().read().ltdcen())
    }

    /// Initialise and enable the layer
    ///
    /// clut - a 256 length color look-up table applies to L8, AL44 and AL88 pixel format and will default to greyscale if `None` supplied and these pixel formats are used
    pub fn init_layer(&mut self, layer_config: &LtdcLayerConfig, clut: Option<&[RgbColor]>) {
        let ltdc = T::regs();
        let layer = ltdc.layer(layer_config.layer as usize);

        // 256 color look-up table for L8, AL88 and AL88 pixel formats
        if let Some(clut) = clut {
            assert_eq!(clut.len(), 256, "Color lookup table must be exactly 256 in length");
            for (index, color) in clut.iter().enumerate() {
                layer.clutwr().write(|w| {
                    w.set_clutadd(index as u8);
                    w.set_red(color.red);
                    w.set_green(color.green);
                    w.set_blue(color.blue);
                });
            }
        }

        // configure the horizontal start and stop position
        let h_win_start = layer_config.window_x0 + ltdc.bpcr().read().ahbp() + 1;
        let h_win_stop = layer_config.window_x1 + ltdc.bpcr().read().ahbp();
        layer.whpcr().write(|w| {
            w.set_whstpos(h_win_start);
            w.set_whsppos(h_win_stop);
        });

        // configure the vertical start and stop position
        let v_win_start = layer_config.window_y0 + ltdc.bpcr().read().avbp() + 1;
        let v_win_stop = layer_config.window_y1 + ltdc.bpcr().read().avbp();
        layer.wvpcr().write(|w| {
            w.set_wvstpos(v_win_start);
            w.set_wvsppos(v_win_stop)
        });

        // set the pixel format
        layer
            .pfcr()
            .write(|w| w.set_pf(Pf::from_bits(layer_config.pixel_format as u8)));

        // set the default color value to transparent black
        layer.dccr().write_value(Dccr::default());

        // set the global constant alpha value
        let alpha = 0xFF;
        layer.cacr().write(|w| w.set_consta(alpha));

        // set the blending factors.
        layer.bfcr().modify(|w| {
            w.set_bf1(Bf1::PIXEL);
            w.set_bf2(Bf2::PIXEL);
        });

        // calculate framebuffer pixel size in bytes
        let bytes_per_pixel = layer_config.pixel_format.bytes_per_pixel() as u16;
        let width = layer_config.window_x1 - layer_config.window_x0;
        let height = layer_config.window_y1 - layer_config.window_y0;

        // framebuffer pitch and line length
        layer.cfblr().modify(|w| {
            w.set_cfbp(width * bytes_per_pixel);
            #[cfg(not(stm32u5))]
            w.set_cfbll(width * bytes_per_pixel + 7);
            #[cfg(stm32u5)]
            w.set_cfbll(width * bytes_per_pixel + 3);
        });

        // framebuffer line number
        layer.cfblnr().modify(|w| w.set_cfblnbr(height));

        // enable LTDC_Layer by setting LEN bit
        layer.cr().modify(|w| {
            if clut.is_some() {
                w.set_cluten(true);
            }
            w.set_len(true);
        });
    }

    /// Set the current buffer. The async function will return when buffer has been completely copied to the LCD screen
    /// frame_buffer_addr is a pointer to memory that should not move (best to make it static)
    pub async fn set_buffer(&mut self, layer: LtdcLayer, frame_buffer_addr: *const ()) -> Result<(), Error> {
        let mut bits = T::regs().isr().read();

        // if all clear
        if !bits.fuif() && !bits.lif() && !bits.rrif() && !bits.terrif() {
            // wait for interrupt
            poll_fn(|cx| {
                // quick check to avoid registration if already done.
                let bits = T::regs().isr().read();
                if bits.fuif() || bits.lif() || bits.rrif() || bits.terrif() {
                    return Poll::Ready(());
                }

                LTDC_WAKER.register(cx.waker());
                Self::clear_interrupt_flags(); // don't poison the request with old flags
                Self::enable_interrupts(true);

                // set the new frame buffer address
                let layer = T::regs().layer(layer as usize);
                layer.cfbar().modify(|w| w.set_cfbadd(frame_buffer_addr as u32));

                // configure a shadow reload for the next blanking period
                T::regs().srcr().write(|w| {
                    w.set_vbr(Vbr::RELOAD);
                });

                // need to check condition after register to avoid a race
                // condition that would result in lost notifications.
                let bits = T::regs().isr().read();
                if bits.fuif() || bits.lif() || bits.rrif() || bits.terrif() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            // re-read the status register after wait.
            bits = T::regs().isr().read();
        }

        let result = if bits.fuif() {
            Err(Error::FifoUnderrun)
        } else if bits.terrif() {
            Err(Error::TransferError)
        } else if bits.lif() {
            panic!("line interrupt event is disabled")
        } else if bits.rrif() {
            // register reload flag is expected
            Ok(())
        } else {
            unreachable!("all interrupt status values checked")
        };

        Self::clear_interrupt_flags();
        result
    }

    fn setup_clocks() {
        critical_section::with(|_cs| {
            // RM says the pllsaidivr should only be changed when pllsai is off. But this could have other unintended side effects. So let's just give it a try like this.
            // According to the debugger, this bit gets set, anyway.
            #[cfg(stm32f7)]
            crate::pac::RCC
                .dckcfgr1()
                .modify(|w| w.set_pllsaidivr(stm32_metapac::rcc::vals::Pllsaidivr::DIV2));

            // It is set to RCC_PLLSAIDIVR_2 in ST's BSP example for the STM32469I-DISCO.
            #[cfg(stm32f4)]
            crate::pac::RCC
                .dckcfgr()
                .modify(|w| w.set_pllsaidivr(stm32_metapac::rcc::vals::Pllsaidivr::DIV2));
        });

        rcc::enable_and_reset::<T>();
    }

    fn clear_interrupt_flags() {
        T::regs().icr().write(|w| {
            w.set_cfuif(Cfuif::CLEAR);
            w.set_clif(Clif::CLEAR);
            w.set_crrif(Crrif::CLEAR);
            w.set_cterrif(Cterrif::CLEAR);
        });
    }

    fn enable_interrupts(enable: bool) {
        T::regs().ier().write(|w| {
            w.set_fuie(enable);
            w.set_lie(false); // we are not interested in the line interrupt enable event
            w.set_rrie(enable);
            w.set_terrie(enable)
        });

        // enable interrupts for LTDC peripheral
        T::Interrupt::unpend();
        if enable {
            unsafe { T::Interrupt::enable() };
        } else {
            T::Interrupt::disable()
        }
    }
}

impl<'d, T: Instance> Drop for Ltdc<'d, T> {
    fn drop(&mut self) {}
}

trait SealedInstance: crate::rcc::SealedRccPeripheral {
    fn regs() -> crate::pac::ltdc::Ltdc;
}

/// LTDC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this LTDC instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(ClkPin, Instance);
pin_trait!(HsyncPin, Instance);
pin_trait!(VsyncPin, Instance);
pin_trait!(DePin, Instance);
pin_trait!(R0Pin, Instance);
pin_trait!(R1Pin, Instance);
pin_trait!(R2Pin, Instance);
pin_trait!(R3Pin, Instance);
pin_trait!(R4Pin, Instance);
pin_trait!(R5Pin, Instance);
pin_trait!(R6Pin, Instance);
pin_trait!(R7Pin, Instance);
pin_trait!(G0Pin, Instance);
pin_trait!(G1Pin, Instance);
pin_trait!(G2Pin, Instance);
pin_trait!(G3Pin, Instance);
pin_trait!(G4Pin, Instance);
pin_trait!(G5Pin, Instance);
pin_trait!(G6Pin, Instance);
pin_trait!(G7Pin, Instance);
pin_trait!(B0Pin, Instance);
pin_trait!(B1Pin, Instance);
pin_trait!(B2Pin, Instance);
pin_trait!(B3Pin, Instance);
pin_trait!(B4Pin, Instance);
pin_trait!(B5Pin, Instance);
pin_trait!(B6Pin, Instance);
pin_trait!(B7Pin, Instance);

foreach_interrupt!(
    ($inst:ident, ltdc, LTDC, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::ltdc::Ltdc {
                crate::pac::$inst
            }
        }
    };
);
