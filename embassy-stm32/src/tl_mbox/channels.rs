//! CPU1                                             CPU2
//!  |             (SYSTEM)                            |
//!  |----HW_IPCC_SYSTEM_CMD_RSP_CHANNEL-------------->|
//!  |                                                 |
//!  |<---HW_IPCC_SYSTEM_EVENT_CHANNEL-----------------|
//!  |                                                 |
//!  |            (ZIGBEE)                             |
//!  |----HW_IPCC_ZIGBEE_CMD_APPLI_CHANNEL------------>|
//!  |                                                 |
//!  |----HW_IPCC_ZIGBEE_CMD_CLI_CHANNEL-------------->|
//!  |                                                 |
//!  |<---HW_IPCC_ZIGBEE_APPLI_NOTIF_ACK_CHANNEL-------|
//!  |                                                 |
//!  |<---HW_IPCC_ZIGBEE_CLI_NOTIF_ACK_CHANNEL---------|
//!  |                                                 |
//!  |             (THREAD)                            |
//!  |----HW_IPCC_THREAD_OT_CMD_RSP_CHANNEL----------->|
//!  |                                                 |
//!  |----HW_IPCC_THREAD_CLI_CMD_CHANNEL-------------->|
//!  |                                                 |
//!  |<---HW_IPCC_THREAD_NOTIFICATION_ACK_CHANNEL------|
//!  |                                                 |
//!  |<---HW_IPCC_THREAD_CLI_NOTIFICATION_ACK_CHANNEL--|
//!  |                                                 |
//!  |             (BLE)                               |
//!  |----HW_IPCC_BLE_CMD_CHANNEL--------------------->|
//!  |                                                 |
//!  |----HW_IPCC_HCI_ACL_DATA_CHANNEL---------------->|
//!  |                                                 |
//!  |<---HW_IPCC_BLE_EVENT_CHANNEL--------------------|
//!  |                                                 |
//!  |             (BLE LLD)                           |
//!  |----HW_IPCC_BLE_LLD_CMD_CHANNEL----------------->|
//!  |                                                 |
//!  |<---HW_IPCC_BLE_LLD_RSP_CHANNEL------------------|
//!  |                                                 |
//!  |<---HW_IPCC_BLE_LLD_M0_CMD_CHANNEL---------------|
//!  |                                                 |
//!  |             (MAC)                               |
//!  |----HW_IPCC_MAC_802_15_4_CMD_RSP_CHANNEL-------->|
//!  |                                                 |
//!  |<---HW_IPCC_MAC_802_15_4_NOTIFICATION_ACK_CHANNEL|
//!  |                                                 |
//!  |             (BUFFER)                            |
//!  |----HW_IPCC_MM_RELEASE_BUFFER_CHANNE------------>|
//!  |                                                 |
//!  |             (TRACE)                             |
//!  |<----HW_IPCC_TRACES_CHANNEL----------------------|
//!  |                                                 |
//!

use crate::ipcc::IpccChannel;

pub enum Cpu1Channel {
    BleCmd,
    SystemCmdRsp,
    #[cfg(feature = "thread")]
    ThreadOtCmdRsp,
    #[cfg(feature = "zigbee")]
    ZigbeeCmdAppli,
    MmReleaseBuffer,
    #[cfg(feature = "mac-802_15_4")]
    Mac802_15_4cmdRsp,
    #[cfg(feature = "thread")]
    ThreadCliCmd,
    #[cfg(feature = "lld-tests")]
    LldTestsCliCmd,
    #[cfg(feature = "ble-lld")]
    BleLldCmd,
    HciAclData,
}

impl From<Cpu1Channel> for IpccChannel {
    fn from(value: Cpu1Channel) -> Self {
        match value {
            Cpu1Channel::BleCmd => IpccChannel::Channel1,
            Cpu1Channel::SystemCmdRsp => IpccChannel::Channel2,
            #[cfg(feature = "thread")]
            Cpu1Channel::ThreadOtCmdRsp => IpccChannel::Channel3,
            #[cfg(feature = "zigbee")]
            Cpu1Channel::ZigbeeCmdAppli => IpccChannel::Channel3,
            #[cfg(feature = "mac-802_15_4")]
            Cpu1Channel::Mac802_15_4cmdRsp => IpccChannel::Channel3,
            Cpu1Channel::MmReleaseBuffer => IpccChannel::Channel4,
            #[cfg(feature = "thread")]
            Cpu1Channel::ThreadCliCmd => IpccChannel::Channel5,
            #[cfg(feature = "lld-tests")]
            Cpu1Channel::LldTestsCliCmd => IpccChannel::Channel5,
            #[cfg(feature = "ble-lld")]
            Cpu1Channel::BleLldCmd => IpccChannel::Channel5,
            Cpu1Channel::HciAclData => IpccChannel::Channel6,
        }
    }
}

pub enum Cpu2Channel {
    BleEvent,
    SystemEvent,
    #[cfg(feature = "thread")]
    ThreadNotifAck,
    #[cfg(feature = "zigbee")]
    ZigbeeAppliNotifAck,
    #[cfg(feature = "mac-802_15_4")]
    Mac802_15_4NotifAck,
    #[cfg(feature = "lld-tests")]
    LldTestsM0Cmd,
    #[cfg(feature = "ble-lld")]
    BleLldM0Cmd,
    #[cfg(feature = "traces")]
    Traces,
    #[cfg(feature = "thread")]
    ThreadCliNotifAck,
    #[cfg(feature = "lld-tests")]
    LldTestsCliRsp,
    #[cfg(feature = "ble-lld")]
    BleLldCliRsp,
    #[cfg(feature = "ble-lld")]
    BleLldRsp,
    #[cfg(feature = "zigbee")]
    ZigbeeM0Request,
}

impl From<Cpu2Channel> for IpccChannel {
    fn from(value: Cpu2Channel) -> Self {
        match value {
            Cpu2Channel::BleEvent => IpccChannel::Channel1,
            Cpu2Channel::SystemEvent => IpccChannel::Channel2,
            #[cfg(feature = "thread")]
            Cpu2Channel::ThreadNotifAck => IpccChannel::Channel3,
            #[cfg(feature = "zigbee")]
            Cpu2Channel::ZigbeeAppliNotifAck => IpccChannel::Channel3,
            #[cfg(feature = "mac-802_15_4")]
            Cpu2Channel::Mac802_15_4NotifAck => IpccChannel::Channel3,
            #[cfg(feature = "lld-tests")]
            Cpu2Channel::LldTestsM0Cmd => IpccChannel::Channel3,
            #[cfg(feature = "ble-lld")]
            Cpu2Channel::BleLldM0Cmd => IpccChannel::Channel3,
            #[cfg(feature = "traces")]
            Cpu2Channel::Traces => IpccChannel::Channel4,
            #[cfg(feature = "thread")]
            Cpu2Channel::ThreadCliNotifAck => IpccChannel::Channel5,
            #[cfg(feature = "lld-tests")]
            Cpu2Channel::LldTestsCliRsp => IpccChannel::Channel5,
            #[cfg(feature = "ble-lld")]
            Cpu2Channel::BleLldCliRsp => IpccChannel::Channel5,
            #[cfg(feature = "ble-lld")]
            Cpu2Channel::BleLldRsp => IpccChannel::Channel5,
            #[cfg(feature = "zigbee")]
            Cpu2Channel::ZigbeeM0Request => IpccChannel::Channel5,
        }
    }
}
