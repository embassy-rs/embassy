use heapless::{String, Vec};

/// internal supporting structures for CtrlMsg

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScanResult {
    #[noproto(tag = "1")]
    pub ssid: String<32>,
    #[noproto(tag = "2")]
    pub chnl: u32,
    #[noproto(tag = "3")]
    pub rssi: u32,
    #[noproto(tag = "4")]
    pub bssid: String<32>,
    #[noproto(tag = "5")]
    pub sec_prot: CtrlWifiSecProt,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConnectedStaList {
    #[noproto(tag = "1")]
    pub mac: String<32>,
    #[noproto(tag = "2")]
    pub rssi: u32,
}
/// * Req/Resp structure *

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqGetMacAddress {
    #[noproto(tag = "1")]
    pub mode: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespGetMacAddress {
    #[noproto(tag = "1")]
    pub mac: String<32>,
    #[noproto(tag = "2")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqGetMode {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespGetMode {
    #[noproto(tag = "1")]
    pub mode: u32,
    #[noproto(tag = "2")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqSetMode {
    #[noproto(tag = "1")]
    pub mode: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespSetMode {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqGetStatus {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespGetStatus {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqSetMacAddress {
    #[noproto(tag = "1")]
    pub mac: String<32>,
    #[noproto(tag = "2")]
    pub mode: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespSetMacAddress {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqGetApConfig {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespGetApConfig {
    #[noproto(tag = "1")]
    pub ssid: String<32>,
    #[noproto(tag = "2")]
    pub bssid: String<32>,
    #[noproto(tag = "3")]
    pub rssi: u32,
    #[noproto(tag = "4")]
    pub chnl: u32,
    #[noproto(tag = "5")]
    pub sec_prot: CtrlWifiSecProt,
    #[noproto(tag = "6")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqConnectAp {
    #[noproto(tag = "1")]
    pub ssid: String<32>,
    #[noproto(tag = "2")]
    pub pwd: String<32>,
    #[noproto(tag = "3")]
    pub bssid: String<32>,
    #[noproto(tag = "4")]
    pub is_wpa3_supported: bool,
    #[noproto(tag = "5")]
    pub listen_interval: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespConnectAp {
    #[noproto(tag = "1")]
    pub resp: u32,
    #[noproto(tag = "2")]
    pub mac: String<32>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqGetSoftApConfig {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespGetSoftApConfig {
    #[noproto(tag = "1")]
    pub ssid: String<32>,
    #[noproto(tag = "2")]
    pub pwd: String<32>,
    #[noproto(tag = "3")]
    pub chnl: u32,
    #[noproto(tag = "4")]
    pub sec_prot: CtrlWifiSecProt,
    #[noproto(tag = "5")]
    pub max_conn: u32,
    #[noproto(tag = "6")]
    pub ssid_hidden: bool,
    #[noproto(tag = "7")]
    pub bw: u32,
    #[noproto(tag = "8")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqStartSoftAp {
    #[noproto(tag = "1")]
    pub ssid: String<32>,
    #[noproto(tag = "2")]
    pub pwd: String<32>,
    #[noproto(tag = "3")]
    pub chnl: u32,
    #[noproto(tag = "4")]
    pub sec_prot: CtrlWifiSecProt,
    #[noproto(tag = "5")]
    pub max_conn: u32,
    #[noproto(tag = "6")]
    pub ssid_hidden: bool,
    #[noproto(tag = "7")]
    pub bw: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespStartSoftAp {
    #[noproto(tag = "1")]
    pub resp: u32,
    #[noproto(tag = "2")]
    pub mac: String<32>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqScanResult {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespScanResult {
    #[noproto(tag = "1")]
    pub count: u32,
    #[noproto(repeated, tag = "2")]
    pub entries: Vec<ScanResult, 16>,
    #[noproto(tag = "3")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqSoftApConnectedSta {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespSoftApConnectedSta {
    #[noproto(tag = "1")]
    pub num: u32,
    #[noproto(repeated, tag = "2")]
    pub stations: Vec<ConnectedStaList, 16>,
    #[noproto(tag = "3")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqOtaBegin {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespOtaBegin {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqOtaWrite {
    #[noproto(tag = "1")]
    pub ota_data: Vec<u8, 1024>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespOtaWrite {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqOtaEnd {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespOtaEnd {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqVendorIeData {
    #[noproto(tag = "1")]
    pub element_id: u32,
    #[noproto(tag = "2")]
    pub length: u32,
    #[noproto(tag = "3")]
    pub vendor_oui: Vec<u8, 8>,
    #[noproto(tag = "4")]
    pub vendor_oui_type: u32,
    #[noproto(tag = "5")]
    pub payload: Vec<u8, 64>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqSetSoftApVendorSpecificIe {
    #[noproto(tag = "1")]
    pub enable: bool,
    #[noproto(tag = "2")]
    pub r#type: CtrlVendorIeType,
    #[noproto(tag = "3")]
    pub idx: CtrlVendorIeid,
    #[noproto(optional, tag = "4")]
    pub vendor_ie_data: Option<CtrlMsgReqVendorIeData>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespSetSoftApVendorSpecificIe {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqSetWifiMaxTxPower {
    #[noproto(tag = "1")]
    pub wifi_max_tx_power: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespSetWifiMaxTxPower {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqGetWifiCurrTxPower {}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespGetWifiCurrTxPower {
    #[noproto(tag = "1")]
    pub wifi_curr_tx_power: u32,
    #[noproto(tag = "2")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgReqConfigHeartbeat {
    #[noproto(tag = "1")]
    pub enable: bool,
    #[noproto(tag = "2")]
    pub duration: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgRespConfigHeartbeat {
    #[noproto(tag = "1")]
    pub resp: u32,
}
/// * Event structure *

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgEventEspInit {
    #[noproto(tag = "1")]
    pub init_data: Vec<u8, 64>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgEventHeartbeat {
    #[noproto(tag = "1")]
    pub hb_num: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgEventStationDisconnectFromAp {
    #[noproto(tag = "1")]
    pub resp: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgEventStationDisconnectFromEspSoftAp {
    #[noproto(tag = "1")]
    pub resp: u32,
    #[noproto(tag = "2")]
    pub mac: String<32>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, noproto::Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg {
    /// msg_type could be req, resp or Event
    #[noproto(tag = "1")]
    pub msg_type: CtrlMsgType,
    /// msg id
    #[noproto(tag = "2")]
    pub msg_id: CtrlMsgId,
    /// union of all msg ids
    #[noproto(
        oneof,
        tags = "101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 301, 302, 303, 304"
    )]
    pub payload: Option<CtrlMsgPayload>,
}

/// union of all msg ids
#[derive(Debug, Clone, Eq, PartialEq, noproto::Oneof)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlMsgPayload {
    /// * Requests *
    #[noproto(tag = "101")]
    ReqGetMacAddress(CtrlMsgReqGetMacAddress),
    #[noproto(tag = "102")]
    ReqSetMacAddress(CtrlMsgReqSetMacAddress),
    #[noproto(tag = "103")]
    ReqGetWifiMode(CtrlMsgReqGetMode),
    #[noproto(tag = "104")]
    ReqSetWifiMode(CtrlMsgReqSetMode),
    #[noproto(tag = "105")]
    ReqScanApList(CtrlMsgReqScanResult),
    #[noproto(tag = "106")]
    ReqGetApConfig(CtrlMsgReqGetApConfig),
    #[noproto(tag = "107")]
    ReqConnectAp(CtrlMsgReqConnectAp),
    #[noproto(tag = "108")]
    ReqDisconnectAp(CtrlMsgReqGetStatus),
    #[noproto(tag = "109")]
    ReqGetSoftapConfig(CtrlMsgReqGetSoftApConfig),
    #[noproto(tag = "110")]
    ReqSetSoftapVendorSpecificIe(CtrlMsgReqSetSoftApVendorSpecificIe),
    #[noproto(tag = "111")]
    ReqStartSoftap(CtrlMsgReqStartSoftAp),
    #[noproto(tag = "112")]
    ReqSoftapConnectedStasList(CtrlMsgReqSoftApConnectedSta),
    #[noproto(tag = "113")]
    ReqStopSoftap(CtrlMsgReqGetStatus),
    #[noproto(tag = "114")]
    ReqSetPowerSaveMode(CtrlMsgReqSetMode),
    #[noproto(tag = "115")]
    ReqGetPowerSaveMode(CtrlMsgReqGetMode),
    #[noproto(tag = "116")]
    ReqOtaBegin(CtrlMsgReqOtaBegin),
    #[noproto(tag = "117")]
    ReqOtaWrite(CtrlMsgReqOtaWrite),
    #[noproto(tag = "118")]
    ReqOtaEnd(CtrlMsgReqOtaEnd),
    #[noproto(tag = "119")]
    ReqSetWifiMaxTxPower(CtrlMsgReqSetWifiMaxTxPower),
    #[noproto(tag = "120")]
    ReqGetWifiCurrTxPower(CtrlMsgReqGetWifiCurrTxPower),
    #[noproto(tag = "121")]
    ReqConfigHeartbeat(CtrlMsgReqConfigHeartbeat),
    /// * Responses *
    #[noproto(tag = "201")]
    RespGetMacAddress(CtrlMsgRespGetMacAddress),
    #[noproto(tag = "202")]
    RespSetMacAddress(CtrlMsgRespSetMacAddress),
    #[noproto(tag = "203")]
    RespGetWifiMode(CtrlMsgRespGetMode),
    #[noproto(tag = "204")]
    RespSetWifiMode(CtrlMsgRespSetMode),
    #[noproto(tag = "205")]
    RespScanApList(CtrlMsgRespScanResult),
    #[noproto(tag = "206")]
    RespGetApConfig(CtrlMsgRespGetApConfig),
    #[noproto(tag = "207")]
    RespConnectAp(CtrlMsgRespConnectAp),
    #[noproto(tag = "208")]
    RespDisconnectAp(CtrlMsgRespGetStatus),
    #[noproto(tag = "209")]
    RespGetSoftapConfig(CtrlMsgRespGetSoftApConfig),
    #[noproto(tag = "210")]
    RespSetSoftapVendorSpecificIe(CtrlMsgRespSetSoftApVendorSpecificIe),
    #[noproto(tag = "211")]
    RespStartSoftap(CtrlMsgRespStartSoftAp),
    #[noproto(tag = "212")]
    RespSoftapConnectedStasList(CtrlMsgRespSoftApConnectedSta),
    #[noproto(tag = "213")]
    RespStopSoftap(CtrlMsgRespGetStatus),
    #[noproto(tag = "214")]
    RespSetPowerSaveMode(CtrlMsgRespSetMode),
    #[noproto(tag = "215")]
    RespGetPowerSaveMode(CtrlMsgRespGetMode),
    #[noproto(tag = "216")]
    RespOtaBegin(CtrlMsgRespOtaBegin),
    #[noproto(tag = "217")]
    RespOtaWrite(CtrlMsgRespOtaWrite),
    #[noproto(tag = "218")]
    RespOtaEnd(CtrlMsgRespOtaEnd),
    #[noproto(tag = "219")]
    RespSetWifiMaxTxPower(CtrlMsgRespSetWifiMaxTxPower),
    #[noproto(tag = "220")]
    RespGetWifiCurrTxPower(CtrlMsgRespGetWifiCurrTxPower),
    #[noproto(tag = "221")]
    RespConfigHeartbeat(CtrlMsgRespConfigHeartbeat),
    /// * Notifications *
    #[noproto(tag = "301")]
    EventEspInit(CtrlMsgEventEspInit),
    #[noproto(tag = "302")]
    EventHeartbeat(CtrlMsgEventHeartbeat),
    #[noproto(tag = "303")]
    EventStationDisconnectFromAp(CtrlMsgEventStationDisconnectFromAp),
    #[noproto(tag = "304")]
    EventStationDisconnectFromEspSoftAp(CtrlMsgEventStationDisconnectFromEspSoftAp),
}

/// Enums similar to ESP IDF
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlVendorIeType {
    #[default]
    Beacon = 0,
    ProbeReq = 1,
    ProbeResp = 2,
    AssocReq = 3,
    AssocResp = 4,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlVendorIeid {
    #[default]
    Id0 = 0,
    Id1 = 1,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlWifiMode {
    #[default]
    None = 0,
    Sta = 1,
    Ap = 2,
    Apsta = 3,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlWifiBw {
    #[default]
    BwInvalid = 0,
    Ht20 = 1,
    Ht40 = 2,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlWifiPowerSave {
    #[default]
    PsInvalid = 0,
    MinModem = 1,
    MaxModem = 2,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlWifiSecProt {
    #[default]
    Open = 0,
    Wep = 1,
    WpaPsk = 2,
    Wpa2Psk = 3,
    WpaWpa2Psk = 4,
    Wpa2Enterprise = 5,
    Wpa3Psk = 6,
    Wpa2Wpa3Psk = 7,
}

/// enums for Control path
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlStatus {
    #[default]
    Connected = 0,
    NotConnected = 1,
    NoApFound = 2,
    ConnectionFail = 3,
    InvalidArgument = 4,
    OutOfRange = 5,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlMsgType {
    #[default]
    MsgTypeInvalid = 0,
    Req = 1,
    Resp = 2,
    Event = 3,
    MsgTypeMax = 4,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, noproto::Enumeration)]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CtrlMsgId {
    #[default]
    MsgIdInvalid = 0,
    /// * Request Msgs *
    ReqBase = 100,
    ReqGetMacAddress = 101,
    ReqSetMacAddress = 102,
    ReqGetWifiMode = 103,
    ReqSetWifiMode = 104,
    ReqGetApScanList = 105,
    ReqGetApConfig = 106,
    ReqConnectAp = 107,
    ReqDisconnectAp = 108,
    ReqGetSoftApConfig = 109,
    ReqSetSoftApVendorSpecificIe = 110,
    ReqStartSoftAp = 111,
    ReqGetSoftApConnectedStaList = 112,
    ReqStopSoftAp = 113,
    ReqSetPowerSaveMode = 114,
    ReqGetPowerSaveMode = 115,
    ReqOtaBegin = 116,
    ReqOtaWrite = 117,
    ReqOtaEnd = 118,
    ReqSetWifiMaxTxPower = 119,
    ReqGetWifiCurrTxPower = 120,
    ReqConfigHeartbeat = 121,
    /// Add new control path command response before Req_Max
    /// and update Req_Max
    ReqMax = 122,
    /// * Response Msgs *
    RespBase = 200,
    RespGetMacAddress = 201,
    RespSetMacAddress = 202,
    RespGetWifiMode = 203,
    RespSetWifiMode = 204,
    RespGetApScanList = 205,
    RespGetApConfig = 206,
    RespConnectAp = 207,
    RespDisconnectAp = 208,
    RespGetSoftApConfig = 209,
    RespSetSoftApVendorSpecificIe = 210,
    RespStartSoftAp = 211,
    RespGetSoftApConnectedStaList = 212,
    RespStopSoftAp = 213,
    RespSetPowerSaveMode = 214,
    RespGetPowerSaveMode = 215,
    RespOtaBegin = 216,
    RespOtaWrite = 217,
    RespOtaEnd = 218,
    RespSetWifiMaxTxPower = 219,
    RespGetWifiCurrTxPower = 220,
    RespConfigHeartbeat = 221,
    /// Add new control path command response before Resp_Max
    /// and update Resp_Max
    RespMax = 222,
    /// * Event Msgs *
    EventBase = 300,
    EventEspInit = 301,
    EventHeartbeat = 302,
    EventStationDisconnectFromAp = 303,
    EventStationDisconnectFromEspSoftAp = 304,
    /// Add new control path command notification before Event_Max
    /// and update Event_Max
    EventMax = 305,
}
