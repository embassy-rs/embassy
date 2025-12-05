//! DMA channel linking example for MCXA276.
//!
//! This example demonstrates DMA channel linking (minor and major loop linking):
//! - Channel 0: Transfers SRC_BUFFER to DEST_BUFFER0, with:
//!   - Minor Link to Channel 1 (triggers CH1 after each minor loop)
//!   - Major Link to Channel 2 (triggers CH2 after major loop completes)
//! - Channel 1: Transfers SRC_BUFFER to DEST_BUFFER1 (triggered by CH0 minor link)
//! - Channel 2: Transfers SRC_BUFFER to DEST_BUFFER2 (triggered by CH0 major link)
//!
//! # Embassy-style features demonstrated:
//! - `DmaChannel::new()` for channel creation
//! - `DmaChannel::is_done()` and `clear_done()` helper methods
//! - Channel linking with `set_minor_link()` and `set_major_link()`
//! - Standard `DmaCh*InterruptHandler` with `bind_interrupts!` macro

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaCh0InterruptHandler, DmaCh1InterruptHandler, DmaCh2InterruptHandler, DmaChannel};
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::{bind_interrupts, pac};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};
use core::fmt::Write as _;

// Buffers
static mut SRC_BUFFER: [u32; 4] = [1, 2, 3, 4];
static mut DEST_BUFFER0: [u32; 4] = [0; 4];
static mut DEST_BUFFER1: [u32; 4] = [0; 4];
static mut DEST_BUFFER2: [u32; 4] = [0; 4];

// Bind DMA channel interrupts using Embassy-style macro
// The standard handlers call on_interrupt() which wakes wakers and clears flags
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
    DMA_CH1 => DmaCh1InterruptHandler;
    DMA_CH2 => DmaCh2InterruptHandler;
});

/// Helper to print a buffer to UART
fn print_buffer(tx: &mut LpuartTx<'_, Blocking>, buf_ptr: *const u32, len: usize) {
    write!(tx, "{:?}", unsafe { core::slice::from_raw_parts(buf_ptr, len) }).ok();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Small delay to allow probe-rs to attach after reset
    for _ in 0..100_000 {
        cortex_m::asm::nop();
    }

    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("DMA channel link example starting...");

    // DMA is initialized during hal::init() - no need to call ensure_init()

    let pac_periphs = unsafe { pac::Peripherals::steal() };
    let dma0 = &pac_periphs.dma0;
    let edma = unsafe { &*pac::Edma0Tcd0::ptr() };

    // Clear any residual state
    for i in 0..3 {
        let t = edma.tcd(i);
        t.ch_csr().write(|w| w.erq().disable().done().clear_bit_by_one());
        t.ch_int().write(|w| w.int().clear_bit_by_one());
        t.ch_es().write(|w| w.err().clear_bit_by_one());
        t.ch_mux().write(|w| unsafe { w.bits(0) });
    }

    // Clear Global Halt/Error state
    dma0.mp_csr().modify(|_, w| {
        w.halt()
            .normal_operation()
            .hae()
            .normal_operation()
            .ecx()
            .normal_operation()
            .cx()
            .normal_operation()
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH0);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH1);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH2);
    }

    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"EDMA channel link example begin.\r\n\r\n").unwrap();

    // Initialize buffers
    unsafe {
        SRC_BUFFER = [1, 2, 3, 4];
        DEST_BUFFER0 = [0; 4];
        DEST_BUFFER1 = [0; 4];
        DEST_BUFFER2 = [0; 4];
    }

    tx.blocking_write(b"Source Buffer:   ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(SRC_BUFFER) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"DEST0 (before):  ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER0) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"DEST1 (before):  ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER1) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"DEST2 (before):  ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER2) as *const u32, 4);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    tx.blocking_write(b"Configuring DMA channels with Embassy-style API...\r\n")
        .unwrap();

    let ch0 = DmaChannel::new(p.DMA_CH0);
    let ch1 = DmaChannel::new(p.DMA_CH1);
    let ch2 = DmaChannel::new(p.DMA_CH2);

    // Configure channels using direct TCD access (advanced feature demo)
    // This example demonstrates channel linking which requires direct TCD manipulation

    // Helper to configure TCD for memory-to-memory transfer
    // Parameters: channel, src, dst, width, nbytes (minor loop), count (major loop), interrupt
    #[allow(clippy::too_many_arguments)]
    unsafe fn configure_tcd(
        edma: &embassy_mcxa::pac::edma_0_tcd0::RegisterBlock,
        ch: usize,
        src: u32,
        dst: u32,
        width: u8,
        nbytes: u32,
        count: u16,
        enable_int: bool,
    ) {
        let t = edma.tcd(ch);

        // Reset channel state
        t.ch_csr().write(|w| {
            w.erq()
                .disable()
                .earq()
                .disable()
                .eei()
                .no_error()
                .ebw()
                .disable()
                .done()
                .clear_bit_by_one()
        });
        t.ch_es().write(|w| w.bits(0));
        t.ch_int().write(|w| w.int().clear_bit_by_one());

        // Source/destination addresses
        t.tcd_saddr().write(|w| w.saddr().bits(src));
        t.tcd_daddr().write(|w| w.daddr().bits(dst));

        // Offsets: increment by width
        t.tcd_soff().write(|w| w.soff().bits(width as u16));
        t.tcd_doff().write(|w| w.doff().bits(width as u16));

        // Attributes: size = log2(width)
        let size = match width {
            1 => 0,
            2 => 1,
            4 => 2,
            _ => 0,
        };
        t.tcd_attr().write(|w| w.ssize().bits(size).dsize().bits(size));

        // Number of bytes per minor loop
        t.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(nbytes));

        // Major loop: reset source address after major loop
        let total_bytes = nbytes * count as u32;
        t.tcd_slast_sda()
            .write(|w| w.slast_sda().bits(-(total_bytes as i32) as u32));
        t.tcd_dlast_sga()
            .write(|w| w.dlast_sga().bits(-(total_bytes as i32) as u32));

        // Major loop count
        t.tcd_biter_elinkno().write(|w| w.biter().bits(count));
        t.tcd_citer_elinkno().write(|w| w.citer().bits(count));

        // Control/status: enable interrupt if requested
        if enable_int {
            t.tcd_csr().write(|w| w.intmajor().set_bit());
        } else {
            t.tcd_csr().write(|w| w.intmajor().clear_bit());
        }

        cortex_m::asm::dsb();
    }

    unsafe {
        // Channel 0: Transfer 16 bytes total (8 bytes per minor loop, 2 major iterations)
        // Minor Link -> Channel 1
        // Major Link -> Channel 2
        configure_tcd(
            edma,
            0,
            core::ptr::addr_of!(SRC_BUFFER) as u32,
            core::ptr::addr_of_mut!(DEST_BUFFER0) as u32,
            4,     // src width
            8,     // nbytes (minor loop = 2 words)
            2,     // count (major loop = 2 iterations)
            false, // no interrupt
        );
        ch0.set_minor_link(1); // Link to CH1 after each minor loop
        ch0.set_major_link(2); // Link to CH2 after major loop

        // Channel 1: Transfer 16 bytes (triggered by CH0 minor link)
        configure_tcd(
            edma,
            1,
            core::ptr::addr_of!(SRC_BUFFER) as u32,
            core::ptr::addr_of_mut!(DEST_BUFFER1) as u32,
            4,
            16, // full buffer in one minor loop
            1,  // 1 major iteration
            false,
        );

        // Channel 2: Transfer 16 bytes (triggered by CH0 major link)
        configure_tcd(
            edma,
            2,
            core::ptr::addr_of!(SRC_BUFFER) as u32,
            core::ptr::addr_of_mut!(DEST_BUFFER2) as u32,
            4,
            16,   // full buffer in one minor loop
            1,    // 1 major iteration
            true, // enable interrupt
        );
    }

    tx.blocking_write(b"Triggering Channel 0 (1st minor loop)...\r\n")
        .unwrap();

    // Trigger first minor loop of CH0
    unsafe {
        ch0.trigger_start();
    }

    // Wait for CH1 to complete (triggered by CH0 minor link)
    while !ch1.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        ch1.clear_done();
    }

    tx.blocking_write(b"CH1 done (via minor link).\r\n").unwrap();
    tx.blocking_write(b"Triggering Channel 0 (2nd minor loop)...\r\n")
        .unwrap();

    // Trigger second minor loop of CH0
    unsafe {
        ch0.trigger_start();
    }

    // Wait for CH0 major loop to complete
    while !ch0.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        ch0.clear_done();
    }

    tx.blocking_write(b"CH0 major loop done.\r\n").unwrap();

    // Wait for CH2 to complete (triggered by CH0 major link)
    // Using is_done() instead of AtomicBool - the standard interrupt handler
    // clears the interrupt flag and wakes wakers, but DONE bit remains set
    while !ch2.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        ch2.clear_done();
    }

    tx.blocking_write(b"CH2 done (via major link).\r\n\r\n").unwrap();

    tx.blocking_write(b"EDMA channel link example finish.\r\n\r\n").unwrap();

    tx.blocking_write(b"DEST0 (after):   ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER0) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"DEST1 (after):   ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER1) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"DEST2 (after):   ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER2) as *const u32, 4);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Verify all buffers match source
    let mut success = true;
    unsafe {
        let src_ptr = core::ptr::addr_of!(SRC_BUFFER) as *const u32;
        let dst0_ptr = core::ptr::addr_of!(DEST_BUFFER0) as *const u32;
        let dst1_ptr = core::ptr::addr_of!(DEST_BUFFER1) as *const u32;
        let dst2_ptr = core::ptr::addr_of!(DEST_BUFFER2) as *const u32;

        for i in 0..4 {
            if *dst0_ptr.add(i) != *src_ptr.add(i) {
                success = false;
            }
            if *dst1_ptr.add(i) != *src_ptr.add(i) {
                success = false;
            }
            if *dst2_ptr.add(i) != *src_ptr.add(i) {
                success = false;
            }
        }
    }

    if success {
        tx.blocking_write(b"PASS: Data verified.\r\n").unwrap();
        defmt::info!("PASS: Data verified.");
    } else {
        tx.blocking_write(b"FAIL: Mismatch detected!\r\n").unwrap();
        defmt::error!("FAIL: Mismatch detected!");
    }

    loop {
        cortex_m::asm::wfe();
    }
}
