//! S/PDIF receiver
#![macro_use]
#![cfg_attr(gpdma, allow(unused))]

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::ringbuffer::OverrunError;
pub use crate::dma::word;
#[cfg(not(gpdma))]
use crate::dma::ReadableRingBuffer;
use crate::dma::{Channel, TransferOptions};
use crate::gpio::{AfType, AnyPin, Pull, SealedPin as _};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::spdifrx::Spdifrx as Regs;
use crate::rcc::{RccInfo, SealedRccPeripheral};
use crate::{interrupt, peripherals, Peripheral};

macro_rules! new_spdif_pin {
    ($name:ident, $af_type:expr) => {{
        let pin = $name.into_ref();
        let input_sel = pin.input_sel();
        pin.set_as_af(pin.af_num(), $af_type);
        (Some(pin.map_into()), input_sel)
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
/// Data and, optionally, channel status information are read by DMAs and stored in ring buffers.
pub struct Spdifrx<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
    spdifrx_in: Option<PeripheralRef<'d, AnyPin>>,
    data_ring_buffer: ReadableRingBuffer<'d, u32>,
    channel_status_ring_buffer: Option<ReadableRingBuffer<'d, u32>>,
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

    /// Create a new `Spdifrx` instance with only data readout.
    pub fn new_data_only(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::GlobalInterrupt, GlobalInterruptHandler<T>> + 'd,
        config: Config,
        spdifrx_in: impl Peripheral<P = impl InPin<T>> + 'd,
        data_dma: impl Peripheral<P = impl Channel + Dma<T>> + 'd,
        data_dma_buf: &'d mut [u32],
    ) -> Self {
        let (spdifrx_in, input_sel) = new_spdif_pin!(spdifrx_in, AfType::input(Pull::None));
        Self::setup(false, config, input_sel);

        into_ref!(peri, data_dma);

        let regs = T::info().regs;
        let dr_request = data_dma.request();
        let dr_ring_buffer =
            unsafe { ReadableRingBuffer::new(data_dma, dr_request, dr_address(regs), data_dma_buf, Self::dma_opts()) };

        Self {
            _peri: peri,
            spdifrx_in,
            data_ring_buffer: dr_ring_buffer,
            channel_status_ring_buffer: None,
        }
    }

    /// Create a new `Spdifrx` instance with data and channel-status-register readout.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::GlobalInterrupt, GlobalInterruptHandler<T>> + 'd,
        config: Config,
        spdifrx_in: impl Peripheral<P = impl InPin<T>> + 'd,
        data_dma: impl Peripheral<P = impl Channel + Dma<T>> + 'd,
        data_dma_buf: &'d mut [u32],
        channel_status_dma: impl Peripheral<P = impl Channel + Dma<T>> + 'd,
        channel_status_dma_buf: &'d mut [u32],
    ) -> Self {
        let (spdifrx_in, input_sel) = new_spdif_pin!(spdifrx_in, AfType::input(Pull::None));
        Self::setup(true, config, input_sel);

        into_ref!(peri, data_dma, channel_status_dma);

        let regs = T::info().regs;
        let dr_request = data_dma.request();
        let dr_ring_buffer =
            unsafe { ReadableRingBuffer::new(data_dma, dr_request, dr_address(regs), data_dma_buf, Self::dma_opts()) };

        let csr_request = channel_status_dma.request();
        let csr_ring_buffer = unsafe {
            ReadableRingBuffer::new(
                channel_status_dma,
                csr_request,
                csr_address(regs),
                channel_status_dma_buf,
                Self::dma_opts(),
            )
        };

        Self {
            _peri: peri,
            spdifrx_in,
            data_ring_buffer: dr_ring_buffer,
            channel_status_ring_buffer: Some(csr_ring_buffer),
        }
    }

    fn setup(read_channel_info: bool, config: Config, in_sel: u8) {
        T::info().rcc.enable_and_reset();
        T::GlobalInterrupt::unpend();
        unsafe { T::GlobalInterrupt::enable() };

        let regs = T::info().regs;

        regs.imr().write(|imr| {
            imr.set_ifeie(true); // Enables interrupts for TERR, SERR, FERR.
            imr.set_syncdie(true); // Enables SYNCD interrupt.
        });

        regs.cr().write(|cr| {
            cr.set_spdifen(0x01); // Enable SPDIF receiver synchronization.
            cr.set_rxdmaen(true); // Use RX DMA for data.
            cr.set_cbdmaen(read_channel_info); // Use RX DMA for channel info.
            cr.set_rxsteo(true); // Operate in stereo mode.
            cr.set_drfmt(0x01); // Data is left-aligned (MSB).

            // Disable all status fields in the data register.
            // Status can be obtained directly with the status register DMA.
            cr.set_pmsk(false); // Write parity error bit to the data register.
            cr.set_vmsk(false); // Write validity to the data register.
            cr.set_cumsk(false); // C and U bits are written to the data register.
            cr.set_ptmsk(false); // Preamble bits are written to the data register.

            cr.set_chsel(match config.control_channel_selection {
                ControlChannelSelection::A => false,
                ControlChannelSelection::B => true,
            }); // Channel status is read from sub-frame A.

            cr.set_nbtr(0x02); // 16 attempts are allowed.
            cr.set_wfa(true); // Wait for activity before going to synchronization phase.
            cr.set_insel(in_sel); // Input pin selection.
            cr.set_cksen(true); // Generate a symbol clock.
            cr.set_cksbkpen(true); // Do not generate a backup symbol clock.
        });
    }

    /// Start the SPDIFRX driver.
    pub fn start(&mut self) {
        self.data_ring_buffer.start();

        if let Some(csr_ring_buffer) = self.channel_status_ring_buffer.as_mut() {
            csr_ring_buffer.start();
        }
    }

    /// Read from the SPDIFRX data ring buffer.
    ///
    /// The peripheral is configured not to store any channel information in the data register.
    /// Therefore, the upper 24 bit are audio sample information, and the lower 8 bit are always zero.
    ///
    /// SPDIFRX is always receiving data in the background. This function pops already-received
    /// data from the buffer.
    ///
    /// If there's less than `data.len()` data in the buffer, this waits until there is.
    pub async fn read_data(&mut self, data: &mut [u32]) -> Result<usize, OverrunError> {
        self.data_ring_buffer.read_exact(data).await
    }

    /// Read from the SPDIFRX channel status ring buffer.
    ///
    /// Panics, if the instance is not configured for reception of the channel status.
    ///
    /// SPDIFRX is always receiving channel status updates in the background. This function pops already-received
    /// data from the buffer.
    ///
    /// If there's less than `data.len()` data in the buffer, this waits until there is.
    pub async fn read_channel_status(&mut self, data: &mut [u32]) -> Result<usize, OverrunError> {
        self.channel_status_ring_buffer.as_mut().unwrap().read_exact(data).await
    }
}

impl<'d, T: Instance> Drop for Spdifrx<'d, T> {
    fn drop(&mut self) {
        T::GlobalInterrupt::disable();

        let regs = T::info().regs;
        regs.cr().modify(|cr| cr.set_spdifen(0x00));
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
        let regs = T::info().regs;
        T::state().waker.wake();

        critical_section::with(|_| {
            let sr = regs.sr().read();
            if sr.serr() || sr.terr() || sr.ferr() {
                // Clear errors by disabling SPDIFRX.
                regs.cr().modify(|cr| cr.set_spdifen(0x00));

                // Attempt to synchronize again.
                regs.cr().modify(|cr| cr.set_spdifen(0x01));
            } else if sr.syncd() {
                // Synchronization was successful, now enable SPDIFRX.
                regs.cr().modify(|cr| cr.set_spdifen(0x3));
            }

            // Clear interrupt flags.
            regs.ifcr().write(|ifcr| {
                ifcr.set_perrcf(true); // Clears parity error flag.
                ifcr.set_ovrcf(true); // Clears overrun error flag.
                ifcr.set_sbdcf(true); // Clears synchronization block detected flag.
                ifcr.set_syncdcf(true); // Clears SYNCD from SR (was read above).
            });
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
