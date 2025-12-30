//! This example targets the NUCLEO-H563ZI platform.
//! USB-C CC lines are protected by a TCPP01-M12 chipset.
#![no_std]
#![no_main]

use defmt::{Format, error, info};
use embassy_executor::Spawner;
use embassy_stm32::gpio::Output;
use embassy_stm32::ucpd::{self, CcPhy, CcPull, CcSel, CcVState, Ucpd};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_time::{Duration, with_timeout};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UCPD1 => ucpd::InterruptHandler<peripherals::UCPD1>;
});

#[derive(Debug, Format)]
enum CableOrientation {
    Normal,
    Flipped,
    DebugAccessoryMode,
}

// Returns true when the cable
async fn wait_attached<T: ucpd::Instance>(cc_phy: &mut CcPhy<'_, T>) -> CableOrientation {
    loop {
        let (cc1, cc2) = cc_phy.vstate();
        if cc1 == CcVState::LOWEST && cc2 == CcVState::LOWEST {
            // Detached, wait until attached by monitoring the CC lines.
            cc_phy.wait_for_vstate_change().await;
            continue;
        }

        // Attached, wait for CC lines to be stable for tCCDebounce (100..200ms).
        if with_timeout(Duration::from_millis(100), cc_phy.wait_for_vstate_change())
            .await
            .is_ok()
        {
            // State has changed, restart detection procedure.
            continue;
        };

        // State was stable for the complete debounce period, check orientation.
        return match (cc1, cc2) {
            (_, CcVState::LOWEST) => CableOrientation::Normal,  // CC1 connected
            (CcVState::LOWEST, _) => CableOrientation::Flipped, // CC2 connected
            _ => CableOrientation::DebugAccessoryMode,          // Both connected (special cable)
        };
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut ucpd = Ucpd::new(p.UCPD1, Irqs {}, p.PB13, p.PB14, Default::default());
    ucpd.cc_phy().set_pull(CcPull::Sink);

    // This pin controls the dead-battery mode on the attached TCPP01-M12.
    // If low, TCPP01-M12 disconnects CC lines and presents dead-battery resistance on CC lines, thus set high.
    // Must only be set after the CC pull is established.
    let _tcpp01_m12_ndb = Output::new(p.PA9, embassy_stm32::gpio::Level::High, embassy_stm32::gpio::Speed::Low);

    info!("Waiting for USB connection...");
    let cable_orientation = wait_attached(ucpd.cc_phy()).await;
    info!("USB cable connected, orientation: {}", cable_orientation);

    let cc_sel = match cable_orientation {
        CableOrientation::Normal => {
            info!("Starting PD communication on CC1 pin");
            CcSel::CC1
        }
        CableOrientation::Flipped => {
            info!("Starting PD communication on CC2 pin");
            CcSel::CC2
        }
        CableOrientation::DebugAccessoryMode => panic!("No PD communication in DAM"),
    };
    let (_cc_phy, mut pd_phy) = ucpd.split_pd_phy(p.GPDMA1_CH0, p.GPDMA1_CH1, cc_sel);

    loop {
        // Enough space for the longest non-extended data message.
        let mut buf = [0_u8; 30];
        match pd_phy.receive(buf.as_mut()).await {
            Ok(n) => info!("USB PD RX: {=[u8]:?}", &buf[..n]),
            Err(e) => error!("USB PD RX: {}", e),
        }
    }
}
