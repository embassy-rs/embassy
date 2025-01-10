#![no_std]
#![no_main]

use defmt::{todo, *};
use embassy_executor::Spawner;
use embassy_nrf::config::HfclkSource;
use embassy_nrf::nfct::{Config as NfcConfig, NfcId, NfcT};
use embassy_nrf::{bind_interrupts, nfct};
use iso14443_4::{Card, IsoDep};
use {defmt_rtt as _, embassy_nrf as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    NFCT => nfct::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    let p = embassy_nrf::init(config);

    dbg!("Setting up...");
    let config = NfcConfig {
        nfcid1: NfcId::DoubleSize([0x04, 0x68, 0x95, 0x71, 0xFA, 0x5C, 0x64]),
        sdd_pat: nfct::SddPat::SDD00100,
        plat_conf: 0b0000,
        protocol: nfct::SelResProtocol::Type4A,
    };

    let mut nfc = NfcT::new(p.NFCT, Irqs, &config);

    let mut buf = [0u8; 256];

    let cc = &[
        0x00, 0x0f, /* CCEN_HI, CCEN_LOW */
        0x20, /* VERSION */
        0x00, 0x7f, /* MLe_HI, MLe_LOW */
        0x00, 0x7f, /* MLc_HI, MLc_LOW */
        /* TLV */
        0x04, 0x06, 0xe1, 0x04, 0x00, 0x7f, 0x00, 0x00,
    ];

    let ndef = &[
        0x00, 0x10, 0xd1, 0x1, 0xc, 0x55, 0x4, 0x65, 0x6d, 0x62, 0x61, 0x73, 0x73, 0x79, 0x2e, 0x64, 0x65, 0x76,
    ];
    let mut selected: &[u8] = cc;

    loop {
        info!("activating");
        nfc.activate().await;
        info!("activated!");

        let mut nfc = IsoDep::new(iso14443_3::Logger(&mut nfc));

        loop {
            let n = match nfc.receive(&mut buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("rx error {}", e);
                    break;
                }
            };
            let req = &buf[..n];
            info!("iso-dep rx {:02x}", req);

            let Ok(apdu) = Apdu::parse(req) else {
                error!("apdu parse error");
                break;
            };

            info!("apdu: {:?}", apdu);

            let resp = match (apdu.cla, apdu.ins, apdu.p1, apdu.p2) {
                (0, 0xa4, 4, 0) => {
                    info!("select app");
                    &[0x90, 0x00][..]
                }
                (0, 0xa4, 0, 12) => {
                    info!("select df");
                    match apdu.data {
                        [0xe1, 0x03] => {
                            selected = cc;
                            &[0x90, 0x00][..]
                        }
                        [0xe1, 0x04] => {
                            selected = ndef;
                            &[0x90, 0x00][..]
                        }
                        _ => todo!(), // return NOT FOUND
                    }
                }
                (0, 0xb0, p1, p2) => {
                    info!("read");
                    let offs = u16::from_be_bytes([p1 & 0x7f, p2]) as usize;
                    let len = if apdu.le == 0 { usize::MAX } else { apdu.le as usize };
                    let n = len.min(selected.len() - offs);
                    buf[..n].copy_from_slice(&selected[offs..][..n]);
                    buf[n..][..2].copy_from_slice(&[0x90, 0x00]);
                    &buf[..n + 2]
                }
                _ => {
                    info!("Got unknown command!");
                    &[0xFF, 0xFF]
                }
            };

            info!("iso-dep tx {:02x}", resp);

            match nfc.transmit(resp).await {
                Ok(()) => {}
                Err(e) => {
                    error!("tx error {}", e);
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone, defmt::Format)]
struct Apdu<'a> {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: &'a [u8],
    pub le: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
struct ApduParseError;

impl<'a> Apdu<'a> {
    pub fn parse(apdu: &'a [u8]) -> Result<Self, ApduParseError> {
        if apdu.len() < 4 {
            return Err(ApduParseError);
        }

        let (data, le) = match apdu.len() - 4 {
            0 => (&[][..], 0),
            1 => (&[][..], apdu[4]),
            n if n == 1 + apdu[4] as usize && apdu[4] != 0 => (&apdu[5..][..apdu[4] as usize], 0),
            n if n == 2 + apdu[4] as usize && apdu[4] != 0 => (&apdu[5..][..apdu[4] as usize], apdu[apdu.len() - 1]),
            _ => return Err(ApduParseError),
        };

        Ok(Apdu {
            cla: apdu[0],
            ins: apdu[1],
            p1: apdu[2],
            p2: apdu[3],
            data,
            le: le as _,
        })
    }
}

mod iso14443_3 {
    use core::future::Future;

    use defmt::info;
    use embassy_nrf::nfct::{Error, NfcT};

    pub trait Card {
        type Error;
        async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
        async fn transmit(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
    }

    impl<'a, T: Card> Card for &'a mut T {
        type Error = T::Error;

        fn receive(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, Self::Error>> {
            T::receive(self, buf)
        }

        fn transmit(&mut self, buf: &[u8]) -> impl Future<Output = Result<(), Self::Error>> {
            T::transmit(self, buf)
        }
    }

    impl<'a> Card for NfcT<'a> {
        type Error = Error;

        fn receive(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, Self::Error>> {
            self.receive(buf)
        }

        fn transmit(&mut self, buf: &[u8]) -> impl Future<Output = Result<(), Self::Error>> {
            self.transmit(buf)
        }
    }

    pub struct Logger<T: Card>(pub T);

    impl<T: Card> Card for Logger<T> {
        type Error = T::Error;

        async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let n = T::receive(&mut self.0, buf).await?;
            info!("<- {:02x}", &buf[..n]);
            Ok(n)
        }

        fn transmit(&mut self, buf: &[u8]) -> impl Future<Output = Result<(), Self::Error>> {
            info!("-> {:02x}", buf);
            T::transmit(&mut self.0, buf)
        }
    }
}

mod iso14443_4 {
    use defmt::info;

    use crate::iso14443_3;

    #[derive(defmt::Format)]
    pub enum Error<T> {
        Deselected,
        Protocol,
        Lower(T),
    }

    pub trait Card {
        type Error;
        async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
        async fn transmit(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
    }

    pub struct IsoDep<T: iso14443_3::Card> {
        nfc: T,

        /// Block count spin bit: 0 or 1
        block_num: u8,

        /// true if deselected. This is permanent, you must create another IsoDep
        /// instance if we get selected again.
        deselected: bool,

        /// last response, in case we need to retransmit.
        resp: [u8; 256],
        resp_len: usize,
    }

    impl<T: iso14443_3::Card> IsoDep<T> {
        pub fn new(nfc: T) -> Self {
            Self {
                nfc,
                block_num: 1,
                deselected: false,
                resp: [0u8; 256],
                resp_len: 0,
            }
        }
    }

    impl<T: iso14443_3::Card> Card for IsoDep<T> {
        type Error = Error<T::Error>;

        async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            if self.deselected {
                return Err(Error::Deselected);
            }

            let mut temp = [0u8; 256];

            loop {
                let n = self.nfc.receive(&mut temp).await.map_err(Error::Lower)?;
                assert!(n != 0);
                match temp[0] {
                    0x02 | 0x03 => {
                        self.block_num ^= 0x01;
                        assert!(temp[0] == 0x02 | self.block_num);
                        buf[..n - 1].copy_from_slice(&temp[1..n]);
                        return Ok(n - 1);
                    }
                    0xb2 | 0xb3 => {
                        if temp[0] & 0x01 != self.block_num {
                            info!("Got NAK, transmitting ACK.");
                            let resp = &[0xA2 | self.block_num];
                            self.nfc.transmit(resp).await.map_err(Error::Lower)?;
                        } else {
                            info!("Got NAK, retransmitting.");
                            let resp: &[u8] = &self.resp[..self.resp_len];
                            self.nfc.transmit(resp).await.map_err(Error::Lower)?;
                        }
                    }
                    0xe0 => {
                        info!("Got RATS, tx'ing ATS");
                        let resp = &[0x06, 0x77, 0x77, 0x81, 0x02, 0x80];
                        self.nfc.transmit(resp).await.map_err(Error::Lower)?;
                    }
                    0xc2 => {
                        info!("Got deselect!");
                        self.deselected = true;
                        let resp = &[0xC2];
                        self.nfc.transmit(resp).await.map_err(Error::Lower)?;
                        return Err(Error::Deselected);
                    }
                    _ => {
                        info!("Got unknown command {:02x}!", temp[0]);
                        return Err(Error::Protocol);
                    }
                };
            }
        }

        async fn transmit(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            if self.deselected {
                return Err(Error::Deselected);
            }

            self.resp[0] = 0x02 | self.block_num;
            self.resp[1..][..buf.len()].copy_from_slice(buf);
            self.resp_len = 1 + buf.len();

            let resp: &[u8] = &self.resp[..self.resp_len];
            self.nfc.transmit(resp).await.map_err(Error::Lower)?;

            Ok(())
        }
    }
}
