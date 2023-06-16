use crate::ble::Ble;
use crate::consts::TlPacketType;
use crate::{shci, TlMbox, STATE};

pub struct RadioCoprocessor<'d> {
    mbox: TlMbox<'d>,
    rx_buf: [u8; 500],
}

impl<'d> RadioCoprocessor<'d> {
    pub fn new(mbox: TlMbox<'d>) -> Self {
        Self {
            mbox,
            rx_buf: [0u8; 500],
        }
    }

    pub fn write(&self, buf: &[u8]) {
        let cmd_code = buf[0];
        let cmd = TlPacketType::try_from(cmd_code).unwrap();

        match &cmd {
            TlPacketType::BleCmd => Ble::send_cmd(buf),
            _ => todo!(),
        }
    }

    pub async fn read(&mut self) -> &[u8] {
        loop {
            STATE.wait().await;

            while let Some(evt) = self.mbox.dequeue_event() {
                let event = evt.evt();

                evt.write(&mut self.rx_buf).unwrap();
            }

            if self.mbox.pop_last_cc_evt().is_some() {
                continue;
            }

            return &self.rx_buf;
        }
    }
}
