use core::slice;

use embassy_stm32::ipcc::{IpccRxChannel, IpccTxChannel};

use crate::cmd::CmdPacket;
use crate::consts::TlPacketType;
use crate::evt::EvtBox;
#[cfg(feature = "wb55_ble")]
use crate::shci::ShciBleInitCmdParam;
use crate::shci::{SchiCommandStatus, SchiFromPacket, SchiSysEventReady, ShciFusGetStateErrorCode, ShciOpcode};
use crate::sub::mm;
use crate::tables::{SysTable, WirelessFwInfoTable};
use crate::unsafe_linked_list::LinkedListNode;
use crate::wb55::{SYS_CMD_BUF, SYSTEM_EVT_QUEUE, TL_DEVICE_INFO_TABLE, TL_SYS_TABLE};

const fn slice8_ref(x: &[u32]) -> &[u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts(x.as_ptr() as *const u8, len) }
}

/// A guard that, once constructed, allows for sys commands to be sent to CPU2.
pub struct Sys<'a> {
    ipcc_system_cmd_rsp_channel: IpccTxChannel<'a>,
    ipcc_system_event_channel: IpccRxChannel<'a>,
}

impl<'a> Sys<'a> {
    /// TL_Sys_Init
    pub(crate) fn new(
        ipcc_system_cmd_rsp_channel: IpccTxChannel<'a>,
        ipcc_system_event_channel: IpccRxChannel<'a>,
    ) -> Self {
        unsafe {
            LinkedListNode::init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            TL_SYS_TABLE.as_mut_ptr().write_volatile(SysTable {
                pcmd_buffer: SYS_CMD_BUF.as_mut_ptr(),
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            });
        }

        Self {
            ipcc_system_cmd_rsp_channel,
            ipcc_system_event_channel,
        }
    }

    /// Returns CPU2 wireless firmware information (if present).
    pub fn wireless_fw_info(&self) -> Option<WirelessFwInfoTable> {
        let info = unsafe { TL_DEVICE_INFO_TABLE.as_mut_ptr().read_volatile().wireless_fw_info_table };

        // Zero version indicates that CPU2 wasn't active and didn't fill the information table
        if info.version != 0 { Some(info) } else { None }
    }

    pub async fn write(&mut self, opcode: ShciOpcode, payload: &[u8]) {
        self.ipcc_system_cmd_rsp_channel
            .send(|| unsafe {
                CmdPacket::write_into(SYS_CMD_BUF.as_mut_ptr(), TlPacketType::SysCmd, opcode as u16, payload);
            })
            .await;
    }

    /// `HW_IPCC_SYS_CmdEvtNot`
    pub async fn write_and_get_response<T: SchiFromPacket>(
        &mut self,
        opcode: ShciOpcode,
        payload: &[u8],
    ) -> Result<T, ()> {
        self.write(opcode, payload).await;
        self.ipcc_system_cmd_rsp_channel.flush().await;

        unsafe { T::from_packet(SYS_CMD_BUF.as_ptr()) }
    }

    #[cfg(feature = "wb55_mac")]
    pub async fn shci_c2_mac_802_15_4_init(&mut self) -> Result<SchiCommandStatus, ()> {
        self.write_and_get_response(ShciOpcode::Mac802_15_4Init, &[]).await
    }

    /// Send a request to CPU2 to initialise the BLE stack.
    ///
    /// This must be called before any BLE commands are sent via the BLE channel (according to
    /// AN5289, Figures 65 and 66). It should only be called after CPU2 sends a system event, via
    /// `HW_IPCC_SYS_EvtNot`, aka `IoBusCallBackUserEvt` (as detailed in Figure 65), aka
    /// [crate::sub::ble::hci::host::uart::UartHci::read].
    #[cfg(feature = "wb55_ble")]
    pub async fn shci_c2_ble_init(&mut self, param: ShciBleInitCmdParam) -> Result<SchiCommandStatus, ()> {
        self.write_and_get_response(ShciOpcode::BleInit, param.payload()).await
    }

    pub async fn shci_c2_fus_getstate(&mut self) -> Result<ShciFusGetStateErrorCode, ()> {
        self.write_and_get_response(ShciOpcode::FusStartWirelessStack, &[])
            .await
    }

    /// Send a request to CPU2 to start the wireless stack
    pub async fn shci_c2_fus_startws(&mut self) -> Result<SchiCommandStatus, ()> {
        self.write_and_get_response(ShciOpcode::FusStartWirelessStack, &[])
            .await
    }

    /// Send a request to CPU2 to upgrade the firmware
    pub async fn shci_c2_fus_fwupgrade(&mut self, fw_src_add: u32, fw_dst_add: u32) -> Result<SchiCommandStatus, ()> {
        let buf = [fw_src_add, fw_dst_add];
        let len = if fw_dst_add != 0 {
            2
        } else if fw_src_add != 0 {
            1
        } else {
            0
        };

        self.write_and_get_response(ShciOpcode::FusFirmwareUpgrade, slice8_ref(&buf[..len]))
            .await
    }

    pub async fn read_ready(&mut self) -> Result<SchiSysEventReady, ()> {
        self.read().await.payload()[0].try_into()
    }

    /// `HW_IPCC_SYS_EvtNot`
    ///
    /// This method takes the place of the `HW_IPCC_SYS_EvtNot`/`SysUserEvtRx`/`APPE_SysUserEvtRx`,
    /// as the embassy implementation avoids the need to call C public bindings, and instead
    /// handles the event channels directly.
    pub async fn read(&mut self) -> EvtBox<mm::MemoryManager<'_>> {
        self.ipcc_system_event_channel
            .receive(|| unsafe {
                if let Some(node_ptr) =
                    critical_section::with(|cs| LinkedListNode::remove_head(cs, SYSTEM_EVT_QUEUE.as_mut_ptr()))
                {
                    Some(EvtBox::new(node_ptr.cast()))
                } else {
                    None
                }
            })
            .await
    }
}
