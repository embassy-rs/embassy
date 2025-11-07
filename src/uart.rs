//! Minimal polling UART2 bring-up replicating MCUXpresso hello_world ordering.
//! WARNING: This is a narrow implementation only for debug console (115200 8N1).

use crate::pac;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use embassy_sync::signal::Signal;

// svd2rust defines the shared LPUART RegisterBlock under lpuart0; all instances reuse it.
type Regs = pac::lpuart0::RegisterBlock;

// Token-based instance pattern like embassy-imxrt
pub trait Instance {
    fn ptr() -> *const Regs;
}

/// Token for LPUART2 provided by embassy-hal-internal peripherals macro.
pub type Lpuart2 = crate::peripherals::LPUART2;
impl Instance for crate::peripherals::LPUART2 {
    #[inline(always)]
    fn ptr() -> *const Regs {
        pac::Lpuart2::ptr()
    }
}

// Also implement Instance for the Peri wrapper type
impl Instance for embassy_hal_internal::Peri<'_, crate::peripherals::LPUART2> {
    #[inline(always)]
    fn ptr() -> *const Regs {
        pac::Lpuart2::ptr()
    }
}

/// UART configuration (explicit src_hz; no hardcoded frequencies)
#[derive(Copy, Clone)]
pub struct Config {
    pub src_hz: u32,
    pub baud: u32,
    pub parity: Parity,
    pub stop_bits: StopBits,
}

#[derive(Copy, Clone)]
pub enum Parity {
    None,
    Even,
    Odd,
}
#[derive(Copy, Clone)]
pub enum StopBits {
    One,
    Two,
}

impl Config {
    pub fn new(src_hz: u32) -> Self {
        Self {
            src_hz,
            baud: 115_200,
            parity: Parity::None,
            stop_bits: StopBits::One,
        }
    }
}

/// Compute a valid (OSR, SBR) tuple for given source clock and baud.
/// Uses a functional fold approach to find the best OSR/SBR combination
/// with minimal baud rate error.
fn compute_osr_sbr(src_hz: u32, baud: u32) -> (u8, u16) {
    let (best_osr, best_sbr, _best_err) = (8u32..=32).fold(
        (16u8, 4u16, u32::MAX), // (best_osr, best_sbr, best_err)
        |(best_osr, best_sbr, best_err), osr| {
            let denom = baud.saturating_mul(osr);
            if denom == 0 {
                return (best_osr, best_sbr, best_err);
            }

            let sbr = (src_hz + denom / 2) / denom; // round
            if sbr == 0 || sbr > 0x1FFF {
                return (best_osr, best_sbr, best_err);
            }

            let actual = src_hz / (osr * sbr);
            let err = actual.abs_diff(baud);

            // Update best if this is better, or same error but higher OSR
            if err < best_err || (err == best_err && osr as u8 > best_osr) {
                (osr as u8, sbr as u16, err)
            } else {
                (best_osr, best_sbr, best_err)
            }
        },
    );
    (best_osr, best_sbr)
}

/// Minimal UART handle for a specific instance I (store the zero-sized token like embassy)
pub struct Uart<I: Instance> {
    _inst: core::marker::PhantomData<I>,
}

impl<I: Instance> Uart<I> {
    /// Create and initialize LPUART (reset + config). Clocks and pins must be prepared by the caller.
    pub fn new(_inst: impl Instance, cfg: Config) -> Self {
        let l = unsafe { &*I::ptr() };
        // 1) software reset pulse
        l.global().write(|w| w.rst().reset());
        cortex_m::asm::delay(3); // Short delay for reset to take effect
        l.global().write(|w| w.rst().no_effect());
        cortex_m::asm::delay(10); // Allow peripheral to stabilize after reset
        // 2) BAUD
        let (osr, sbr) = compute_osr_sbr(cfg.src_hz, cfg.baud);
        l.baud().modify(|_, w| {
            let w = match cfg.stop_bits {
                StopBits::One => w.sbns().one(),
                StopBits::Two => w.sbns().two(),
            };
            // OSR field encodes (osr-1); use raw bits to avoid a long match on all variants
            let raw_osr = osr.saturating_sub(1) as u8;
            unsafe { w.osr().bits(raw_osr).sbr().bits(sbr) }
        });
        // 3) CTRL baseline and parity
        l.ctrl().write(|w| {
            let w = w.ilt().from_stop().idlecfg().idle_2();
            let w = match cfg.parity {
                Parity::None => w.pe().disabled(),
                Parity::Even => w.pe().enabled().pt().even(),
                Parity::Odd => w.pe().enabled().pt().odd(),
            };
            w.re().enabled().te().enabled().rie().disabled()
        });
        // 4) FIFOs and WATER: keep it simple for polling; disable FIFOs and set RX watermark to 0
        l.fifo().modify(|_, w| {
            w.txfe()
                .disabled()
                .rxfe()
                .disabled()
                .txflush()
                .txfifo_rst()
                .rxflush()
                .rxfifo_rst()
        });
        l.water()
            .modify(|_, w| unsafe { w.txwater().bits(0).rxwater().bits(0) });
        Self {
            _inst: core::marker::PhantomData,
        }
    }

    /// Enable RX interrupts. The caller must ensure an appropriate IRQ handler is installed.
    pub unsafe fn enable_rx_interrupts(&self) {
        let l = &*I::ptr();
        l.ctrl().modify(|_, w| w.rie().enabled());
    }

    #[inline(never)]
    pub fn write_byte(&self, b: u8) {
        let l = unsafe { &*I::ptr() };
        // Timeout after ~10ms at 12MHz (assuming 115200 baud, should be plenty)
        const DATA_OFFSET: usize = 0x1C; // DATA register offset inside LPUART block
        let data_ptr = unsafe { (I::ptr() as *mut u8).add(DATA_OFFSET) };
        for _ in 0..120000 {
            if l.water().read().txcount().bits() == 0 {
                unsafe { core::ptr::write_volatile(data_ptr, b) };
                return;
            }
        }
        // If timeout, skip the write to avoid hanging
    }

    #[inline(never)]
    pub fn write_str_blocking(&self, s: &str) {
        for &b in s.as_bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
    }
    pub fn read_byte_blocking(&self) -> u8 {
        let l = unsafe { &*I::ptr() };
        while !l.stat().read().rdrf().is_rxdata() {}
        (l.data().read().bits() & 0xFF) as u8
    }
}

// Simple ring buffer for UART RX data
const RX_BUFFER_SIZE: usize = 256;
pub struct RingBuffer {
    buffer: [u8; RX_BUFFER_SIZE],
    read_idx: usize,
    write_idx: usize,
    count: usize,
}

impl RingBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [0; RX_BUFFER_SIZE],
            read_idx: 0,
            write_idx: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, data: u8) -> bool {
        if self.count >= RX_BUFFER_SIZE {
            return false; // Buffer full
        }
        self.buffer[self.write_idx] = data;
        self.write_idx = (self.write_idx + 1) % RX_BUFFER_SIZE;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.count == 0 {
            return None;
        }
        let data = self.buffer[self.read_idx];
        self.read_idx = (self.read_idx + 1) % RX_BUFFER_SIZE;
        self.count -= 1;
        Some(data)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

// Global RX buffer shared between interrupt handler and UART instance
static RX_BUFFER: Mutex<RefCell<RingBuffer>> = Mutex::new(RefCell::new(RingBuffer::new()));
static RX_SIGNAL: Signal<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, ()> =
    Signal::new();

// Debug counter for interrupt handler calls
static mut INTERRUPT_COUNT: u32 = 0;

impl<I: Instance> Uart<I> {
    /// Read a byte asynchronously using interrupts
    pub async fn read_byte_async(&self) -> u8 {
        loop {
            // Check if we have data in the buffer
            let byte = cortex_m::interrupt::free(|cs| {
                let mut buffer = RX_BUFFER.borrow(cs).borrow_mut();
                buffer.pop()
            });

            if let Some(byte) = byte {
                return byte;
            }

            // Wait for the interrupt signal
            RX_SIGNAL.wait().await;
        }
    }

    /// Check if there's data available in the RX buffer
    pub fn rx_data_available(&self) -> bool {
        cortex_m::interrupt::free(|cs| {
            let buffer = RX_BUFFER.borrow(cs).borrow();
            !buffer.is_empty()
        })
    }

    /// Try to read a byte from RX buffer (non-blocking)
    pub fn try_read_byte(&self) -> Option<u8> {
        cortex_m::interrupt::free(|cs| {
            let mut buffer = RX_BUFFER.borrow(cs).borrow_mut();
            buffer.pop()
        })
    }
}

/// Type-level handler for LPUART2 interrupts, compatible with bind_interrupts!.
pub struct UartInterruptHandler;

impl crate::interrupt::typelevel::Handler<crate::interrupt::typelevel::LPUART2>
    for UartInterruptHandler
{
    unsafe fn on_interrupt() {
        INTERRUPT_COUNT += 1;

        let lpuart = &*pac::Lpuart2::ptr();

        // Check if we have RX data
        if lpuart.stat().read().rdrf().is_rxdata() {
            // Read the data byte
            let data = (lpuart.data().read().bits() & 0xFF) as u8;

            // Store in ring buffer
            cortex_m::interrupt::free(|cs| {
                let mut buffer = RX_BUFFER.borrow(cs).borrow_mut();
                if buffer.push(data) {
                    // Data added successfully, signal waiting tasks
                    RX_SIGNAL.signal(());
                }
            });
        }
        // Always clear any error flags that might cause spurious interrupts
        let _ = lpuart.stat().read();
    }
}
