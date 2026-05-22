use core::{mem, ptr, slice};

use embassy_stm32::ipcc::{Ipcc, IpccRxChannel, IpccTxChannel};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_io::ErrorKind;

use crate::evt::MemoryManager;
use crate::net;
use crate::net::iface::mcps::{
    ConfirmPacket as McpsConfirmPacket, IndicationPacket as McpsIndicationPacket, Packet as McpsPacket,
    PacketKind as McpsPacketKind,
};
use crate::net::iface::mlme::{
    ConfirmPacket as MlmeConfirmPacket, IndicationPacket as MlmeIndicationPacket, Packet as MlmePacket,
    PacketKind as MlmePacketKind, RequestPacketKind as MlmeRequestPacketKind,
    ResponsePacketKind as MlmeResponsePacketKind,
};
use crate::net::iface::{ControllerToHostPacket, FromHciBytes, PacketKind, WriteHci};
use crate::net::indications::{
    AssociateIndication, BeaconNotifyIndication, CommStatusIndication, DataIndication, DisassociateIndication,
    DpsIndication, GtsIndication, OrphanIndication, PollIndication, SyncLossIndication,
};
use crate::net::responses::{
    AssociateConfirm, CalibrateConfirm, DataConfirm, DisassociateConfirm, DpsConfirm, GetConfirm, GtsConfirm,
    PollConfirm, PurgeConfirm, ResetConfirm, RxEnableConfirm, ScanConfirm, SetConfirm, SoundingConfirm, StartConfirm,
};
use crate::util::Flag;
use crate::wb::channels::cpu1::IPCC_MAC_802_15_4_CMD_RSP_CHANNEL;
use crate::wb::cmd::CmdPacket;
use crate::wb::consts::TlPacketType;
use crate::wb::evt::{self, EvtBox, EvtPacket};
use crate::wb::tables::{
    MAC_802_15_4_CMD_BUFFER, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER, Mac802_15_4Table, TL_MAC_802_15_4_TABLE,
    TL_TRACES_TABLE, TRACES_EVT_QUEUE, TracesTable,
};
use crate::wb::unsafe_linked_list::LinkedListNode;

static MAC_EVT_OUT: Flag = Flag::new(false);

pub struct Mac<'a> {
    ipcc_mac_802_15_4_cmd_rsp_channel: IpccTxChannel<'a>,
    ipcc_mac_802_15_4_notification_ack_channel: IpccRxChannel<'a>,
}

impl<'a> Mac<'a> {
    pub(crate) fn new(
        ipcc_mac_802_15_4_cmd_rsp_channel: IpccTxChannel<'a>,
        ipcc_mac_802_15_4_notification_ack_channel: IpccRxChannel<'a>,
    ) -> Self {
        unsafe {
            LinkedListNode::init_head(TRACES_EVT_QUEUE.as_mut_ptr() as *mut _);

            TL_TRACES_TABLE.as_mut_ptr().write_volatile(TracesTable {
                traces_queue: TRACES_EVT_QUEUE.as_ptr() as *const _,
            });

            TL_MAC_802_15_4_TABLE.as_mut_ptr().write_volatile(Mac802_15_4Table {
                p_cmdrsp_buffer: MAC_802_15_4_CMD_BUFFER.as_mut_ptr().cast(),
                p_notack_buffer: MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr().cast(),
                evt_queue: core::ptr::null_mut(),
            });
        };

        Self {
            ipcc_mac_802_15_4_cmd_rsp_channel,
            ipcc_mac_802_15_4_notification_ack_channel,
        }
    }
}

pub struct ControllerAdapter<'d> {
    ipcc_mac_802_15_4_cmd_rsp_channel: Mutex<NoopRawMutex, IpccTxChannel<'d>>,
    ipcc_mac_802_15_4_notification_ack_channel: Mutex<NoopRawMutex, IpccRxChannel<'d>>,
}

impl<'d> ControllerAdapter<'d> {
    pub const fn new(mac: Mac<'d>) -> Self {
        Self {
            ipcc_mac_802_15_4_cmd_rsp_channel: Mutex::new(mac.ipcc_mac_802_15_4_cmd_rsp_channel),
            ipcc_mac_802_15_4_notification_ack_channel: Mutex::new(mac.ipcc_mac_802_15_4_notification_ack_channel),
        }
    }
}

impl<'d> embedded_io::ErrorType for ControllerAdapter<'d> {
    type Error = embedded_io::ErrorKind;
}

pub struct ParsedIndication<'d> {
    pkt: ControllerToHostPacket<'d>,
}

impl<'d> Drop for ParsedIndication<'d> {
    fn drop(&mut self) {
        unsafe { Mac::drop_event_packet(MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _) };
    }
}

impl<'d, 'a> net::iface::ControllerToHostPacketBox<'a> for ParsedIndication<'d> {
    fn packet<'b>(&'b self) -> ControllerToHostPacket<'b>
    where
        'a: 'b,
    {
        self.pkt
    }
}

impl<'d> net::iface::Controller for ControllerAdapter<'d> {
    type Packet<'a> = ParsedIndication<'a>;

    async fn read<'a>(&self, _buf: &'a mut [u8]) -> Result<Self::Packet<'a>, Self::Error> {
        MAC_EVT_OUT.wait_for_low().await;

        // Return a new event box
        let evt: EvtBox<Mac<'d>> = self
            .ipcc_mac_802_15_4_notification_ack_channel
            .lock()
            .await
            .receive(|| unsafe { Some(EvtBox::new(MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _)) })
            .await;

        let payload = unsafe { slice::from_raw_parts(evt.payload() as *const _ as *const u8, evt.payload().len()) };

        mem::forget(evt);

        let (opcode, payload) = payload.split_at(2);
        let pkt = match u16::from_le_bytes(opcode.try_into().unwrap()) {
            0 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Associate(
                AssociateConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            1 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Disassociate(
                DisassociateConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            2 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Get(
                GetConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            3 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Gts(
                GtsConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            4 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Reset(
                ResetConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            5 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::RxEnable(
                RxEnableConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            6 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Scan(
                ScanConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            7 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Set(
                SetConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            8 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Start(
                StartConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            9 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Poll(
                PollConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            10 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Dps(
                DpsConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            11 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Sounding(
                SoundingConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            12 => ControllerToHostPacket::Mlme(MlmePacket::Confirm(MlmeConfirmPacket::Calibrate(
                CalibrateConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            13 => ControllerToHostPacket::Mcps(McpsPacket::Confirm(McpsConfirmPacket::Data(
                DataConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            14 => ControllerToHostPacket::Mcps(McpsPacket::Confirm(McpsConfirmPacket::Purge(
                PurgeConfirm::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            15 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::Associate(
                AssociateIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            16 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::Disassociate(
                DisassociateIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            17 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::BeaconNotify(
                BeaconNotifyIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            18 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::CommStatus(
                CommStatusIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            19 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::Gts(
                GtsIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            20 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::Orphan(
                OrphanIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            21 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::SyncLoss(
                SyncLossIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            22 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::Dps(
                DpsIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            23 => ControllerToHostPacket::Mcps(McpsPacket::Indication(McpsIndicationPacket::Data(
                DataIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            24 => ControllerToHostPacket::Mlme(MlmePacket::Indication(MlmeIndicationPacket::Poll(
                PollIndication::from_hci_bytes(payload).map_err(|e| e.into())?,
            ))),
            _ => return Err(ErrorKind::InvalidData),
        };

        Ok(ParsedIndication { pkt })
    }

    async fn write(&self, packet: &impl net::iface::HostToControllerPacket) -> Result<(), Self::Error> {
        const ST_VENDOR_OGF: u16 = 0x3F;
        const MAC_802_15_4_CMD_OPCODE_OFFSET: u16 = 0x280;

        const fn opcode(ocf: u16) -> isize {
            ((ST_VENDOR_OGF << 9) | (MAC_802_15_4_CMD_OPCODE_OFFSET + ocf)) as isize
        }

        struct WithIndicator<'d, T: net::iface::HostToControllerPacket> {
            pkt: &'d T,
        }

        impl<'d, T: net::iface::HostToControllerPacket> WriteHci for WithIndicator<'d, T> {
            fn size(&self) -> usize {
                4 + size_of_val(self.pkt)
            }

            fn write_hci<W: embedded_io::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                let opcode = match self.pkt.kind() {
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Associate)) => opcode(0x00),
                    PacketKind::Mlme(MlmePacketKind::Response(MlmeResponsePacketKind::Associate)) => opcode(0x01),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Dissassociate)) => opcode(0x02),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Get)) => opcode(0x03),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Gts)) => opcode(0x04),
                    PacketKind::Mlme(MlmePacketKind::Response(MlmeResponsePacketKind::Orphan)) => opcode(0x05),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Reset)) => opcode(0x06),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::RxEnable)) => opcode(0x07),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Scan)) => opcode(0x08),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Set)) => opcode(0x09),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Start)) => opcode(0x0A),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Sync)) => opcode(0x0B),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Poll)) => opcode(0x0C),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Dps)) => opcode(0x0D),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Sounding)) => opcode(0x0E),
                    PacketKind::Mlme(MlmePacketKind::Request(MlmeRequestPacketKind::Calibrate)) => opcode(0x0F),
                    PacketKind::Mcps(McpsPacketKind::Data) => opcode(0x10),
                    PacketKind::Mcps(McpsPacketKind::Purge) => opcode(0x11),
                }
                .to_le_bytes();

                writer.write_all(&[
                    TlPacketType::MacCmd as u8,
                    opcode[0],
                    opcode[1],
                    self.pkt.size().try_into().unwrap(),
                ])?;
                self.pkt.write_hci(writer)
            }
        }

        let mut ipcc_mac_802_15_4_cmd_rsp_channel = self.ipcc_mac_802_15_4_cmd_rsp_channel.lock().await;

        ipcc_mac_802_15_4_cmd_rsp_channel
            .send(|| unsafe {
                WithIndicator { pkt: packet }.write_hci(CmdPacket::writer(MAC_802_15_4_CMD_BUFFER.as_mut_ptr()))
            })
            .await?;

        ipcc_mac_802_15_4_cmd_rsp_channel.flush().await;

        let response = unsafe {
            let p_event_packet = MAC_802_15_4_CMD_BUFFER.as_ptr() as *const EvtPacket;
            let p_mac_rsp_evt = &((*p_event_packet).evt_serial.evt.payload) as *const u8;

            ptr::read_volatile(p_mac_rsp_evt)
        };

        if response == 0x00 {
            Ok(())
        } else {
            Err(ErrorKind::InvalidInput)
        }
    }
}

impl<'a> evt::MemoryManager for Mac<'a> {
    unsafe fn new_event_packet(evt: *mut EvtPacket) {
        if ptr::eq(evt, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _) {
            MAC_EVT_OUT.set_high();
        }
    }

    unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        if ptr::eq(evt, MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _) {
            trace!("mac drop event");

            // Write the ack
            CmdPacket::write_into(
                MAC_802_15_4_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut _,
                TlPacketType::OtAck,
                0,
                &[],
            );

            // Clear the rx flag
            Ipcc::clear(IPCC_MAC_802_15_4_CMD_RSP_CHANNEL as u8);
            MAC_EVT_OUT.set_low();
        }
    }
}
