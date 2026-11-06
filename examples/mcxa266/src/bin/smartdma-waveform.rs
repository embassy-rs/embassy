//! SmartDMA-generated nominal 5 kHz square wave on P0_4, camera J9 pin4.
//! Analyzer ground: J9 pin18. Disconnect camera/shields; use a 3.3V-compatible
//! high-impedance input. No external jumper is required for internal readback.
//! Program and resources are permanent: enter ISP before replacing firmware.
//! See examples/mcxa266/SMARTDMA.md for measurement steps and timing limits.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::Timer;
use hal::smartdma::{LoadedProgram, SmartDma, SmartDmaPin};
use panic_probe as _;
use static_cell::StaticCell;

static CODE: StaticCell<[u32; 64]> = StaticCell::new();
static STACK: StaticCell<[u32; 64]> = StaticCell::new();
static PARAMETERS: StaticCell<[u32; 3]> = StaticCell::new();
static PINS: StaticCell<[SmartDmaPin; 1]> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    let mut engine = SmartDma::new(p.SMART_DMA0).unwrap();
    let frequency = engine.frequency();
    let reload = frequency / 10_000;
    assert!((1..=65535).contains(&reload));
    let pin = engine.make_pin(p.P0_4).unwrap();
    let parameters = PARAMETERS.init([0, pin.mask(), reload]);
    let stack = STACK.init([0; 64]);
    let stack_address = stack.as_ptr() as u32;
    let parameter_address = parameters.as_ptr() as u32;
    let image = hal::smartdma_asm_file!("src/bin/smartdma-waveform.ezh");
    let loaded = LoadedProgram::load(&image, CODE.init([0; 64])).unwrap();
    defmt::info!(
        "SmartDMA: code={=u32:08x}, params={=u32:08x}, SP={=u32:08x}, clock={} Hz, reload={}",
        loaded.address(),
        parameter_address,
        stack_address,
        frequency,
        reload
    );
    defmt::info!("Output P0_4/PIO0: camera J9 pin4, ground J9 pin18; nominal 5kHz, 100us edge spacing");
    // SAFETY: this reviewed program initializes SP from the static parameter
    // block, reads only its three words, drives only owned PIO0, and writes only
    // the SmartDMA mailbox. Code/stack/parameters/pin ownership is permanent;
    // the CPU never mutates the installed code or parameters after launch.
    // Ordinary-SRAM instruction fetch is being validated by this bring-up.
    let running = unsafe { engine.start(loaded, "start", stack, parameters, PINS.init([pin])) }.unwrap();
    let mut previous = 0;
    let mut observed = false;
    for _ in 0..5 {
        Timer::after_millis(100).await;
        let status = running.status();
        let mut high = 0;
        let mut low = 0;
        for _ in 0..64 {
            if running.pin_is_high(0).unwrap() {
                high += 1;
            } else {
                low += 1;
            }
            Timer::after_micros(37).await;
        }
        let delta = status.notification.wrapping_sub(previous);
        observed |= status.sp == stack_address && delta != 0 && high != 0 && low != 0;
        defmt::info!(
            "PC={=u32:08x} SP={=u32:08x} CTRL={=u32:08x} mailbox={} delta={} pin samples high={} low={}",
            status.pc,
            status.sp,
            status.control,
            status.notification,
            delta,
            high,
            low
        );
        previous = status.notification;
    }
    if observed {
        defmt::info!("Stack/progress/pad activity observed; external analyzer measurement still required");
    } else {
        defmt::error!("Bring-up not validated; retain resources and enter ISP before reflashing");
    }
    core::future::pending::<()>().await;
}
