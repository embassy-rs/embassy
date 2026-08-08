use defmt::info;
use embassy_futures::join::join;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::driver::Driver;

use crate::Disconnected;

pub type Params = crate::UsbParams;

pub type Resources<'d> = crate::UsbResources<'d>;

pub async fn run<'d, D: Driver<'d>, const MPS: usize>(
    driver: D,
    resources: &'d mut Resources<'d>,
    params: Params,
    connected: &'static str,
    disconnected: &'static str,
) {
    let (mut usb, mut class) = crate::cdc(driver, resources, params, MPS as u16);

    let echo = async {
        loop {
            class.wait_connection().await;
            info!("{}", connected);
            let _ = echo::<D, MPS>(&mut class).await;
            info!("{}", disconnected);
        }
    };

    join(usb.run(), echo).await;
}

async fn echo<'d, D: Driver<'d>, const MPS: usize>(class: &mut CdcAcmClass<'d, D>) -> Result<(), Disconnected> {
    let mut buf = [0; MPS];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
        // A full packet needs a zero-length packet to terminate the transfer.
        if n == MPS {
            class.write_packet(&[]).await?;
        }
    }
}
