#![no_std]
#![no_main]

#[cfg(feature = "mspm0g3507")]
teleprobe_meta::target!(b"lp-mspm0g3507");

use defmt::{assert_eq, unwrap, *};
use embassy_executor::Spawner;
use embassy_mspm0::mode::Blocking;
use embassy_mspm0::uart::{ClockSel, Config, Error, Uart};
use {defmt_rtt as _, panic_probe as _};

fn read<const N: usize>(uart: &mut Uart<'_, Blocking>) -> Result<[u8; N], Error> {
    let mut buf = [255; N];
    uart.blocking_read(&mut buf)?;
    Ok(buf)
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_mspm0::init(Default::default());
    info!("Hello World!");

    // TODO: Allow creating a looped-back UART (so pins are not needed).
    // Do not select default UART since the virtual COM port is attached to UART0.
    #[cfg(feature = "mspm0g3507")]
    let (mut tx, mut rx, mut uart) = (p.PA8, p.PA9, p.UART1);

    const MFCLK_BUAD_RATES: &[u32] = &[1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200];

    for &rate in MFCLK_BUAD_RATES {
        info!("{} baud using MFCLK", rate);

        let mut config = Config::default();
        // MSPM0 hardware supports a loopback mode to allow self test.
        config.loop_back_enable = true;
        config.baudrate = rate;

        let mut uart = unwrap!(Uart::new_blocking(
            uart.reborrow(),
            rx.reborrow(),
            tx.reborrow(),
            config
        ));

        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        let data = [0xC0, 0xDE];
        unwrap!(uart.blocking_write(&data));
        assert_eq!(unwrap!(read(&mut uart)), data);
    }

    // 9600 is the maximum possible value for 32.768 kHz.
    const LFCLK_BAUD_RATES: &[u32] = &[1200, 2400, 4800, 9600];

    for &rate in LFCLK_BAUD_RATES {
        info!("{} baud using LFCLK", rate);

        let mut config = Config::default();
        // MSPM0 hardware supports a loopback mode to allow self test.
        config.loop_back_enable = true;
        config.baudrate = rate;
        config.clock_source = ClockSel::LfClk;

        let mut uart = expect!(Uart::new_blocking(
            uart.reborrow(),
            rx.reborrow(),
            tx.reborrow(),
            config,
        ));

        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        let data = [0xC0, 0xDE];
        unwrap!(uart.blocking_write(&data));
        assert_eq!(unwrap!(read(&mut uart)), data);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
