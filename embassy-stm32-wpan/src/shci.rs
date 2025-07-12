use core::{mem, slice};

use crate::consts::{TL_CS_EVT_SIZE, TL_EVT_HEADER_SIZE, TL_PACKET_HEADER_SIZE};

const SHCI_OGF: u16 = 0x3F;

const fn opcode(ogf: u16, ocf: u16) -> isize {
    ((ogf << 10) + ocf) as isize
}

#[allow(dead_code)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SchiCommandStatus {
    ShciSuccess = 0x00,
    ShciUnknownCmd = 0x01,
    ShciMemoryCapacityExceededErrCode = 0x07,
    ShciErrUnsupportedFeature = 0x11,
    ShciErrInvalidHciCmdParams = 0x12,
    ShciErrInvalidParams = 0x42,   /* only used for release < v1.13.0 */
    ShciErrInvalidParamsV2 = 0x92, /* available for release >= v1.13.0 */
    ShciFusCmdNotSupported = 0xFF,
}

impl TryFrom<u8> for SchiCommandStatus {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == SchiCommandStatus::ShciSuccess as u8 => Ok(SchiCommandStatus::ShciSuccess),
            x if x == SchiCommandStatus::ShciUnknownCmd as u8 => Ok(SchiCommandStatus::ShciUnknownCmd),
            x if x == SchiCommandStatus::ShciMemoryCapacityExceededErrCode as u8 => {
                Ok(SchiCommandStatus::ShciMemoryCapacityExceededErrCode)
            }
            x if x == SchiCommandStatus::ShciErrUnsupportedFeature as u8 => {
                Ok(SchiCommandStatus::ShciErrUnsupportedFeature)
            }
            x if x == SchiCommandStatus::ShciErrInvalidHciCmdParams as u8 => {
                Ok(SchiCommandStatus::ShciErrInvalidHciCmdParams)
            }
            x if x == SchiCommandStatus::ShciErrInvalidParams as u8 => Ok(SchiCommandStatus::ShciErrInvalidParams), /* only used for release < v1.13.0 */
            x if x == SchiCommandStatus::ShciErrInvalidParamsV2 as u8 => Ok(SchiCommandStatus::ShciErrInvalidParamsV2), /* available for release >= v1.13.0 */
            x if x == SchiCommandStatus::ShciFusCmdNotSupported as u8 => Ok(SchiCommandStatus::ShciFusCmdNotSupported),
            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ShciOpcode {
    // 0x50 reserved
    // 0x51 reserved
    FusGetState = opcode(SHCI_OGF, 0x52),
    // 0x53 reserved
    FusFirmwareUpgrade = opcode(SHCI_OGF, 0x54),
    FusFirmwareDelete = opcode(SHCI_OGF, 0x55),
    FusUpdateAuthKey = opcode(SHCI_OGF, 0x56),
    FusLockAuthKey = opcode(SHCI_OGF, 0x57),
    FusStoreUserKey = opcode(SHCI_OGF, 0x58),
    FusLoadUserKey = opcode(SHCI_OGF, 0x59),
    FusStartWirelessStack = opcode(SHCI_OGF, 0x5a),
    // 0x5b reserved
    // 0x5c reserved
    FusLockUserKey = opcode(SHCI_OGF, 0x5d),
    FusUnloadUserKey = opcode(SHCI_OGF, 0x5e),
    FusActivateAntirollback = opcode(SHCI_OGF, 0x5f),
    // 0x60 reserved
    // 0x61 reserved
    // 0x62 reserved
    // 0x63 reserved
    // 0x64 reserved
    // 0x65 reserved
    BleInit = opcode(SHCI_OGF, 0x66),
    ThreadInit = opcode(SHCI_OGF, 0x67),
    DebugInit = opcode(SHCI_OGF, 0x68),
    FlashEraseActivity = opcode(SHCI_OGF, 0x69),
    ConcurrentSetMode = opcode(SHCI_OGF, 0x6a),
    FlashStoreData = opcode(SHCI_OGF, 0x6b),
    FlashEraseData = opcode(SHCI_OGF, 0x6c),
    RadioAllowLowPower = opcode(SHCI_OGF, 0x6d),
    Mac802_15_4Init = opcode(SHCI_OGF, 0x6e),
    ReInit = opcode(SHCI_OGF, 0x6f),
    ZigbeeInit = opcode(SHCI_OGF, 0x70),
    LldTestsInit = opcode(SHCI_OGF, 0x71),
    ExtraConfig = opcode(SHCI_OGF, 0x72),
    SetFlashActivityControl = opcode(SHCI_OGF, 0x73),
    BleLldInit = opcode(SHCI_OGF, 0x74),
    Config = opcode(SHCI_OGF, 0x75),
    ConcurrentGetNextBleEvtTime = opcode(SHCI_OGF, 0x76),
    ConcurrentEnableNext802_15_4EvtNotification = opcode(SHCI_OGF, 0x77),
    Mac802_15_4DeInit = opcode(SHCI_OGF, 0x78),
}

pub const SHCI_C2_CONFIG_EVTMASK1_BIT0_ERROR_NOTIF_ENABLE: u8 = 1 << 0;
pub const SHCI_C2_CONFIG_EVTMASK1_BIT1_BLE_NVM_RAM_UPDATE_ENABLE: u8 = 1 << 1;
pub const SHCI_C2_CONFIG_EVTMASK1_BIT2_THREAD_NVM_RAM_UPDATE_ENABLE: u8 = 1 << 2;
pub const SHCI_C2_CONFIG_EVTMASK1_BIT3_NVM_START_WRITE_ENABLE: u8 = 1 << 3;
pub const SHCI_C2_CONFIG_EVTMASK1_BIT4_NVM_END_WRITE_ENABLE: u8 = 1 << 4;
pub const SHCI_C2_CONFIG_EVTMASK1_BIT5_NVM_START_ERASE_ENABLE: u8 = 1 << 5;
pub const SHCI_C2_CONFIG_EVTMASK1_BIT6_NVM_END_ERASE_ENABLE: u8 = 1 << 6;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ShciConfigParam {
    pub payload_cmd_size: u8,
    pub config: u8,
    pub event_mask: u8,
    pub spare: u8,
    pub ble_nvm_ram_address: u32,
    pub thread_nvm_ram_address: u32,
    pub revision_id: u16,
    pub device_id: u16,
}

impl ShciConfigParam {
    pub fn payload<'a>(&'a self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Self>()) }
    }
}

impl Default for ShciConfigParam {
    fn default() -> Self {
        Self {
            payload_cmd_size: (mem::size_of::<Self>() - 1) as u8,
            config: 0,
            event_mask: SHCI_C2_CONFIG_EVTMASK1_BIT0_ERROR_NOTIF_ENABLE
                + SHCI_C2_CONFIG_EVTMASK1_BIT1_BLE_NVM_RAM_UPDATE_ENABLE
                + SHCI_C2_CONFIG_EVTMASK1_BIT2_THREAD_NVM_RAM_UPDATE_ENABLE
                + SHCI_C2_CONFIG_EVTMASK1_BIT3_NVM_START_WRITE_ENABLE
                + SHCI_C2_CONFIG_EVTMASK1_BIT4_NVM_END_WRITE_ENABLE
                + SHCI_C2_CONFIG_EVTMASK1_BIT5_NVM_START_ERASE_ENABLE
                + SHCI_C2_CONFIG_EVTMASK1_BIT6_NVM_END_ERASE_ENABLE,
            spare: 0,
            ble_nvm_ram_address: 0,
            thread_nvm_ram_address: 0,
            revision_id: 0,
            device_id: 0,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ShciBleInitCmdParam {
    /// NOT USED - shall be set to 0
    pub p_ble_buffer_address: u32,
    /// NOT USED - shall be set to 0
    pub ble_buffer_size: u32,
    /// Maximum number of attribute records related to all the required characteristics (excluding the services)
    /// that can be stored in the GATT database, for the specific BLE user application.
    /// For each characteristic, the number of attribute records goes from two to five depending on the characteristic properties:
    ///    - minimum of two (one for declaration and one for the value)
    ///    - add one more record for each additional property: notify or indicate, broadcast, extended property.
    /// The total calculated value must be increased by 9, due to the records related to the standard attribute profile and
    /// GAP service characteristics, and automatically added when initializing GATT and GAP layers
    ///  - Min value: <number of user attributes> + 9
    ///  - Max value: depending on the GATT database defined by user application
    pub num_attr_record: u16,
    /// Defines the maximum number of services that can be stored in the GATT database. Note that the GAP and GATT services
    /// are automatically added at initialization so this parameter must be the number of user services increased by two.
    ///    - Min value: <number of user service> + 2
    ///    - Max value: depending GATT database defined by user application
    pub num_attr_serv: u16,
    /// NOTE: This parameter is ignored by the CPU2 when the parameter "Options" is set to "LL_only" ( see Options description in that structure )
    ///
    /// Size of the storage area for the attribute values.
    /// Each characteristic contributes to the attrValueArrSize value as follows:
    ///    - Characteristic value length plus:
    ///        + 5 bytes if characteristic UUID is 16 bits
    ///        + 19 bytes if characteristic UUID is 128 bits
    ///        + 2 bytes if characteristic has a server configuration descriptor
    ///        + 2 bytes * NumOfLinks if the characteristic has a client configuration descriptor
    ///        + 2 bytes if the characteristic has extended properties
    /// Each descriptor contributes to the attrValueArrSize value as follows:
    ///    - Descriptor length
    pub attr_value_arr_size: u16,
    /// Maximum number of BLE links supported
    ///    - Min value: 1
    ///    - Max value: 8
    pub num_of_links: u8,
    /// Disable/enable the extended packet length BLE 5.0 feature
    ///    - Disable: 0
    ///    - Enable: 1
    pub extended_packet_length_enable: u8,
    /// NOTE: This parameter is ignored by the CPU2 when the parameter "Options" is set to "LL_only" ( see Options description in that structure )
    ///
    /// Maximum number of supported "prepare write request"
    ///    - Min value: given by the macro DEFAULT_PREP_WRITE_LIST_SIZE
    ///    - Max value: a value higher than the minimum required can be specified, but it is not recommended
    pub prepare_write_list_size: u8,
    /// NOTE: This parameter is overwritten by the CPU2 with an hardcoded optimal value when the parameter "Options" is set to "LL_only"
    /// ( see Options description in that structure )
    ///
    /// Number of allocated memory blocks for the BLE stack
    ///     - Min value: given by the macro MBLOCKS_CALC
    ///     - Max value: a higher value can improve data throughput performance, but uses more memory
    pub block_count: u8,
    /// NOTE: This parameter is ignored by the CPU2 when the parameter "Options" is set to "LL_only" ( see Options description in that structure )
    ///
    /// Maximum ATT MTU size supported
    ///     - Min value: 23
    ///     - Max value: 512
    pub att_mtu: u16,
    /// The sleep clock accuracy (ppm value) that used in BLE connected slave mode to calculate the window widening
    /// (in combination with the sleep clock accuracy sent by master in CONNECT_REQ PDU),
    /// refer to BLE 5.0 specifications - Vol 6 - Part B - chap 4.5.7 and 4.2.2
    ///     - Min value: 0
    ///     - Max value: 500 (worst possible admitted by specification)
    pub slave_sca: u16,
    /// The sleep clock accuracy handled in master mode. It is used to determine the connection and advertising events timing.
    /// It is transmitted to the slave in CONNEC_REQ PDU used by the slave to calculate the window widening,
    /// see SlaveSca and Bluetooth Core Specification v5.0 Vol 6 - Part B - chap 4.5.7 and 4.2.2
    /// Possible values:
    ///    - 251 ppm to 500 ppm: 0
    ///    - 151 ppm to 250 ppm: 1
    ///    - 101 ppm to 150 ppm: 2
    ///    - 76 ppm to 100 ppm: 3
    ///    - 51 ppm to 75 ppm: 4
    ///    - 31 ppm to 50 ppm: 5
    ///    - 21 ppm to 30 ppm: 6
    ///    - 0 ppm to 20 ppm: 7
    pub master_sca: u8,
    /// Some information for Low speed clock mapped in bits field
    /// - bit 0:
    ///     - 1: Calibration for the RF system wakeup clock source
    ///     - 0: No calibration for the RF system wakeup clock source
    /// - bit 1:
    ///     - 1: STM32W5M Module device
    ///     - 0: Other devices as STM32WBxx SOC, STM32WB1M module
    /// - bit 2:
    ///     - 1: HSE/1024 Clock config
    ///     - 0: LSE Clock config
    pub ls_source: u8,
    /// This parameter determines the maximum duration of a slave connection event. When this duration is reached the slave closes
    /// the current connections event (whatever is the CE_length parameter specified by the master in HCI_CREATE_CONNECTION HCI command),
    /// expressed in units of 625/256 µs (~2.44 µs)
    ///    - Min value: 0 (if 0 is specified, the master and slave perform only a single TX-RX exchange per connection event)
    ///    - Max value: 1638400 (4000 ms). A higher value can be specified (max 0xFFFFFFFF) but results in a maximum connection time
    ///      of 4000 ms as specified. In this case the parameter is not applied, and the predicted CE length calculated on slave is not shortened
    pub max_conn_event_length: u32,
    /// Startup time of the high speed (16 or 32 MHz) crystal oscillator in units of 625/256 µs (~2.44 µs).
    ///    - Min value: 0
    ///    - Max value:  820 (~2 ms). A higher value can be specified, but the value that implemented in stack is forced to ~2 ms
    pub hs_startup_time: u16,
    /// Viterbi implementation in BLE LL reception.
    ///    - 0: Enable
    ///    - 1: Disable
    pub viterbi_enable: u8,
    /// - bit 0:
    ///     - 1: LL only
    ///     - 0: LL + host
    /// - bit 1:
    ///     - 1: no service change desc.
    ///     - 0: with service change desc.
    /// - bit 2:
    ///     - 1: device name Read-Only
    ///     - 0: device name R/W
    /// - bit 3:
    ///     - 1: extended advertizing supported
    ///     - 0: extended advertizing not supported
    /// - bit 4:
    ///     - 1: CS Algo #2 supported
    ///     - 0: CS Algo #2 not supported
    /// - bit 5:
    ///     - 1: Reduced GATT database in NVM
    ///     - 0: Full GATT database in NVM
    /// - bit 6:
    ///     - 1: GATT caching is used
    ///     - 0: GATT caching is not used
    /// - bit 7:
    ///     - 1: LE Power Class 1
    ///     - 0: LE Power Classe 2-3
    /// - other bits: complete with Options_extension flag
    pub options: u8,
    /// Reserved for future use - shall be set to 0
    pub hw_version: u8,
    //    /**
    //     * Maximum number of connection-oriented channels in initiator mode.
    //     * Range: 0 .. 64
    //     */
    //    pub max_coc_initiator_nbr: u8,
    //
    //    /**
    //     * Minimum transmit power in dBm supported by the Controller.
    //     * Range: -127 .. 20
    //     */
    //    pub min_tx_power: i8,
    //
    //    /**
    //     * Maximum transmit power in dBm supported by the Controller.
    //     * Range: -127 .. 20
    //     */
    //    pub max_tx_power: i8,
    //
    //    /**
    //     * RX model configuration
    //     * - bit 0:   1: agc_rssi model improved vs RF blockers    0: Legacy agc_rssi model
    //     * - other bits: reserved ( shall be set to 0)
    //     */
    //    pub rx_model_config: u8,
    //
    //    /* Maximum number of advertising sets.
    //     * Range: 1 .. 8 with limitation:
    //     * This parameter is linked to max_adv_data_len such as both compliant with allocated Total memory computed with BLE_EXT_ADV_BUFFER_SIZE based
    //     * on Max Extended advertising configuration supported.
    //     * This parameter is considered by the CPU2 when Options has SHCI_C2_BLE_INIT_OPTIONS_EXT_ADV flag set
    //     */
    //    pub max_adv_set_nbr: u8,
    //
    //    /* Maximum advertising data length (in bytes)
    //     * Range: 31 .. 1650 with limitation:
    //     * This parameter is linked to max_adv_set_nbr such as both compliant with allocated Total memory computed with BLE_EXT_ADV_BUFFER_SIZE based
    //     * on Max Extended advertising configuration supported.
    //     * This parameter is considered by the CPU2 when Options has SHCI_C2_BLE_INIT_OPTIONS_EXT_ADV flag set
    //     */
    //    pub max_adv_data_len: u16,
    //
    //    /* RF TX Path Compensation Value (16-bit signed integer). Units: 0.1 dB.
    //     * Range: -1280 .. 1280
    //     */
    //    pub tx_path_compens: i16,
    //
    //    /* RF RX Path Compensation Value (16-bit signed integer). Units: 0.1 dB.
    //     * Range: -1280 .. 1280
    //     */
    //    pub rx_path_compens: i16,
    //
    //    /* BLE core specification version (8-bit unsigned integer).
    //     * values as: 11(5.2), 12(5.3)
    //     */
    //    pub ble_core_version: u8,
    //
    //    /**
    //     * Options flags extension
    //     * - bit 0:   1: appearance Writable              0: appearance Read-Only
    //     * - bit 1:   1: Enhanced ATT supported           0: Enhanced ATT not supported
    //     * - other bits: reserved ( shall be set to 0)
    //     */
    //    pub options_extension: u8,
}

impl ShciBleInitCmdParam {
    pub fn payload<'a>(&'a self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Self>()) }
    }
}

impl Default for ShciBleInitCmdParam {
    fn default() -> Self {
        Self {
            p_ble_buffer_address: 0,
            ble_buffer_size: 0,
            num_attr_record: 68,
            num_attr_serv: 8,
            attr_value_arr_size: 1344,
            num_of_links: 2,
            extended_packet_length_enable: 1,
            prepare_write_list_size: 0x3A,
            block_count: 0x79,
            att_mtu: 156,
            slave_sca: 500,
            master_sca: 0,
            ls_source: 1,
            max_conn_event_length: 0xFFFFFFFF,
            hs_startup_time: 0x148,
            viterbi_enable: 1,
            options: 0,
            hw_version: 0,
        }
    }
}

pub const TL_BLE_EVT_CS_PACKET_SIZE: usize = TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE;
#[allow(dead_code)] // Not used currently but reserved
const TL_BLE_EVT_CS_BUFFER_SIZE: usize = TL_PACKET_HEADER_SIZE + TL_BLE_EVT_CS_PACKET_SIZE;
