use embassy_futures::select::{select3, Either3};

use crate::mac::MTU;
use crate::sub::mac::Mac;

pub struct Runner {
    mac: Mac,
    // TODO: tx_ring
    // TODO: rx_buf
}

impl Runner {
    pub(crate) fn new(mac: Mac) -> Self {
        Self { mac }
    }

    pub(crate) async fn init(&mut self, firmware: &[u8]) {
        debug!("wifi init done");
    }

    pub async fn run(mut self) -> ! {
        let mut buf = [0; 512];
        loop {
            // TODO
        }
    }
}
