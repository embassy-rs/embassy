use core::slice;

use embassy_stm32::ipcc::{IpccRxChannel, IpccTxChannel};
use embedded_io::Write;

#[cfg(feature = "wb-ble")]
use crate::shci::ShciBleInitCmdParam;
use crate::shci::{
    SchiCommandStatus, SchiFromPacket, SchiSysEventReady, ShciFusGetStateErrorCode, ShciFusState, ShciOpcode,
};
use crate::sub::mm;
use crate::wb::cmd::{CmdSerialStub, VolatileWriter};
use crate::wb::consts::TlPacketType;
use crate::wb::evt::EvtBox;
use crate::wb::tables::{FusDeviceInfoTable, SysTable, WirelessFwInfoTable};
use crate::wb::unsafe_linked_list::LinkedListNode;
use crate::wb::{SYS_CMD_BUF, SYSTEM_EVT_QUEUE, TL_DEVICE_INFO_TABLE, TL_SYS_TABLE};

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

    /// True when CPU2 is running the FUS (rather than the wireless firmware).
    ///
    /// The FUS rewrites the device info table handed over via the reference table with
    /// its own layout, marked by [`FUS_DEVICE_INFO_TABLE_VALIDITY_KEYWORD`].
    pub fn fus_running(&self) -> bool {
        // The FUS places its table at TL_DEVICE_INFO_TABLE, which is only
        // 4-byte aligned, so the whole struct must be read unaligned.
        let table = unsafe { (TL_DEVICE_INFO_TABLE.as_ptr() as *const FusDeviceInfoTable).read_unaligned() };
        table.is_valid()
    }

    /// Returns the device info table as rewritten by the running FUS.
    pub fn fus_info(&self) -> Option<FusDeviceInfoTable> {
        let table = unsafe { (TL_DEVICE_INFO_TABLE.as_ptr() as *const FusDeviceInfoTable).read_unaligned() };
        if table.is_valid() { Some(table) } else { None }
    }

    /// Returns CPU2 wireless firmware information (if present).
    ///
    /// The information is only valid while the wireless firmware is running on CPU2; when
    /// the FUS is running it rewrites the table (see [`Self::fus_info`]).
    pub fn wireless_fw_info(&self) -> Option<WirelessFwInfoTable> {
        if self.fus_running() {
            return None;
        }

        let info = unsafe { TL_DEVICE_INFO_TABLE.as_mut_ptr().read_volatile().wireless_fw_info_table };

        // Zero version indicates that CPU2 wasn't active and didn't fill the information table
        if info.version != 0 { Some(info) } else { None }
    }

    /// Returns the FUS version, if CPU2 is running (either FUS or the wireless stack).
    ///
    /// The FUS version lives in the FUS info table, whose location depends on which
    /// firmware runs on CPU2 (see ST's `SHCI_GetWirelessFwInfo` in `shci.c`, and AN5185,
    /// "FUS versioning and identification").
    pub fn fus_version(&self) -> Option<u32> {
        if self.fus_running() {
            Some(self.fus_info().unwrap().fus_version)
        } else {
            let fus_info = unsafe { TL_DEVICE_INFO_TABLE.as_ptr().read_volatile().fus_info_table };
            if fus_info.version != 0 {
                Some(fus_info.version)
            } else {
                None
            }
        }
    }

    pub async fn write(&mut self, opcode: ShciOpcode, payload: &[u8]) {
        self.ipcc_system_cmd_rsp_channel
            .send(|| unsafe {
                VolatileWriter::with_stub(
                    SYS_CMD_BUF.as_mut_ptr(),
                    CmdSerialStub {
                        ty: TlPacketType::SysCmd as u8,
                        cmd_code: opcode as u16,
                        payload_len: payload.len().try_into().unwrap(),
                    },
                )
                .write_all(payload)
                .unwrap();
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

    #[cfg(feature = "wb-mac")]
    pub async fn shci_c2_mac_802_15_4_init(&mut self) -> Result<SchiCommandStatus, ()> {
        self.write_and_get_response(ShciOpcode::Mac802_15_4Init, &[]).await
    }

    #[cfg(feature = "wb-thread")]
    pub async fn shci_c2_thread_init(&mut self) -> Result<SchiCommandStatus, ()> {
        self.write_and_get_response(ShciOpcode::ThreadInit, &[]).await
    }

    /// Send a request to CPU2 to initialise the BLE stack.
    ///
    /// This must be called before any BLE commands are sent via the BLE channel (according to
    /// AN5289, Figures 65 and 66). It should only be called after CPU2 sends a system event, via
    /// `HW_IPCC_SYS_EvtNot`, aka `IoBusCallBackUserEvt` (as detailed in Figure 65), aka
    /// [crate::sub::ble::hci::host::uart::UartHci::read].
    #[cfg(feature = "wb-ble")]
    pub async fn shci_c2_ble_init(&mut self, param: ShciBleInitCmdParam) -> Result<SchiCommandStatus, ()> {
        self.write_and_get_response(ShciOpcode::BleInit, param.payload()).await
    }

    /// `SHCI_C2_FUS_GetState`, returning the raw FUS state value and last error code.
    ///
    /// Unlike [`Self::shci_c2_fus_getstate`], this also reports the "ongoing" and
    /// "not running" states, which are needed to track an upgrade in progress.
    pub async fn shci_c2_fus_get_state(&mut self) -> Result<ShciFusState, ()> {
        self.write_and_get_response(ShciOpcode::FusGetState, &[]).await
    }

    pub async fn shci_c2_fus_getstate(&mut self) -> Result<ShciFusGetStateErrorCode, ()> {
        self.write_and_get_response(ShciOpcode::FusGetState, &[]).await
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

    /// Send a request to CPU2 to delete the wireless stack.
    ///
    /// Required before [`Self::shci_c2_fus_fwupgrade`] when the installed stack has a
    /// different type than the one being installed: the FUS refuses the upgrade
    /// while another wireless stack is present (see AN5185). The device must be
    /// reset after the delete completes.
    pub async fn shci_c2_fus_fwdelete(&mut self) -> Result<SchiCommandStatus, ()> {
        self.write_and_get_response(ShciOpcode::FusFirmwareDelete, &[]).await
    }

    pub async fn read_ready(&mut self) -> Result<SchiSysEventReady, ()> {
        // ST's `SHCI_C2_Ready_Evt_t` is `{ sub_evt_code: u16 (0x9200), ready_code: u8 }`,
        // so the ready code sits at payload[2], after the sub-event code.
        match self.read().await.payload() {
            [0x00, 0x92, code, ..] => (*code).try_into(),
            _ => Err(()),
        }
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
