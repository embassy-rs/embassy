//! S/PDIF receiver
#![macro_use]
#![cfg_attr(gpdma, allow(unused))]

use core::marker::PhantomData;

use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::ringbuffer::Error as RingbufferError;
pub use crate::dma::word;
#[cfg(not(gpdma))]
use crate::dma::ReadableRingBuffer;
use crate::dma::{Channel, TransferOptions};
use crate::gpio::{AfType, AnyPin, Pull, SealedPin as _};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::spdifrx::Spdifrx as Regs;
use crate::rcc::{RccInfo, SealedRccPeripheral};
use crate::{interrupt, peripherals, Peri};

/// Possible S/PDIF preamble types.
#[allow(dead_code)]
#[repr(u8)]
enum PreambleType {
    Unused = 0x00,
    /// The preamble changes to preamble “B” once every 192 frames to identify the start of the block structure used to
    /// organize the channel status and user information.
    B = 0x01,
    /// The first sub-frame (left or “A” channel in stereophonic operation and primary channel in monophonic operation)
    /// normally starts with preamble “M”
    M = 0x02,
    /// The second sub-frame (right or “B” channel in stereophonic operation and secondary channel in monophonic
    /// operation) always starts with preamble “W”.
    W = 0x03,
}

macro_rules! new_spdifrx_pin {
    ($name:ident, $af_type:expr) => {{
        let pin = $name;
        let input_sel = pin.input_sel();
        pin.set_as_af(pin.af_num(), $af_type);
        (Some(pin.into()), input_sel)
    }};
}

macro_rules! impl_spdifrx_pin {
    ($inst:ident, $pin:ident, $af:expr, $sel:expr) => {
        impl crate::spdifrx::InPin<peripherals::$inst> for crate::peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
            fn input_sel(&self) -> u8 {
                $sel
            }
        }
    };
}

/// Ring-buffered SPDIFRX driver.
///
/// Data is read by DMAs and stored in a ring buffer.
#[cfg(not(gpdma))]
pub struct Spdifrx<'d, T: Instance> {
    _peri: Peri<'d, T>,
    spdifrx_in: Option<Peri<'d, AnyPin>>,
    data_ring_buffer: ReadableRingBuffer<'d, u32>,
}

/// Gives the address of the data register.
fn dr_address(r: Regs) -> *mut u32 {
    #[cfg(spdifrx_v1)]
    let address = r.dr().as_ptr() as _;
    #[cfg(spdifrx_h7)]
    let address = r.fmt0_dr().as_ptr() as _; // All fmtx_dr() implementations have the same address.

    return address;
}

/// Gives the address of the channel status register.
#[allow(unused)]
fn csr_address(r: Regs) -> *mut u32 {
    r.csr().as_ptr() as _
}

/// Select the channel for capturing control information.
pub enum ControlChannelSelection {
    /// Capture control info from channel A.
    A,
    /// Capture control info from channel B.
    B,
}

/// Configuration options for the SPDIFRX driver.
pub struct Config {
    /// Select the channel for capturing control information.
    pub control_channel_selection: ControlChannelSelection,
}

/// S/PDIF errors.
#[derive(Debug)]
pub enum Error {
    /// DMA overrun error.
    RingbufferError(RingbufferError),
    /// Left/right channel synchronization error.
    ChannelSyncError,
}

impl From<RingbufferError> for Error {
    fn from(error: RingbufferError) -> Self {
        Self::RingbufferError(error)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            control_channel_selection: ControlChannelSelection::A,
        }
    }
}

#[cfg(not(gpdma))]
impl<'d, T: Instance> Spdifrx<'d, T> {
    fn dma_opts() -> TransferOptions {
        TransferOptions {
            half_transfer_ir: true,
            // new_write() and new_read() always use circular mode
            ..Default::default()
        }
    }

    /// Create a new `Spdifrx` instance.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::GlobalInterrupt, GlobalInterruptHandler<T>> + 'd,
        config: Config,
        spdifrx_in: Peri<'d, impl InPin<T>>,
        data_dma: Peri<'d, impl Channel + Dma<T>>,
        data_dma_buf: &'d mut [u32],
    ) -> Self {
        let (spdifrx_in, input_sel) = new_spdifrx_pin!(spdifrx_in, AfType::input(Pull::None));
        Self::setup(config, input_sel);

        let regs = T::info().regs;
        let dr_request = data_dma.request();
        let dr_ring_buffer =
            unsafe { ReadableRingBuffer::new(data_dma, dr_request, dr_address(regs), data_dma_buf, Self::dma_opts()) };

        Self {
            _peri: peri,
            spdifrx_in,
            data_ring_buffer: dr_ring_buffer,
        }
    }

    fn setup(config: Config, input_sel: u8) {
        T::info().rcc.enable_and_reset();
        T::GlobalInterrupt::unpend();
        unsafe { T::GlobalInterrupt::enable() };

        let regs = T::info().regs;

        regs.imr().write(|imr| {
            imr.set_ifeie(true); // Enables interrupts for TERR, SERR, FERR.
            imr.set_syncdie(true); // Enables SYNCD interrupt.
        });

        regs.cr().write(|cr| {
            cr.set_spdifen(0x00); // Disable SPDIF receiver synchronization.
            cr.set_rxdmaen(true); // Use RX DMA for data. Enabled on `read`.
            cr.set_cbdmaen(false); // Do not capture channel info.
            cr.set_rxsteo(true); // Operate in stereo mode.
            cr.set_drfmt(0x01); // Data is left-aligned (MSB).

            // Disable all status fields in the data register.
            // Status can be obtained directly with the status register DMA.
            cr.set_pmsk(false); // Write parity bit to the data register. FIXME: Add parity check.
            cr.set_vmsk(false); // Write validity to the data register.
            cr.set_cumsk(true); // Do not write C and U bits to the data register.
            cr.set_ptmsk(false); // Write preamble bits to the data register.

            cr.set_chsel(match config.control_channel_selection {
                ControlChannelSelection::A => false,
                ControlChannelSelection::B => true,
            }); // Select channel status source.

            cr.set_nbtr(0x02); // 16 attempts are allowed.
            cr.set_wfa(true); // Wait for activity before going to synchronization phase.
            cr.set_insel(input_sel); // Input pin selection.

            #[cfg(stm32h7)]
            cr.set_cksen(true); // Generate a symbol clock.

            #[cfg(stm32h7)]
            cr.set_cksbkpen(true); // Generate a backup symbol clock.
        });
    }

    /// Start the SPDIFRX driver.
    pub fn start(&mut self) {
        self.data_ring_buffer.start();

        T::info().regs.cr().modify(|cr| {
            cr.set_spdifen(0x03); // Enable S/PDIF receiver.
        });
    }

    /// Read from the SPDIFRX data ring buffer.
    ///
    /// SPDIFRX is always receiving data in the background. This function pops already-received
    /// data from the buffer.
    ///
    /// If there's less than `data.len()` data in the buffer, this waits until there is.
    pub async fn read(&mut self, data: &mut [u32]) -> Result<(), Error> {
        self.data_ring_buffer.read_exact(data).await?;

        let first_preamble = (data[0] >> 4) & 0b11_u32;
        if (first_preamble as u8) == (PreambleType::W as u8) {
            trace!("S/PDIF left/right mismatch");

            // Resynchronize until the first sample is for the left channel.
            self.data_ring_buffer.clear();
            return Err(Error::ChannelSyncError);
        };

        for sample in data.as_mut() {
            if (*sample & (0x0002_u32)) == 0x0001 {
                // Discard invalid samples, setting them to mute level.
                *sample = 0;
            } else {
                // Discard status information in the lowest byte.
                *sample &= 0xFFFFFF00;
            }
        }

        Ok(())
    }
}

#[cfg(not(gpdma))]
impl<'d, T: Instance> Drop for Spdifrx<'d, T> {
    fn drop(&mut self) {
        T::info().regs.cr().modify(|cr| cr.set_spdifen(0x00));
        self.spdifrx_in.as_ref().map(|x| x.set_as_disconnected());
    }
}

struct State {
    #[allow(unused)]
    waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

struct Info {
    regs: crate::pac::spdifrx::Spdifrx,
    rcc: RccInfo,
}

peri_trait!(
    irqs: [GlobalInterrupt],
);

/// SPIDFRX pin trait
pub trait InPin<T: Instance>: crate::gpio::Pin {
    /// Get the GPIO AF number needed to use this pin.
    fn af_num(&self) -> u8;
    /// Get the SPIDFRX INSEL number needed to use this pin.
    fn input_sel(&self) -> u8;
}

dma_trait!(Dma, Instance);

/// Global interrupt handler.
pub struct GlobalInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::GlobalInterrupt> for GlobalInterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::state().waker.wake();

        let regs = T::info().regs;
        let sr = regs.sr().read();

        if sr.serr() || sr.terr() || sr.ferr() {
            trace!("SPDIFRX error, resync");

            // Clear errors by disabling SPDIFRX, then reenable.
            regs.cr().modify(|cr| cr.set_spdifen(0x00));
            regs.cr().modify(|cr| cr.set_spdifen(0x03));
        } else if sr.syncd() {
            // Synchronization was successful.
            trace!("SPDIFRX sync success");
        }

        // Clear interrupt flags.
        regs.ifcr().write(|ifcr| {
            ifcr.set_perrcf(true); // Clears parity error flag.
            ifcr.set_ovrcf(true); // Clears overrun error flag.
            ifcr.set_sbdcf(true); // Clears synchronization block detected flag.
            ifcr.set_syncdcf(true); // Clears SYNCD from SR (was read above).
        });
    }
}

foreach_peripheral!(
    (spdifrx, $inst:ident) => {
        #[allow(private_interfaces)]
        impl SealedInstance for peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info{
                    regs: crate::pac::$inst,
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {
            type GlobalInterrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
);
