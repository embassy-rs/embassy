#![no_std]
#![no_main]
// #![feature(impl_trait_in_assoc_type)]

use defmt::Format;
use embassy_executor::Spawner;
use embassy_nrf::config::HfclkSource;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::nfc::{
    Config as NfcConfig, CrcMode, DiscardMode, Error as NfcError, FrameDelayConfig, NfcId, NfcT, RxdFrameConfig,
    ShortsConfig, TxdFrameConfig,
};
use embassy_nrf::{
    bind_interrupts,
    interrupt::{self, InterruptExt},
    nfc,
};
use embassy_time::Timer;
// use nrf_softdevice::{raw, Softdevice};
use {defmt_rtt as _, embassy_nrf as _, panic_probe as _};

#[macro_use]
extern crate defmt;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct AutocollConfig<'a> {
    pub uid: NfcId,
    pub atqa: [u8; 2],
    pub sak: u8,
    pub ats: Option<&'a [u8]>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum State {
    Idle,
    Halted,
    Ready,
    Active,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Format)]
enum CascadeLevel {
    Level1,
    Level2,
    Level3,
}

const NFC_TAG_14A_CMD_REQA: u8 = 0x26;
const NFC_TAG_14A_CMD_WUPA: u8 = 0x52;
const NFC_TAG_14A_CMD_HALT: u8 = 0x50;
const NFC_TAG_14A_CMD_RATS: u8 = 0xE0;

const NFC_TAG_14A_CMD_ANTICOLL_OR_SELECT_1: u8 = 0x93;
const NFC_TAG_14A_CMD_ANTICOLL_OR_SELECT_2: u8 = 0x95;
const NFC_TAG_14A_CMD_ANTICOLL_OR_SELECT_3: u8 = 0x97;

bind_interrupts!(struct Irqs {
    NFCT => nfc::InterruptHandler;
});

#[inline(always)]
fn calc_bcc(bytes: &mut [u8]) {
    let mut bcc = 0;

    let (last, bytes) = bytes.split_last_mut().unwrap();
    for byte in bytes {
        bcc ^= *byte;
    }

    *last = bcc;
}

// #[embassy_executor::task]
// async fn softdevice_task(sd: &'static Softdevice) -> ! {
//     sd.run().await
// }

async fn handle_nfc_comm(mut nfc: NfcT<'_>, mut led: Output<'_>, tx_rx_buf: &mut [u8]) {
    let autocoll_config = AutocollConfig {
        uid: NfcId::DoubleSize([0x04, 0x68, 0x95, 0x71, 0xFA, 0x5C, 0x64]),
        atqa: [0x44, 0x00],
        sak: 0x00,
        ats: None,
    };

    // nfc.setup_shorts(ShortsConfig {
    //     fielddetected_activate: false,
    //     fieldlost_sense: false,
    //     txframeend_enablerxdata: false,
    // });

    defmt::info!("hello");

    let mut state = State::Idle;
    let mut buf = [0u8; 256];

    // let memory = &[0x04, 0x68, 0x95, 0x71, 0xFA, 0x5C, 0x64, 0x80, 0x42, 0x48, 0x00, 0x00];

    debug!("activating");
    nfc.activate().await;

    // give it some time
    Timer::after_millis(30).await;
    info!("NFCT is active");

    loop {
        trace!("Waiting for field");
        nfc.wait_for_active().await;

        trace!("Got field, trying rx");
        let result = nfc
            .recv_frame2(
                &mut buf,
                RxdFrameConfig {
                    crc_mode: CrcMode::NoCrc,
                    ..Default::default()
                },
            )
            .await;

        let (bytes, odd_bits) = match result {
            Ok(frame) => frame,
            Err(e) => {
                error!("other error {}", e);
                continue;
            }
        };

        trace!("received frame {=[u8]:#x} odd bits {=u8}", bytes, odd_bits);
        todo!("RX finished now we should tx probably");
    }

    #[cfg(feature = "old")]
    loop {
        let result = nfc
            .recv_frame_with_cfg(
                &mut buf,
                RxdFrameConfig {
                    crc_mode: CrcMode::NoCrc,
                    ..Default::default()
                },
            )
            .await;

        let (bytes, odd_bits) = match result {
            Ok(frame) => frame,
            Err(NfcError::LostField) => {
                warn!("lost field, going to idle");

                state = State::Idle;
                continue;
            }
            Err(e) => {
                error!("other error {}", e);
                continue;
            }
        };

        trace!("received frame {=[u8]:#x} odd bits {=u8}", bytes, odd_bits);

        if bytes.is_empty() {
            warn!("empty frame received, ignoring");
            continue;
        }

        let cmd = bytes[0];
        if cmd == NFC_TAG_14A_CMD_WUPA || (cmd == NFC_TAG_14A_CMD_REQA && state == State::Halted) {
            match nfc
                .tx_frame_with_config(
                    &autocoll_config.atqa,
                    0,
                    TxdFrameConfig {
                        crc_mode: CrcMode::NoCrc,
                        ..Default::default()
                    },
                )
                .await
            {
                Ok(()) => {
                    debug!("sent atqa, transitioned to ready");

                    state = State::Ready;
                    continue;
                }
                Err(NfcError::LostField) => {
                    warn!("lost field, going to idle");

                    state = State::Idle;
                    continue;
                }
                Err(e) => {
                    error!("other error {}", e);
                    continue;
                }
            }
        }

        match state {
            State::Halted | State::Idle | State::Ready => {
                if bytes.len() < 2 {
                    error!("too short command {=u8:#02x} in ready state", bytes.len() as u8);
                    state = State::Idle;
                    continue;
                }

                let level = match cmd {
                    NFC_TAG_14A_CMD_ANTICOLL_OR_SELECT_1 => CascadeLevel::Level1,
                    NFC_TAG_14A_CMD_ANTICOLL_OR_SELECT_2 => CascadeLevel::Level2,
                    NFC_TAG_14A_CMD_ANTICOLL_OR_SELECT_3 => CascadeLevel::Level3,
                    NFC_TAG_14A_CMD_HALT if bytes[1] == 0 => {
                        debug!("acking halt");
                        nfc.tx_frame_with_config(
                            &[0xAA],
                            4,
                            TxdFrameConfig {
                                crc_mode: CrcMode::NoCrc,
                                ..Default::default()
                            },
                        )
                        .await
                        .unwrap();
                        state = State::Halted;
                        continue;
                    }
                    other => {
                        error!("unexpected command {=u8:#02x} in ready state", other);
                        state = State::Idle;
                        continue;
                    }
                };

                debug!("cascade level {:?}", level);

                let mut uid = [0u8; 5];
                match &autocoll_config.uid {
                    NfcId::SingleSize(bytes) => match level {
                        CascadeLevel::Level1 => {
                            uid[..4].copy_from_slice(bytes);
                        }
                        _ => {
                            error!("invalid cascade level for 4-byte uid");
                            state = State::Idle;
                            continue;
                        }
                    },
                    NfcId::DoubleSize(bytes) => match level {
                        CascadeLevel::Level1 => {
                            uid[0] = 0x88;
                            uid[1..4].copy_from_slice(&bytes[..3]);
                        }
                        CascadeLevel::Level2 => {
                            uid[..4].copy_from_slice(&bytes[3..]);
                        }
                        _ => {
                            error!("invalid cascade level for 4-byte uid");
                            state = State::Idle;
                            continue;
                        }
                    },
                    NfcId::TripleSize(bytes) => match level {
                        CascadeLevel::Level1 => {
                            uid[0] = 0x88;
                            uid[1..4].copy_from_slice(&bytes[..3]);
                        }
                        CascadeLevel::Level2 => {
                            uid[0] = 0x88;
                            uid[1..4].copy_from_slice(&bytes[3..6]);
                        }
                        CascadeLevel::Level3 => {
                            uid[..4].copy_from_slice(&bytes[6..]);
                        }
                    },
                }
                calc_bcc(&mut uid);

                if bytes[1] == 0x20 {
                    // SELECT ALL
                    match nfc
                        .tx_frame_with_config(
                            &uid,
                            0,
                            TxdFrameConfig {
                                crc_mode: CrcMode::NoCrc,
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        Ok(()) => debug!("send select all response"),
                        Err(NfcError::LostField) => {
                            warn!("lost field");
                            state = State::Idle;
                        }
                        Err(e) => {
                            warn!("other tx error {}", e);
                        }
                    }
                    continue;
                } else if bytes.len() == 9 && odd_bits == 0 && bytes[1] == 0x70 && bytes[2..6] == uid[..4] {
                    // Incoming SELECT CLx for any cascade level
                    let cl_finished = match (&autocoll_config.uid, level) {
                        (NfcId::SingleSize(_), CascadeLevel::Level1)
                        | (NfcId::DoubleSize(_), CascadeLevel::Level2)
                        | (NfcId::TripleSize(_), CascadeLevel::Level3) => true,
                        _ => false,
                    };

                    let result = if cl_finished {
                        state = State::Active;
                        nfc.tx_frame(&[autocoll_config.sak], 0).await
                    } else {
                        nfc.tx_frame_with_config(
                            &[0x04, 0xda, 0x17],
                            0,
                            TxdFrameConfig {
                                crc_mode: CrcMode::NoCrc,
                                ..Default::default()
                            },
                        )
                        .await
                    };

                    match result {
                        Ok(()) => (),
                        Err(NfcError::LostField) => {
                            state = State::Idle;
                        }
                        Err(_) => (),
                    }
                    continue;
                } else {
                    state = State::Idle;
                    continue;
                }
            }
            State::Active => {}
        }
    }

    #[cfg(feature = "col")]
    'coll_loop: loop {
        led.set_high();

        info!("waiting for coll");
        match nfc.wait_for_coll().await {
            Ok(()) => (),
            Err(NfcError::Collision) => {
                nfc.sleep();
                warn!("collision detected, retrying autocollision");
                continue 'coll_loop;
            }
            Err(_) => {
                warn!("lost field");
                nfc.idle();
                continue 'coll_loop;
            }
        }

        led.set_low();

        info!("coll");
        let field = nfc.is_field_present();
        info!("Field present: {=bool}", field);

        'selected_loop: loop {
            dbg!("Starting recv_frame");
            let frame = nfc.recv_frame(&mut buf).await;
            if let Ok((data, odd_bits)) = frame {
                // Stop tx probably.

                info!("Got frame: {=[u8]:02x} {}", data, odd_bits);

                let mut resp = [0x00; 16];
                match data[0] {
                    0x30 => {
                        // emulate read
                        let block_num = data[1] as usize;
                        for i in 0..4 {
                            let block_num = (block_num + i) % 16;
                            if block_num < 3 {
                                resp[i * 4..i * 4 + 4].copy_from_slice(&memory[block_num..block_num + 4]);
                            } else {
                                resp[i * 4..i * 4 + 4].fill(0);
                            }
                        }

                        match nfc.tx_frame(&resp, 0).await {
                            Ok(()) => info!("emulated read"),
                            Err(NfcError::LostField) => {
                                warn!("lost field");
                                nfc.idle();
                                continue 'coll_loop;
                            }
                            Err(e) => info!("write error {}", e),
                        }
                    }
                    0x52 => {
                        resp[1] = 0x44;
                        match nfc.tx_frame(&resp[..2], 0).await {
                            Ok(()) => info!("sent SAK"),
                            Err(NfcError::LostField) => {
                                warn!("lost field");
                                nfc.idle();
                                continue 'coll_loop;
                            }
                            Err(e) => info!("write error {}", e),
                        }
                    }
                    0x50 => {
                        resp[0] = 0;
                        match nfc
                            .tx_frame_with_config(
                                &resp[..1],
                                4,
                                TxdFrameConfig {
                                    parity: true,
                                    discard_mode: DiscardMode::DiscardStart,
                                    add_sof: true,
                                    crc_mode: CrcMode::NoCrc,
                                },
                            )
                            .await
                        {
                            Ok(()) => {
                                nfc.sleep();
                                info!("entered sleep_a");
                                break 'selected_loop;
                            }
                            Err(NfcError::LostField) => {
                                warn!("lost field");
                                nfc.idle();
                                continue 'coll_loop;
                            }
                            Err(e) => info!("write error {}", e),
                        }
                    }
                    _ => {
                        resp[0] = 0;
                        match nfc
                            .tx_frame_with_config(
                                &resp[..1],
                                4,
                                TxdFrameConfig {
                                    parity: false,
                                    discard_mode: DiscardMode::DiscardStart,
                                    add_sof: false,
                                    crc_mode: CrcMode::NoCrc,
                                },
                            )
                            .await
                        {
                            Ok(()) => {
                                nfc.sleep();
                                info!("entered sleep_a");
                                break 'selected_loop;
                            }
                            Err(NfcError::LostField) => {
                                warn!("lost field");
                                nfc.idle();
                                continue 'coll_loop;
                            }
                            Err(e) => info!("write error {}", e),
                        }
                    }
                }
            } else if let Err(e) = frame {
                if e == NfcError::LostField {
                    warn!("lost field");
                    nfc.idle();
                    continue 'coll_loop;
                }
                dbg!("Got frame err: {}", e);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    config.gpiote_interrupt_priority = interrupt::Priority::P2;
    config.time_interrupt_priority = interrupt::Priority::P2;
    let p = embassy_nrf::init(config);

    let mut tx_rx_buf = [0u8; 256];

    interrupt::NFCT.set_priority(interrupt::Priority::P6);

    // let config = nrf_softdevice::Config {
    //     clock: Some(raw::nrf_clock_lf_cfg_t {
    //         source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
    //         rc_ctiv: 0,
    //         rc_temp_ctiv: 0,
    //         accuracy: raw::NRF_CLOCK_LF_ACCURACY_20_PPM as u8,
    //     }),
    //     ..Default::default()
    // };

    // let sd = Softdevice::enable(&config);
    // unwrap!(spawner.spawn(softdevice_task(sd)));

    dbg!("Setting up...");
    let config = NfcConfig {
        autocoll_config: /*Some(AutoCollConfig {
            nfcid1: NfcId::DoubleSize([0x04, 0x68, 0x95, 0x71, 0xFA, 0x5C, 0x64]),
            sdd_pat: SddPat::Sdd00100,
            plat_conf: 0b0000,
            protocol: SelResProtocol::Type2,
        })*/ None,
        txd_frame_config: TxdFrameConfig::default(),
        rxd_frame_config: RxdFrameConfig::default(),
        frame_delay_config: FrameDelayConfig::WindowGrid(0x480..4096),
    };

    let led = Output::new(p.P1_01, Level::High, OutputDrive::Standard);
    let _hf_ant_sel = Output::new(p.P1_10, Level::High, OutputDrive::Standard);

    let nfc = NfcT::new(p.NFCT, Irqs, &config);
    dbg!("Set up finished, going into nfc loop");
    handle_nfc_comm(nfc, led, &mut tx_rx_buf).await;
}
