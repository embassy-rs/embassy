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

pub mod cpu1 {
    use embassy_stm32::ipcc::IpccChannel;

    pub const IPCC_BLE_CMD_CHANNEL: IpccChannel = IpccChannel::Channel1;
    pub const IPCC_SYSTEM_CMD_RSP_CHANNEL: IpccChannel = IpccChannel::Channel2;
    pub const IPCC_THREAD_OT_CMD_RSP_CHANNEL: IpccChannel = IpccChannel::Channel3;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_ZIGBEE_CMD_APPLI_CHANNEL: IpccChannel = IpccChannel::Channel3;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_MAC_802_15_4_CMD_RSP_CHANNEL: IpccChannel = IpccChannel::Channel3;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_MM_RELEASE_BUFFER_CHANNEL: IpccChannel = IpccChannel::Channel4;
    pub const IPCC_THREAD_CLI_CMD_CHANNEL: IpccChannel = IpccChannel::Channel5;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_LLDTESTS_CLI_CMD_CHANNEL: IpccChannel = IpccChannel::Channel5;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_BLE_LLD_CMD_CHANNEL: IpccChannel = IpccChannel::Channel5;
    pub const IPCC_HCI_ACL_DATA_CHANNEL: IpccChannel = IpccChannel::Channel6;
}

pub mod cpu2 {
    use embassy_stm32::ipcc::IpccChannel;

    pub const IPCC_BLE_EVENT_CHANNEL: IpccChannel = IpccChannel::Channel1;
    pub const IPCC_SYSTEM_EVENT_CHANNEL: IpccChannel = IpccChannel::Channel2;
    pub const IPCC_THREAD_NOTIFICATION_ACK_CHANNEL: IpccChannel = IpccChannel::Channel3;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_ZIGBEE_APPLI_NOTIF_ACK_CHANNEL: IpccChannel = IpccChannel::Channel3;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_MAC_802_15_4_NOTIFICATION_ACK_CHANNEL: IpccChannel = IpccChannel::Channel3;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_LDDTESTS_M0_CMD_CHANNEL: IpccChannel = IpccChannel::Channel3;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_BLE_LLDÇM0_CMD_CHANNEL: IpccChannel = IpccChannel::Channel3;
    pub const IPCC_TRACES_CHANNEL: IpccChannel = IpccChannel::Channel4;
    pub const IPCC_THREAD_CLI_NOTIFICATION_ACK_CHANNEL: IpccChannel = IpccChannel::Channel5;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_LLDTESTS_CLI_RSP_CHANNEL: IpccChannel = IpccChannel::Channel5;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_BLE_LLD_CLI_RSP_CHANNEL: IpccChannel = IpccChannel::Channel5;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_BLE_LLD_RSP_CHANNEL: IpccChannel = IpccChannel::Channel5;
    #[allow(dead_code)] // Not used currently but reserved
    pub const IPCC_ZIGBEE_M0_REQUEST_CHANNEL: IpccChannel = IpccChannel::Channel5;
}
