use ch::driver::LinkState;
use defmt::Debug2Format;
use embassy_net_driver_channel as ch;
use heapless::String;

use crate::ioctl::Shared;
use crate::proto::{self, CtrlMsg};

#[derive(Debug)]
pub struct Error {
    pub status: u32,
}

pub struct Control<'a> {
    state_ch: ch::StateRunner<'a>,
    shared: &'a Shared,
}

#[allow(unused)]
enum WifiMode {
    None = 0,
    Sta = 1,
    Ap = 2,
    ApSta = 3,
}

impl<'a> Control<'a> {
    pub(crate) fn new(state_ch: ch::StateRunner<'a>, shared: &'a Shared) -> Self {
        Self { state_ch, shared }
    }

    pub async fn init(&mut self) {
        debug!("wait for init event...");
        self.shared.init_wait().await;

        debug!("set wifi mode");
        self.set_wifi_mode(WifiMode::Sta as _).await;

        let mac_addr = self.get_mac_addr().await;
        debug!("mac addr: {:02x}", mac_addr);
        self.state_ch.set_ethernet_address(mac_addr);
    }

    pub async fn join(&mut self, ssid: &str, password: &str) {
        let req = proto::CtrlMsg {
            msg_id: proto::CtrlMsgId::ReqConnectAp as _,
            msg_type: proto::CtrlMsgType::Req as _,
            payload: Some(proto::CtrlMsgPayload::ReqConnectAp(proto::CtrlMsgReqConnectAp {
                ssid: String::from(ssid),
                pwd: String::from(password),
                bssid: String::new(),
                listen_interval: 3,
                is_wpa3_supported: false,
            })),
        };
        let resp = self.ioctl(req).await;
        let proto::CtrlMsgPayload::RespConnectAp(resp) = resp.payload.unwrap() else { panic!("unexpected resp") };
        debug!("======= {:?}", Debug2Format(&resp));
        assert_eq!(resp.resp, 0);
        self.state_ch.set_link_state(LinkState::Up);
    }

    async fn get_mac_addr(&mut self) -> [u8; 6] {
        let req = proto::CtrlMsg {
            msg_id: proto::CtrlMsgId::ReqGetMacAddress as _,
            msg_type: proto::CtrlMsgType::Req as _,
            payload: Some(proto::CtrlMsgPayload::ReqGetMacAddress(
                proto::CtrlMsgReqGetMacAddress {
                    mode: WifiMode::Sta as _,
                },
            )),
        };
        let resp = self.ioctl(req).await;
        let proto::CtrlMsgPayload::RespGetMacAddress(resp) = resp.payload.unwrap() else { panic!("unexpected resp") };
        assert_eq!(resp.resp, 0);

        // WHY IS THIS A STRING? WHYYYY
        fn nibble_from_hex(b: u8) -> u8 {
            match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b + 0xa - b'a',
                b'A'..=b'F' => b + 0xa - b'A',
                _ => panic!("invalid hex digit {}", b),
            }
        }

        let mac = resp.mac.as_bytes();
        let mut res = [0; 6];
        assert_eq!(mac.len(), 17);
        for (i, b) in res.iter_mut().enumerate() {
            *b = (nibble_from_hex(mac[i * 3]) << 4) | nibble_from_hex(mac[i * 3 + 1])
        }
        res
    }

    async fn set_wifi_mode(&mut self, mode: u32) {
        let req = proto::CtrlMsg {
            msg_id: proto::CtrlMsgId::ReqSetWifiMode as _,
            msg_type: proto::CtrlMsgType::Req as _,
            payload: Some(proto::CtrlMsgPayload::ReqSetWifiMode(proto::CtrlMsgReqSetMode { mode })),
        };
        let resp = self.ioctl(req).await;
        let proto::CtrlMsgPayload::RespSetWifiMode(resp) = resp.payload.unwrap() else { panic!("unexpected resp") };
        assert_eq!(resp.resp, 0);
    }

    async fn ioctl(&mut self, req: CtrlMsg) -> CtrlMsg {
        debug!("ioctl req: {:?}", &req);

        let mut buf = [0u8; 128];

        let req_len = noproto::write(&req, &mut buf).unwrap();

        struct CancelOnDrop<'a>(&'a Shared);

        impl CancelOnDrop<'_> {
            fn defuse(self) {
                core::mem::forget(self);
            }
        }

        impl Drop for CancelOnDrop<'_> {
            fn drop(&mut self) {
                self.0.ioctl_cancel();
            }
        }

        let ioctl = CancelOnDrop(self.shared);

        let resp_len = ioctl.0.ioctl(&mut buf, req_len).await;

        ioctl.defuse();

        let res = noproto::read(&buf[..resp_len]).unwrap();
        debug!("ioctl resp: {:?}", &res);

        res
    }
}
