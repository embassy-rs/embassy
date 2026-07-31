use defmt::info;
use embassy_futures::join::join;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::driver::Driver;

use crate::Disconnected;
use crate::throughput::{Event, Parser};

pub type Params = crate::UsbParams;

pub type Resources<'d> = crate::UsbResources<'d>;

pub async fn run<
    'd,
    D: Driver<'d>,
    const MPS: usize,
    const TX_LEN: usize,
    const IN_CHUNK: usize,
    const OUT_CHUNK: usize,
>(
    driver: D,
    resources: &'d mut Resources<'d>,
    params: Params,
) {
    let (mut usb, mut class) = crate::cdc(driver, resources, params, MPS as u16);

    let benchmark = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = bench::<D, MPS, TX_LEN, IN_CHUNK, OUT_CHUNK>(&mut class).await;
            info!("Disconnected");
        }
    };

    join(usb.run(), benchmark).await;
}

async fn bench<
    'd,
    D: Driver<'d>,
    const MPS: usize,
    const TX_LEN: usize,
    const IN_CHUNK: usize,
    const OUT_CHUNK: usize,
>(
    class: &mut CdcAcmClass<'d, D>,
) -> Result<(), Disconnected> {
    let mut tx = [0u8; TX_LEN];
    for (index, byte) in tx.iter_mut().enumerate() {
        *byte = index as u8;
    }
    let mut buf = [0u8; OUT_CHUNK];
    let mut parser = Parser::new();

    loop {
        let len = class.read_packet(&mut buf).await?;
        let mut offset = 0;
        while offset < len {
            let feed = parser.feed(&buf[offset..len]);
            offset += feed.consumed;

            match feed.event {
                Some(Event::In(total)) => {
                    info!("IN test: {} bytes", total);
                    let mut remaining = total;
                    let mut ramp_offset = 0;
                    while remaining != 0 {
                        let chunk = remaining.min(IN_CHUNK as u32).min((TX_LEN - ramp_offset) as u32) as usize;
                        class.write_packet(&tx[ramp_offset..ramp_offset + chunk]).await?;
                        ramp_offset = (ramp_offset + chunk) % TX_LEN;
                        remaining -= chunk as u32;
                    }
                    if total % MPS as u32 == 0 {
                        class.write_packet(&[]).await?;
                    }
                }
                Some(Event::OutStarted(total)) => info!("OUT test: {} bytes", total),
                Some(Event::OutComplete(total)) => class.write_packet(&total.to_le_bytes()).await?,
                Some(Event::Unknown(command)) => info!("unknown command {}", command),
                None => {}
            }
        }
    }
}
