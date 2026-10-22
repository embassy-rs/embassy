//! Ethernet (ETH)
#![macro_use]

#[cfg(all(feature = "ptp", eth_v1a))]
compile_error!("The 'ptp' feature is not supported on STM32 Ethernet MAC v1a.");

#[cfg_attr(any(eth_v1a, eth_v1b, eth_v1c), path = "v1/mod.rs")]
#[cfg_attr(any(eth_v2, eth_v2a, eth_v2b), path = "v2/mod.rs")]
mod _version;
mod generic_phy;
mod ring;
mod sma;

use core::mem::MaybeUninit;
use core::task::{Context, Waker};

use embassy_sync::waitqueue::AtomicWaker;
#[cfg(feature = "ptp")]
use heapless::Deque;
#[cfg(feature = "ptp")]
use xarxa_driver::TxTimestamp;
use xarxa_driver::{Capabilities, Driver, HardwareAddress, LinkState, Medium, NotSupported, PacketBuf};

pub use crate::eth::_version::{InterruptHandler, *};
pub use crate::eth::generic_phy::*;
pub use crate::eth::sma::{Instance as SmaInstance, Sma, StationManagement};
use crate::pac::eth::Eth as Regs;

/// Failure to initialize or access the Ethernet MAC or PHY.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The MAC DMA software reset did not complete.
    DmaResetTimeout,
    /// An MDIO transaction did not complete.
    MdioTimeout,
    /// No MDIO address completed a PHY software reset.
    PhyNotFound,
    /// The PHY did not complete its software reset.
    PhyResetTimeout,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::DmaResetTimeout => "MAC DMA software reset timed out",
            Self::MdioTimeout => "MDIO transaction timed out",
            Self::PhyNotFound => "no MDIO address completed a PHY software reset",
            Self::PhyResetTimeout => "PHY software reset timed out",
        })
    }
}

impl core::error::Error for Error {}

// ST's F4/H7 HAL gives Ethernet DMA reset 500 ms. Use the same limit so
// a slow reset is not rejected before the vendor driver would time out.
pub(crate) const MAC_RESET_TIMEOUT_US: u64 = 500_000;

// IEEE 802.3 allows a PHY reset to take up to 500 ms.
pub(crate) const PHY_RESET_TIMEOUT_US: u64 = 500_000;

// DMA shutdown has no universal completion time across MACs and traffic loads.
// The 100 ms wait is a failure budget: if it expires, Drop must not release
// buffers the DMA might still own.
pub(crate) const DMA_STOP_TIMEOUT_US: u64 = 100_000;

// With `time` the deadline is a real clock. Without it the budget is a cycle count, which
// depends on the core and can run shorter than `timeout_us`.
pub(crate) fn wait_for(mut ready: impl FnMut() -> bool, timeout_us: u64) -> bool {
    if ready() {
        return true;
    }
    #[cfg(feature = "time")]
    {
        let timeout = crate::wait::Timeout::from_micros(timeout_us);
        while timeout.check().is_ok() {
            if ready() {
                return true;
            }
        }
        ready()
    }
    #[cfg(not(feature = "time"))]
    {
        // SAFETY: Ethernet operations run after RCC initializes CLOCK_FREQS; the
        // configured system clock is not changed during this synchronous wait.
        let sys_hz = unsafe { crate::rcc::get_freqs().sys.to_hertz().unwrap().0 };
        let cycles_per_us = sys_hz.div_ceil(1_000_000).max(1);
        for _ in 0..timeout_us {
            cortex_m::asm::delay(cycles_per_us);
            if ready() {
                return true;
            }
        }
        false
    }
}

#[cfg(feature = "ptp")]
fn adjusted_ptp_addend(nominal: u32, adjustment: embassy_ptp_driver::ScaledPpm) -> u32 {
    let scale = 1.0 + f64::from(adjustment.raw()) / ((1i32 << 16) as f64 * 1e6);
    // The cast saturates; the addend must remain nonzero.
    ((f64::from(nominal) * scale + 0.5) as u32).max(1)
}

/// Maximum Ethernet frame size, header included, FCS excluded.
#[cfg(not(any(feature = "eth-2k-frames", feature = "eth-jumbo-frames")))]
const MTU: usize = 1514;
#[cfg(all(feature = "eth-2k-frames", not(feature = "eth-jumbo-frames")))]
const MTU: usize = 1996;
#[cfg(feature = "eth-jumbo-frames")]
const MTU: usize = 9014;

#[cfg(all(
    any(feature = "eth-2k-frames", feature = "eth-jumbo-frames"),
    any(eth_v1a, eth_v1b, eth_v1c)
))]
compile_error!("`eth-2k-frames` and `eth-jumbo-frames` need the v2 Ethernet MAC (STM32H5, H7, N6, C5).");

/// Ethernet descriptor rings.
///
/// This struct owns the DMA descriptors of the transmit and receive rings.
/// The frames themselves live in `embassy-net`'s packet buffer pool: the
/// receive ring holds one buffer per descriptor, filled by DMA in place, and
/// the transmit ring holds each frame's buffer until the hardware is done
/// with it.
///
/// `TX` is the number of descriptors in the transmit ring, `RX` in the receive
/// ring. A bigger ring allows the hardware to receive more frames while the
/// CPU is busy doing other things, which may increase performance (especially
/// for RX), at the cost of pinning more packet buffers. Make sure the packet
/// pool is bigger than `TX + RX`, with room to spare for the stack and sockets.
/// The pool's buffers must be 8-byte aligned, and the DMA uses their size
/// rounded down to a multiple of 8. To receive full 1514-byte frames, use
/// `StaticPool<1520, N, 8>`. With `eth-2k-frames` (1996-byte frames) use
/// `StaticPool<2000, N, 8>`, and with `eth-jumbo-frames` (9014-byte frames)
/// `StaticPool<9016, N, 8>`.
/// The v2 driver reserves one descriptor in each ring as a DMA tail guard.
/// It requires at least two TX/RX descriptors, or three RX descriptors with PTP.
pub struct PacketQueue<const TX: usize, const RX: usize> {
    tx_desc: [TDes; TX],
    rx_desc: [RDes; RX],
    #[cfg(feature = "ptp")]
    tx_timestamps: Deque<TxTimestamp, TX>,
    tx_buf: [Option<PacketBuf>; TX],
    rx_buf: [Option<PacketBuf>; RX],
}

impl<const TX: usize, const RX: usize> PacketQueue<TX, RX> {
    /// Create a new packet queue.
    pub const fn new() -> Self {
        Self::new_inner()
    }

    const fn new_inner() -> Self {
        Self {
            tx_desc: [const { TDes::new() }; TX],
            rx_desc: [const { RDes::new() }; RX],
            #[cfg(feature = "ptp")]
            tx_timestamps: Deque::new(),
            tx_buf: [const { None }; TX],
            rx_buf: [const { None }; RX],
        }
    }

    /// Initialize a packet queue in-place.
    ///
    /// This can be helpful to avoid accidentally stack-allocating the packet queue in the stack. The
    /// Rust compiler can sometimes be a bit dumb when working with large owned values: if you call `new()`
    /// and then store the returned PacketQueue in its final place (like a `static`), the compiler might
    /// place it temporarily on the stack then move it. Since this struct is quite big, it may result
    /// in a stack overflow.
    ///
    /// With this function, you can create an uninitialized `static` with type `MaybeUninit<PacketQueue<...>>`
    /// and initialize the descriptor and buffer arrays in-place.
    ///
    /// After calling this function, calling `assume_init` on the MaybeUninit is guaranteed safe.
    pub fn init(this: &mut MaybeUninit<Self>) {
        // Descriptors are valid when zeroed. Construct buffers without relying
        // on their private representation.
        unsafe {
            let ptr = this.as_mut_ptr();
            (&raw mut (*ptr).tx_desc).write_bytes(0, 1);
            (&raw mut (*ptr).rx_desc).write_bytes(0, 1);
            // Copy a constant template: constructing a deque by value can also
            // put its backing array on the stack, especially without optimization.
            #[cfg(feature = "ptp")]
            (&raw mut (*ptr).tx_timestamps).copy_from_nonoverlapping(const { &Deque::new() }, 1);
            for i in 0..TX {
                (&raw mut (*ptr).tx_buf[i]).write(None);
            }
            for i in 0..RX {
                (&raw mut (*ptr).rx_buf[i]).write(None);
            }
        }
    }
}

static WAKER: AtomicWaker = AtomicWaker::new();

impl<'d, T: Instance, P: Phy> Driver for Ethernet<'d, T, P> {
    #[inline]
    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = MTU;
        // The v2/v1b/v1c MAC offloads the IPv4 header and TCP/UDP payload
        // checksums in hardware (MACCR.IPC + TDES3.CIC; bad RX frames are dropped
        // in the descriptor ring), so xarxa can skip them. On v2 each direction is
        // only offloaded when frames up to `MTU` fit the MTL queue it needs.
        #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
        {
            use xarxa_driver::ChecksumOffload;
            let offload = ChecksumOffload {
                rx: _version::RX_STORE_AND_FORWARD,
                tx: _version::TX_CHECKSUM_OFFLOAD,
            };
            caps.checksum.ipv4 = offload;
            caps.checksum.tcp = offload;
            caps.checksum.udp = offload;
            caps.checksum.icmpv4 = offload;
            caps.checksum.icmpv6 = offload;
        }
        #[cfg(any(eth_v1b, eth_v1c))]
        {
            use xarxa_driver::ChecksumOffload;
            caps.checksum.ipv4 = ChecksumOffload::BOTH;
            caps.checksum.tcp = ChecksumOffload::BOTH;
            caps.checksum.udp = ChecksumOffload::BOTH;
            caps.checksum.icmpv4 = ChecksumOffload::BOTH;
            caps.checksum.icmpv6 = ChecksumOffload::BOTH;
        }
        caps
    }

    fn rx_wanted(&mut self) -> usize {
        self.rx.wanted()
    }

    fn rx_give(&mut self, buf: PacketBuf) {
        self.rx.give(buf);
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        self.tx.fast_forward();

        match self.rx.receive() {
            Some(buf) => {
                self.wake_guard.disable();
                Some(buf)
            }
            None => {
                self.wake_guard.enable();
                None
            }
        }
    }

    fn can_transmit(&mut self) -> bool {
        self.tx.fast_forward();

        if self.tx.can_transmit() {
            self.wake_guard.disable();
            true
        } else {
            self.wake_guard.enable();
            false
        }
    }

    fn transmit(&mut self, buf: PacketBuf) -> Result<(), PacketBuf> {
        self.tx.fast_forward();

        if !self.tx.can_transmit() {
            return Err(buf);
        }
        self.tx.transmit(buf);
        Ok(())
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ethernet(self.mac_addr)
    }

    fn link_state(&mut self) -> LinkState {
        self.link_state
    }

    fn register_waker(&mut self, waker: &Waker) -> Result<(), NotSupported> {
        WAKER.register(waker);

        // The periodic PHY link poll is driven from here: this is called once
        // per stack poll, with the waker `Phy::poll_link` re-arms its timer
        // against. `Driver::link_state` then reports the cached state.
        let mut cx = Context::from_waker(waker);
        if let Some(link_state) = self.phy.poll_link(&mut cx) {
            self.link_state = if link_state { LinkState::Up } else { LinkState::Down };
        }

        Ok(())
    }

    #[cfg(feature = "ptp")]
    fn poll_tx_timestamp(&mut self) -> Option<xarxa_driver::TxTimestamp> {
        self.tx.fast_forward();
        self.tx.poll_timestamp()
    }
}

#[cfg(all(test, feature = "ptp"))]
mod tests {
    use embassy_ptp_driver::ScaledPpm;

    use super::adjusted_ptp_addend;

    const NOMINAL: u32 = 0xa000_0000;

    #[test]
    fn ptp_addend_uses_absolute_scaled_ppm() {
        assert_eq!(adjusted_ptp_addend(NOMINAL, ScaledPpm::ZERO), NOMINAL);
        assert_eq!(
            adjusted_ptp_addend(NOMINAL, ScaledPpm::from_raw(500 << 16)),
            0xa014_7ae1
        );
        assert_eq!(
            adjusted_ptp_addend(NOMINAL, ScaledPpm::from_raw(-500 << 16)),
            0x9feb_851f
        );
    }

    #[test]
    fn ptp_addend_stays_in_the_valid_register_range() {
        assert!(adjusted_ptp_addend(NOMINAL, ScaledPpm::from_raw(i32::MIN)) >= 1);
        assert_eq!(adjusted_ptp_addend(u32::MAX, ScaledPpm::from_raw(i32::MAX)), u32::MAX);
    }
}

/// Trait for an Ethernet PHY
pub trait Phy {
    /// Reset PHY and wait for it to come out of reset.
    /// Returns an error if reset or MDIO access fails.
    fn phy_reset(&mut self) -> Result<(), Error>;
    /// PHY initialisation.
    /// Returns an error if initialization fails.
    fn phy_init(&mut self) -> Result<(), Error>;
    /// Poll link to see if it is up and FD with 100Mbps
    fn poll_link(&mut self, cx: &mut Context) -> Option<bool>;
}

impl<'d, T: Instance, P: Phy> Ethernet<'d, T, P> {
    /// Access the user-supplied `Phy`.
    pub fn phy(&self) -> &P {
        &self.phy
    }

    /// Mutably access the user-supplied `Phy`.
    pub fn phy_mut(&mut self) -> &mut P {
        &mut self.phy
    }
}

struct State {}

impl State {
    const fn new() -> Self {
        Self {}
    }
}

peri_trait!(
    irqs: [Interrupt],
);

foreach_interrupt! {
    ($inst:ident, eth, $block:ident, GLOBAL, $irq:ident) => {
        peri_trait_impl!(
            $inst,
            irqs: [Interrupt : $irq]
        );
    };
}

pin_trait!(RXClkPin, Instance, @A);
pin_trait!(TXClkPin, Instance, @A);
pin_trait!(RefClkPin, Instance, @A);
pin_trait!(MDIOPin, sma::Instance, @A);
pin_trait!(MDCPin, sma::Instance, @A);
pin_trait!(RXDVPin, Instance, @A);
pin_trait!(CRSPin, Instance, @A);
pin_trait!(RXD0Pin, Instance, @A);
pin_trait!(RXD1Pin, Instance, @A);
pin_trait!(RXD2Pin, Instance, @A);
pin_trait!(RXD3Pin, Instance, @A);
pin_trait!(TXD0Pin, Instance, @A);
pin_trait!(TXD1Pin, Instance, @A);
pin_trait!(TXD2Pin, Instance, @A);
pin_trait!(TXD3Pin, Instance, @A);
pin_trait!(TXEnPin, Instance, @A);

pin_trait!(RGMIIGTXClkPin, Instance, @A);
pin_trait!(RGMIIRXClkPin, Instance, @A);
pin_trait!(RGMIIRXCtlPin, Instance, @A);
pin_trait!(RGMIITXCtlPin, Instance, @A);
pin_trait!(RGMIIRXD0Pin, Instance, @A);
pin_trait!(RGMIIRXD1Pin, Instance, @A);
pin_trait!(RGMIIRXD2Pin, Instance, @A);
pin_trait!(RGMIIRXD3Pin, Instance, @A);
pin_trait!(RGMIITXD0Pin, Instance, @A);
pin_trait!(RGMIITXD1Pin, Instance, @A);
pin_trait!(RGMIITXD2Pin, Instance, @A);
pin_trait!(RGMIITXD3Pin, Instance, @A);
pin_trait!(RGMIICLK125Pin, Instance, @A);
