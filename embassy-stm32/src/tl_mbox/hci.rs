use super::ble::Ble;
use super::consts::TlPacketType;
use super::evt::CcEvt;
use super::shci::{shci_ble_init, ShciBleInitCmdParam};
use super::{TlMbox, STATE};

pub struct RadioCoprocessor<'d> {
    mbox: TlMbox<'d>,
    config: ShciBleInitCmdParam,
    rx_buffer: [u8; 500],
}

impl<'d> RadioCoprocessor<'d> {
    pub fn new(mbox: TlMbox<'d>, config: ShciBleInitCmdParam) -> Self {
        Self {
            mbox,
            config,
            rx_buffer: [0u8; 500],
        }
    }

    pub fn write(&mut self, params: &[u8]) -> Result<(), ()> {
        let cmd_code = params[0];
        let cmd = TlPacketType::try_from(cmd_code)?;

        match cmd {
            TlPacketType::BleCmd => Ble::send_cmd(params),
            _ => todo!(),
        }

        Ok(())
    }

    pub async fn read(&mut self) -> &[u8] {
        self.rx_buffer = [0u8; 500];

        loop {
            STATE.wait().await;

            if let Some(evt) = self.mbox.dequeue_event() {
                let event = evt.evt();
                evt.write(&mut self.rx_buffer).unwrap();

                if event.kind() == 18 {
                    shci_ble_init(self.config);
                    self.rx_buffer[0] = 0x04; // replace event code with one that is supported by HCI
                }

                if let Some(cc) = self.mbox.pop_last_cc_evt() {


                    continue;
                }

                let payload_len = self.rx_buffer[2];
                return &self.rx_buffer[..3 + payload_len as usize];
            }
        }
    }
}
