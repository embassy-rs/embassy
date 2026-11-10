//! Serial Peripheral Interface in slave mode.
use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::dma::{Channel, ChannelInstance};
use crate::gpio::{AnyPin, InputFuture, InterruptTrigger, SealedPin as _};
use crate::mode::{Async, Blocking, Mode};
pub use crate::spi::{ClkPin, CsPin, Instance, MisoPin, MosiPin};
use crate::spi::{Info, configure_pins};
use crate::{RegExt, pac};

// RP2040 exposes the entire transfer-count register as a u32.
#[cfg(feature = "rp2040")]
const DMA_MAX_COUNT: usize = 0x0fff_ffff;
// Derive the count field's maximum from the PAC without including the mode bits.
#[cfg(feature = "_rp235x")]
const DMA_MAX_COUNT: usize = {
    let mut r = pac::dma::regs::ChTransCount(0);
    r.set_count(u32::MAX);
    r.count() as usize
};

/// SPI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// RX FIFO overflowed and bytes were lost.
    ReceiveOverrun,
    /// Buffer exceeds the DMA count limit.
    BufferTooLong,
    /// DMA padding or discard capacity exhausted.
    TransactionTooLong,
    /// DMA bus error.
    DmaError,
    /// CS was low or changed before buffers were armed.
    NotReady,
}

/// SPI configuration. Defaults to mode 1, allowing multi-byte CS assertions.
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Config {
    /// SPI phase. Use `CaptureOnSecondTransition` for multi-byte transactions.
    /// `CaptureOnFirstTransition` requires CS to rise between bytes.
    pub phase: Phase,
    /// Polarity.
    pub polarity: Polarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            phase: Phase::CaptureOnSecondTransition,
            polarity: Polarity::IdleLow,
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Config {
    fn format(&self, f: defmt::Formatter) {
        let phase = match self.phase {
            Phase::CaptureOnFirstTransition => {
                defmt::intern!("CaptureOnFirstTransition")
            }
            Phase::CaptureOnSecondTransition => {
                defmt::intern!("CaptureOnSecondTransition")
            }
        };

        let polarity = match self.polarity {
            Polarity::IdleLow => defmt::intern!("IdleLow"),
            Polarity::IdleHigh => defmt::intern!("IdleHigh"),
        };

        defmt::write!(f, "Config {{ phase: {=istr}, polarity: {=istr} }}", phase, polarity,);
    }
}

struct DmaChannels<'d> {
    tx: Channel<'d, Blocking>,
    rx: Channel<'d, Blocking>,
    padding: Channel<'d, Blocking>,
    discard: Channel<'d, Blocking>,
}

impl DmaChannels<'_> {
    fn numbers(&self) -> [u8; 4] {
        [
            self.tx.number(),
            self.rx.number(),
            self.padding.number(),
            self.discard.number(),
        ]
    }

    fn mask(&self) -> u32 {
        self.numbers().iter().fold(0, |mask, &channel| mask | (1 << channel))
    }

    fn tail_mask(&self) -> u32 {
        (1 << self.padding.number()) | (1 << self.discard.number())
    }

    fn has_error(&self) -> bool {
        self.numbers()
            .iter()
            .any(|&channel| pac::DMA.ch(channel as _).ctrl_trig().read().ahb_error())
    }
}

/// Serial Peripheral Interface in slave mode.
///
/// Operations finish when CS rises. Buffers are independent upper bounds;
/// excess TX is zero-filled and RX discarded. Short transactions leave the unused
/// portion of RX unchanged. Returned counts are capped at each buffer's length;
/// padding and discarded bytes are not included. Full counts do not distinguish
/// an exactly filled buffer from a longer transaction.
/// SPI is enabled during transfer setup and left disabled after completion or cancellation.
///
/// Keep CS high until buffers are armed, and between operations until cleanup and
/// rearming finish. Async futures must be polled to arm DMA; master timing must allow
/// for worst-case software latency. CS GPIO interrupts wake software, not autonomously
/// stop DMA: successive assertions before software stops DMA may merge.
///
/// Use mode 1 or 3 for multi-byte transactions. Pace SCK so FIFO service keeps up,
/// including DMA contention. TX underruns cannot be detected: SSP has no latched
/// TX-underrun flag.
/// Slave MOSI uses [`MisoPin`] (RX); slave MISO uses [`MosiPin`] (TX).
///
/// Cancellation aborts DMA and clears FIFOs/overrun state; RX may be partially
/// modified. The master must end the old assertion and keep CS high through cleanup
/// and rearming.
pub struct Spi<'d, M: Mode> {
    info: &'static Info,
    cs: Peri<'d, AnyPin>,
    dma: Option<DmaChannels<'d>>,
    phantom: PhantomData<(&'d mut (), M)>,
}

impl<'d, M: Mode> Spi<'d, M> {
    fn new_inner<T: Instance>(
        _spi: Peri<'d, T>,
        clk: Peri<'d, AnyPin>,
        mosi: Option<Peri<'d, AnyPin>>,
        miso: Option<Peri<'d, AnyPin>>,
        cs: Peri<'d, AnyPin>,
        dma: Option<DmaChannels<'d>>,
        config: Config,
    ) -> Self {
        let p = T::info().regs;

        // Disable before selecting SPI slave mode.
        p.cr1().write(|w| {
            w.set_ms(true);
            w.set_sse(false)
        });

        Self::apply_config(T::info(), &config);

        // Always enable DREQ signals -- harmless if DMA is not listening
        p.dmacr().write(|reg| {
            reg.set_rxdmae(true);
            reg.set_txdmae(true);
        });

        configure_pins(&[Some(&clk), mosi.as_ref(), miso.as_ref(), Some(&cs)]);

        Self {
            info: T::info(),
            cs,
            dma,
            phantom: PhantomData,
        }
    }

    /// Apply SPI configuration (phase and polarity) while the driver is disabled.
    fn apply_config(info: &Info, config: &Config) {
        let p = info.regs;
        p.cr0().write(|w| {
            w.set_dss(0b0111); // 8bit
            w.set_spo(config.polarity == Polarity::IdleHigh);
            w.set_sph(config.phase == Phase::CaptureOnSecondTransition);
        });
    }

    /// Sets SPI configuration between transactions.
    pub fn set_config(&mut self, config: &Config) {
        // Transfers and cancellation leave SPI disabled. Exclusive access prevents
        // reconfiguration during a transfer.
        Self::apply_config(self.info, config);
    }

    /// Sends data, discarding received data. Blocks until CS rises.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.blocking_transfer(&mut [], data).map(|(_, sent)| sent)
    }

    /// Reads data, sending zeroes. Blocks until CS rises.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.blocking_transfer(data, &[]).map(|(received, _)| received)
    }

    /// Simultaneously sends and receives data. Blocks until CS rises.
    /// Returns `(received, sent)` counts, capped at the respective buffer lengths.
    pub fn blocking_transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(usize, usize), Error> {
        let rx_len = rx.len();
        let count_limit = rx_len.max(tx.len());
        let mut exchanged = 0usize;
        let mut tx_bytes = tx.iter();
        self.prepare_blocking_transfer(&mut tx_bytes)?;

        let p = self.info.regs;
        let cs_pin = self.cs._pin() as usize;
        let cs_intr = self.cs.io().intr(cs_pin / 8);

        // Service both FIFOs until CS rises, padding TX and discarding excess RX.
        let mut rx_bytes = rx.iter_mut();
        while !cs_intr.read().edge_high(cs_pin % 8) {
            // Refill TX when there is space, sending zeroes after the buffer ends.
            if p.sr().read().tnf() {
                let byte = tx_bytes.next().copied().unwrap_or(0);
                p.dr().write(|w| w.set_data(byte as _));
            }

            // Always consume RX; discard bytes once the buffer is full.
            if p.sr().read().rne() {
                let byte = p.dr().read().data() as u8;
                exchanged = exchanged.saturating_add(1).min(count_limit);
                if let Some(slot) = rx_bytes.next() {
                    *slot = byte;
                }
            }
        }

        p.cr1().modify(|w| w.set_sse(false));
        exchanged = exchanged.saturating_add(self.finish_transfer(&mut rx_bytes)?);
        Ok((exchanged.min(rx_len), exchanged.min(tx.len())))
    }

    /// Clears previous transaction state and preloads TX, rejecting CS activity during setup.
    fn prepare_blocking_transfer(&mut self, tx_bytes: &mut core::slice::Iter<'_, u8>) -> Result<(), Error> {
        let p = self.info.regs;

        // Select the raw interrupt register covering CS; each register holds eight GPIOs.
        let cs_pin = self.cs._pin() as usize;
        let cs_intr = self.cs.io().intr(cs_pin / 8);

        // Clear stale CS edges and latch any assertion during setup.
        cs_intr.write(|w| {
            w.set_edge_low(cs_pin % 8, true);
            w.set_edge_high(cs_pin % 8, true);
        });

        // Clear stale FIFO data and overrun state before preparing the transaction.
        Self::disable_and_clear_fifos(self.info);

        // Reject a transaction already in progress.
        if !self.cs.gpio().status().read().infrompad() {
            return Err(Error::NotReady);
        }

        // Enable for FIFO preload and the next CS assertion.
        p.cr1().modify(|w| w.set_sse(true));

        // Preload the eight-entry TX FIFO without waiting for clocks.
        for _ in 0..8 {
            if !p.sr().read().tnf() {
                break;
            }
            let byte = tx_bytes.next().copied().unwrap_or(0);
            p.dr().write(|w| w.set_data(byte as _));
        }

        // Reject CS activity before preload completed.
        if !self.cs.gpio().status().read().infrompad()
            || cs_intr.read().edge_low(cs_pin % 8)
            || cs_intr.read().edge_high(cs_pin % 8)
        {
            Self::disable_and_clear_fifos(self.info);
            return Err(Error::NotReady);
        }
        Ok(())
    }

    /// Stores residual RX, checks overrun and clears transaction state.
    /// SPI and DMA must be stopped before calling.
    fn finish_transfer(&self, rx_bytes: &mut core::slice::IterMut<'_, u8>) -> Result<usize, Error> {
        let p = self.info.regs;
        let drained = self.drain_received(rx_bytes);

        // Inspect overrun before cleanup clears it.
        if p.ris().read().rorris() {
            Self::disable_and_clear_fifos(self.info);
            return Err(Error::ReceiveOverrun);
        }

        Self::disable_and_clear_fifos(self.info);
        Ok(drained)
    }

    /// Stores queued RX bytes in the remaining buffer slots, discarding excess bytes.
    /// SPI and RX DMA must be stopped before calling.
    fn drain_received(&self, rx_bytes: &mut core::slice::IterMut<'_, u8>) -> usize {
        let p = self.info.regs;
        let mut drained = 0;

        // Drain queued bytes without waiting for more clocks.
        while p.sr().read().rne() {
            let byte = p.dr().read().data() as u8;
            drained += 1;
            if let Some(slot) = rx_bytes.next() {
                *slot = byte;
            }
        }
        drained
    }

    /// Disables SPI and clears RX, TX and receive-overrun state, preserving configuration.
    /// DMA must be stopped before calling.
    fn disable_and_clear_fifos(info: &Info) {
        let p = info.regs;

        p.cr1().modify(|w| w.set_sse(false));

        let saved_cr0 = p.cr0().read();
        let saved_cr1 = p.cr1().read();
        let saved_dmacr = p.dmacr().read();

        // Reset clears both FIFOs and overrun state; SSP has no TX FIFO clear register.
        let is_spi0 = p.as_ptr() == pac::SPI0.as_ptr();
        pac::RESETS.reset().write_set(|w| {
            w.set_spi0(is_spi0);
            w.set_spi1(!is_spi0);
        });
        pac::RESETS.reset().write_clear(|w| {
            w.set_spi0(is_spi0);
            w.set_spi1(!is_spi0);
        });
        while !(if is_spi0 {
            pac::RESETS.reset_done().read().spi0()
        } else {
            pac::RESETS.reset_done().read().spi1()
        }) {}

        // Restore configuration, leaving SPI disabled.
        p.cr0().write_value(saved_cr0);
        p.cr1().write_value(saved_cr1);
        p.dmacr().write_value(saved_dmacr);
    }
}

impl<'d> Spi<'d, Blocking> {
    /// Create an SPI slave driver in blocking mode.
    pub fn new_blocking<T: Instance>(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MisoPin<T> + 'd>,
        miso: Peri<'d, impl MosiPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            spi,
            clk.into(),
            Some(miso.into()),
            Some(mosi.into()),
            cs.into(),
            None,
            config,
        )
    }

    /// Create an SPI slave driver in blocking mode supporting reads only (MOSI only).
    pub fn new_blocking_rxonly<T: Instance>(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MisoPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
        config: Config,
    ) -> Self {
        Self::new_inner(spi, clk.into(), None, Some(mosi.into()), cs.into(), None, config)
    }
}

impl<'d> Spi<'d, Async> {
    /// Create an SPI slave driver in async mode supporting DMA operations.
    /// Uses four DMA channels for buffers and padding/discard tails; no DMA IRQ required.
    pub fn new<
        T: Instance,
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
        TxTailDma: ChannelInstance,
        RxTailDma: ChannelInstance,
    >(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MisoPin<T> + 'd>,
        miso: Peri<'d, impl MosiPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        tx_tail_dma: Peri<'d, TxTailDma>,
        rx_tail_dma: Peri<'d, RxTailDma>,
        config: Config,
    ) -> Self {
        Self::new_async_inner(
            spi,
            clk.into(),
            Some(miso.into()),
            mosi.into(),
            cs.into(),
            tx_dma,
            rx_dma,
            tx_tail_dma,
            rx_tail_dma,
            config,
        )
    }

    /// Create an SPI slave driver in async mode supporting reads only (MOSI only).
    /// Uses four DMA channels; no DMA IRQ required.
    pub fn new_rxonly<
        T: Instance,
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
        TxTailDma: ChannelInstance,
        RxTailDma: ChannelInstance,
    >(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        mosi: Peri<'d, impl MisoPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        tx_tail_dma: Peri<'d, TxTailDma>,
        rx_tail_dma: Peri<'d, RxTailDma>,
        config: Config,
    ) -> Self {
        Self::new_async_inner(
            spi,
            clk.into(),
            None,
            mosi.into(),
            cs.into(),
            tx_dma,
            rx_dma,
            tx_tail_dma,
            rx_tail_dma,
            config,
        )
    }

    fn new_async_inner<T: Instance>(
        spi: Peri<'d, T>,
        clk: Peri<'d, AnyPin>,
        tx_pin: Option<Peri<'d, AnyPin>>,
        rx_pin: Peri<'d, AnyPin>,
        cs: Peri<'d, AnyPin>,
        tx_dma: Peri<'d, impl ChannelInstance>,
        rx_dma: Peri<'d, impl ChannelInstance>,
        padding_dma: Peri<'d, impl ChannelInstance>,
        discard_dma: Peri<'d, impl ChannelInstance>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            spi,
            clk,
            tx_pin,
            Some(rx_pin),
            cs,
            Some(DmaChannels {
                tx: Channel::new_no_interrupt(tx_dma),
                rx: Channel::new_no_interrupt(rx_dma),
                padding: Channel::new_no_interrupt(padding_dma),
                discard: Channel::new_no_interrupt(discard_dma),
            }),
            config,
        )
    }

    /// Sends data, discarding received data. Waits until CS rises.
    /// See [`Self::transfer`] for DMA limits.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.transfer(&mut [], data).await.map(|(_, sent)| sent)
    }

    /// Reads data, sending zeroes. Waits until CS rises.
    /// See [`Self::transfer`] for DMA limits.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.transfer(data, &[]).await.map(|(received, _)| received)
    }

    /// Simultaneously sends and receives data until CS rises.
    /// Preloads TX; see [`Spi`] for master timing requirements.
    ///
    /// Short transactions leave the unused portion of RX unchanged.
    /// Returns `(received, sent)` counts, capped at the respective buffer lengths.
    /// Both counts can be full while the operation continues waiting for CS to rise.
    /// Each DMA buffer is limited to the chip's DMA transfer count maximum
    /// (`u32::MAX` bytes on RP2040, `0x0fff_ffff` bytes on RP235x). After the buffers
    /// end, DMA can pad TX and discard RX for up to that many bytes each.
    /// Exhausting either tail returns `TransactionTooLong`; RX FIFO overrun returns
    /// `ReceiveOverrun`. All buffers stay borrowed until DMA has stopped.
    pub async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(usize, usize), Error> {
        // Reject buffers whose lengths cannot be represented by DMA.
        if rx.len() > DMA_MAX_COUNT || tx.len() > DMA_MAX_COUNT {
            return Err(Error::BufferTooLong);
        }

        // Each transfer owns its scratch byte, including when both SPI peripherals run.
        // Declare it before cleanup so DMA stops before its storage is released.
        let mut discard_byte = 0;

        let cs_pin = self.cs._pin() as usize;
        let cs_gpio = self.cs.gpio();
        let cs_intr = self.cs.io().intr(cs_pin / 8);

        let info = self.info;
        let dma = self.dma.as_ref().unwrap();

        // Stop DMA before releasing buffer borrows on cancellation.
        let cleanup = OnDrop::new(|| {
            Self::stop_dma(info, dma);
            Self::disable_and_clear_fifos(info);
        });

        // Release the CS future's pin borrow before finishing the transfer.
        {
            // Clear stale edges once, before setup, so assertions during setup stay latched.
            let cs_end = InputFuture::new(self.cs.reborrow(), InterruptTrigger::EdgeHigh);
            if !cs_gpio.status().read().infrompad() {
                return Err(Error::NotReady);
            }

            // Clear the previous transaction before configuring DMA.
            Self::disable_and_clear_fifos(info);
            Self::prepare_dma(dma);
            let mut cs_end = core::pin::pin!(cs_end);

            // SAFETY: Channels are owned and idle, and buffer lengths were checked above.
            // The cleanup guard stops DMA before any buffer borrow is released.
            unsafe { Self::configure_transfer_dma(info, dma, rx, tx, &mut discard_byte) };

            // Publish buffer contents before any DMA channel starts.
            compiler_fence(Ordering::SeqCst);

            // Start RX before TX so incoming bytes cannot be lost during setup.
            pac::DMA.multi_chan_trigger().write(|w| {
                w.set_multi_chan_trigger(
                    1 << if rx.is_empty() {
                        dma.discard.number()
                    } else {
                        dma.rx.number()
                    },
                );
            });

            // Enable only after DMA is configured and RX is armed.
            info.regs.cr1().modify(|w| w.set_sse(true));

            // Start TX preload, using padding directly when the TX buffer is empty.
            pac::DMA.multi_chan_trigger().write(|w| {
                w.set_multi_chan_trigger(
                    1 << if tx.is_empty() {
                        dma.padding.number()
                    } else {
                        dma.tx.number()
                    },
                );
            });

            // Keep preload cancellable and register the CS waker during setup.
            poll_fn(|cx| {
                // Observe preload first, then validate CS so activity during this poll
                // cannot be accepted just because the FIFO became full afterward.
                let preloaded = !info.regs.sr().read().tnf();

                // Reject a transaction that starts before preload is complete.
                if cs_end.as_mut().poll(cx).is_ready()
                    || !cs_gpio.status().read().infrompad()
                    || cs_intr.read().edge_low(cs_pin % 8)
                {
                    return Poll::Ready(Err(Error::NotReady));
                }

                // Reject DMA bus errors during preload.
                if dma.has_error() {
                    return Poll::Ready(Err(Error::DmaError));
                }

                // A full TX FIFO is ready for the next CS assertion.
                if preloaded {
                    return Poll::Ready(Ok(()));
                }

                // DMA IRQs are masked, so poll again while preload is in progress.
                cx.waker().wake_by_ref();
                Poll::Pending
            })
            .await?;

            // Wait for CS to rise before stopping SPI and DMA.
            cs_end.as_mut().await;
        }

        // Stop DMA and retire pending writes before accessing RX.
        let completed_channels = Self::stop_dma(info, dma);
        cleanup.defuse();

        self.finish_dma_transfer(rx, tx.len(), dma, completed_channels)
    }

    /// Configures the buffer channels and their padding/discard tails without starting DMA.
    ///
    /// # Safety
    /// Channels must be owned and idle, and buffer lengths must fit `DMA_MAX_COUNT`.
    /// Buffers and the scratch byte must remain borrowed
    /// and at stable addresses until DMA has stopped.
    unsafe fn configure_transfer_dma(
        info: &Info,
        dma: &DmaChannels<'_>,
        rx: &mut [u8],
        tx: &[u8],
        discard_byte: &mut u8,
    ) {
        // static mut so that this is allocated in RAM; only read by DMA.
        static mut PADDING_BYTE: u8 = 0;

        // Discard excess RX into one byte without advancing the destination.
        Self::configure_dma(
            dma.discard.number(),
            info.regs.dr().as_ptr() as _,
            discard_byte,
            DMA_MAX_COUNT,
            false,
            false,
            info.rx_dreq,
            dma.discard.number(),
        );

        // Repeatedly send the same zero byte after the TX buffer ends.
        Self::configure_dma(
            dma.padding.number(),
            core::ptr::addr_of_mut!(PADDING_BYTE) as *const u8,
            info.regs.dr().as_ptr() as _,
            DMA_MAX_COUNT,
            false,
            false,
            info.tx_dreq,
            dma.padding.number(),
        );

        // Store RX in the buffer, then discard excess bytes.
        if !rx.is_empty() {
            Self::configure_dma(
                dma.rx.number(),
                info.regs.dr().as_ptr() as _,
                rx.as_mut_ptr(),
                rx.len(),
                false,
                true,
                info.rx_dreq,
                dma.discard.number(),
            );
        }

        // Send the TX buffer once, then chain to zero padding.
        if !tx.is_empty() {
            Self::configure_dma(
                dma.tx.number(),
                tx.as_ptr(),
                info.regs.dr().as_ptr() as _,
                tx.len(),
                true,
                false,
                info.tx_dreq,
                dma.padding.number(),
            );
        }
    }

    /// Stores residual RX, checks DMA and SPI errors and clears transaction state.
    /// SPI and all DMA channels must be stopped before calling.
    fn finish_dma_transfer(
        &self,
        rx: &mut [u8],
        tx_len: usize,
        dma: &DmaChannels<'_>,
        completed_channels: u32,
    ) -> Result<(usize, usize), Error> {
        // Check bus errors before using the DMA destination address.
        if dma.has_error() {
            Self::disable_and_clear_fifos(self.info);
            return Err(Error::DmaError);
        }

        // Resume filling RX at the DMA destination; pending FIFO bytes still belong to it.
        // An empty buffer leaves its channel unused, with a potentially stale address.
        let next = if rx.is_empty() {
            0
        } else {
            let addr = pac::DMA.ch(dma.rx.number() as _).write_addr().read();
            (addr.wrapping_sub(rx.as_ptr() as u32) as usize).min(rx.len())
        };
        // The discard channel is configured for every operation, even with empty RX.
        // Count completed exchanges using RX, since TX DMA also counts FIFO preload.
        let discard = pac::DMA.ch(dma.discard.number() as _).trans_count().read();
        #[cfg(feature = "rp2040")]
        let discard_remaining = discard as usize;
        #[cfg(feature = "_rp235x")]
        let discard_remaining = discard.count() as usize;
        let exchanged = next.saturating_add(DMA_MAX_COUNT - discard_remaining);
        let drained = self.finish_transfer(&mut rx[next..].iter_mut())?;
        let exchanged = exchanged.saturating_add(drained);

        // Both tails are finite DMA jobs; completion means their capacity was exhausted.
        if completed_channels & dma.tail_mask() != 0 {
            return Err(Error::TransactionTooLong);
        }

        Ok((exchanged.min(rx.len()), exchanged.min(tx_len)))
    }

    /// Configures a byte-wide DMA channel without triggering it.
    ///
    /// # Safety
    /// The channel must be owned and idle, and `chain_to` must select an owned channel.
    /// `len` must be nonzero and at most `DMA_MAX_COUNT`. Source and destination must
    /// support the configured accesses and remain valid until DMA stops. DMA must have
    /// exclusive access to destination storage, and source storage must not be modified.
    unsafe fn configure_dma(
        channel: u8,
        from: *const u8,
        to: *mut u8,
        len: usize,
        incr_read: bool,
        incr_write: bool,
        dreq: pac::dma::vals::TreqSel,
        chain_to: u8,
    ) {
        let p = pac::DMA.ch(channel as _);

        p.read_addr().write_value(from as u32);
        p.write_addr().write_value(to as u32);

        // Program a finite transfer count on either chip.
        #[cfg(feature = "rp2040")]
        p.trans_count().write_value(len as u32);
        #[cfg(feature = "_rp235x")]
        p.trans_count().write(|w| {
            w.set_mode(0.into());
            w.set_count(len as u32);
        });

        // Use a non-triggering alias so tails can be configured before buffer channels.
        p.al1_ctrl().write(|ctrl| {
            ctrl.set_data_size(pac::dma::vals::DataSize::SizeByte);
            ctrl.set_incr_read(incr_read);
            ctrl.set_incr_write(incr_write);

            ctrl.set_treq_sel(dreq);
            ctrl.set_chain_to(chain_to);

            // Report completion for channels that do not chain onward.
            ctrl.set_irq_quiet(chain_to != channel);

            // Clear stale bus errors and enable the channel for a later trigger.
            ctrl.set_read_error(true);
            ctrl.set_write_error(true);
            ctrl.set_en(true);
        });
    }

    /// Masks DMA interrupts and clears previous completion, error and chaining state.
    fn prepare_dma(dma: &DmaChannels<'_>) {
        let mask = dma.mask();

        // Mask all owned channels; bus errors ignore IRQ_QUIET.
        pac::DMA.inte(0).write_clear(|w| *w = mask);
        pac::DMA.inte(1).write_clear(|w| *w = mask);
        #[cfg(feature = "_rp235x")]
        {
            pac::DMA.inte(2).write_clear(|w| *w = mask);
            pac::DMA.inte(3).write_clear(|w| *w = mask);
        }

        // Clear completion flags from the previous transaction.
        pac::DMA.intr(0).write_value(mask);

        // Leave channels disabled and clear bus errors; self-chaining disables chaining.
        for channel in dma.numbers() {
            pac::DMA.ch(channel as _).al1_ctrl().write(|w| {
                w.set_read_error(true);
                w.set_write_error(true);
                w.set_chain_to(channel);
            });
        }
    }

    /// Stops SPI and DMA, waits for pending writes and returns the pre-abort completion mask.
    fn stop_dma(info: &Info, dma: &DmaChannels<'_>) -> u32 {
        // Stop SPI before disabling the DMA channels servicing its FIFOs.
        info.regs.cr1().modify(|w| w.set_sse(false));

        let mask = dma.mask();
        // Disable all channels before abort to prevent tail retriggering (RP2350-E5).
        for channel in dma.numbers() {
            pac::DMA.ch(channel as _).ctrl_trig().write_clear(|w| w.set_en(true));
        }

        // Sever chains before abort to prevent retriggering.
        for channel in dma.numbers() {
            pac::DMA.ch(channel as _).al1_ctrl().modify(|w| {
                w.set_chain_to(channel);

                // These flags are write-one-to-clear; preserve errors for the final check.
                w.set_read_error(false);
                w.set_write_error(false);
            });
        }

        // Capture genuine completion before abort can assert it (RP2040-E13).
        let completed_channels = pac::DMA.intr(0).read() & mask;

        // Abort all owned channels and wait for the requests to clear.
        pac::DMA.chan_abort().write(|w| w.set_chan_abort(mask as _));
        while u32::from(pac::DMA.chan_abort().read().chan_abort()) & mask != 0 {}

        // RP2040-E13: wait for BUSY too; CHAN_ABORT can clear before writes retire.
        while dma
            .numbers()
            .iter()
            .any(|&channel| pac::DMA.ch(channel as _).ctrl_trig().read().busy())
        {}

        // Keep subsequent buffer accesses after DMA has stopped.
        compiler_fence(Ordering::SeqCst);

        // RP2040-E13 may assert completion during abort.
        pac::DMA.intr(0).write_value(mask);

        completed_channels
    }
}

impl<M: Mode> Drop for Spi<'_, M> {
    fn drop(&mut self) {
        self.info.regs.dmacr().write(|w| {
            w.set_rxdmae(false);
            w.set_txdmae(false);
        });
        self.info.regs.cr1().modify(|w| w.set_sse(false));
    }
}

// ====================

impl<'d, M: Mode> SetConfig for Spi<'d, M> {
    type Config = Config;
    type ConfigError = core::convert::Infallible;
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config);
        Ok(())
    }
}
