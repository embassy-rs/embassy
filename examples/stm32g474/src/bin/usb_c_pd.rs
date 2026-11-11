//! USB Type-C / Power Delivery sink on a NUCLEO-G474RE with an X-NUCLEO-DRP1M1 expansion board.
//!
//! The expansion board is required: it provides the USB-C connector and routes CC1/CC2 to the
//! UCPD pins through a TCPP03-M20 protection IC. The TCPP03 is enabled and put into Normal mode
//! over I2C at startup; until then its CC switches are open and UCPD can't see the CC lines.
//!
//! The example detects attach and detach in a loop and enables PD on every attach, without
//! recreating the UCPD driver.

#![no_std]
#![no_main]

use defmt::{Format, error, info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::ucpd::{self, CcPhy, CcPull, CcSel, CcVState, PdPhy, Ucpd};
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use embassy_time::{Duration, Timer, with_timeout};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    UCPD1 => ucpd::InterruptHandler<peripherals::UCPD1>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_CHANNEL5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
    DMA1_CHANNEL6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
});

/// I2C address of the TCPP03-M20 on the X-NUCLEO-DRP1M1.
const TCPP03_ADDR: u8 = 0x34;

#[derive(Debug, Format)]
enum CableOrientation {
    Normal,
    Flipped,
    DebugAccessoryMode,
}

// Returns the cable orientation once a cable is attached.
async fn wait_attached<T: ucpd::Instance>(cc_phy: &mut CcPhy<'_, T>) -> CableOrientation {
    loop {
        let (cc1, cc2) = cc_phy.vstate();
        if cc1 == CcVState::Lowest && cc2 == CcVState::Lowest {
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
            (_, CcVState::Lowest) => CableOrientation::Normal,  // CC1 connected
            (CcVState::Lowest, _) => CableOrientation::Flipped, // CC2 connected
            _ => CableOrientation::DebugAccessoryMode,          // Both connected (special cable)
        };
    }
}

// Returns once the cable is detached.
async fn wait_detached<T: ucpd::Instance>(cc_phy: &mut CcPhy<'_, T>) {
    loop {
        let (cc1, cc2) = cc_phy.vstate();
        if cc1 == CcVState::Lowest && cc2 == CcVState::Lowest {
            return;
        }
        cc_phy.wait_for_vstate_change().await;
    }
}

async fn receive_messages<T: ucpd::Instance>(pd_phy: &mut PdPhy<'_, T>) {
    loop {
        // Enough space for the longest non-extended data message.
        let mut buf = [0_u8; 30];
        match pd_phy.receive(buf.as_mut()).await {
            Ok(n) => info!("USB PD RX: {=[u8]:?}", &buf[..n]),
            Err(e) => error!("USB PD RX: {}", e),
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.enable_ucpd1_dead_battery = true;
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    // Bring up the TCPP03-M20: EN = PC8, I2C1 on PB8 (SCL) / PB9 (SDA).
    // Keep `tcpp_en` alive: dropping the Output releases PC8, which turns the TCPP03 off.
    let mut tcpp_en = Output::new(p.PC8, Level::Low, Speed::Low);
    let mut tcpp_i2c = I2c::new(p.I2C1, p.PB8, p.PB9, p.DMA1_CH5, p.DMA1_CH6, Irqs, Default::default());
    Timer::after_millis(10).await;
    tcpp_en.set_high(); // EN low -> high: reset, I2C becomes active
    Timer::after_millis(10).await;

    // Register 0: PM = 01 (Normal, CC switches closed, BMC passes through),
    // consumer gate open (bit 3 = 1), provider gate open, Vconn open.
    unwrap!(tcpp_i2c.write(TCPP03_ADDR, &[0x00, 0x18]).await);
    info!("TCPP03 in Normal mode");

    let mut ucpd = Ucpd::new(p.UCPD1, p.PB6, p.PB4, Irqs {}, Default::default());
    ucpd.cc_phy().set_pull(CcPull::Sink);

    // Both parts are kept across attach/detach cycles; PD is only enabled while attached.
    let (mut cc_phy, mut pd_phy) = ucpd.split_idle_pd_phy(p.DMA1_CH1, p.DMA1_CH2, Irqs);

    loop {
        info!("Waiting for USB connection...");
        let cable_orientation = wait_attached(&mut cc_phy).await;
        info!("USB cable connected, orientation: {}", cable_orientation);

        let cc_sel = match cable_orientation {
            CableOrientation::Normal => CcSel::Cc1,
            CableOrientation::Flipped => CcSel::Cc2,
            CableOrientation::DebugAccessoryMode => {
                info!("No PD communication in DAM");
                wait_detached(&mut cc_phy).await;
                continue;
            }
        };

        info!("Starting PD communication on {}", cc_sel);
        let mut active_pd_phy = pd_phy.activate(cc_sel);
        match select(wait_detached(&mut cc_phy), receive_messages(&mut active_pd_phy)).await {
            Either::First(()) => {}
            Either::Second(()) => {
                // A PD stack could end the session, e.g. if the partner doesn't support PD.
                // The cable is still attached, so wait for the detach before starting over.
                info!("PD session ended, waiting for detach");
                drop(active_pd_phy);
                wait_detached(&mut cc_phy).await;
            }
        }

        // Dropping `active_pd_phy` disables PD communication until the next attach.
        info!("USB cable disconnected");
    }
}
