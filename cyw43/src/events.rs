#![allow(dead_code)]
#![allow(non_camel_case_types)]

use core::cell::RefCell;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Subscriber};

use crate::structs::BssInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Event {
    #[num_enum(default)]
    Unknown = 0xFF,
    /// indicates status of set SSID
    SET_SSID = 0,
    /// differentiates join IBSS from found (START) IBSS
    JOIN = 1,
    /// STA founded an IBSS or AP started a BSS
    START = 2,
    /// 802.11 AUTH request
    AUTH = 3,
    /// 802.11 AUTH indication
    AUTH_IND = 4,
    /// 802.11 DEAUTH request
    DEAUTH = 5,
    /// 802.11 DEAUTH indication
    DEAUTH_IND = 6,
    /// 802.11 ASSOC request
    ASSOC = 7,
    /// 802.11 ASSOC indication
    ASSOC_IND = 8,
    /// 802.11 REASSOC request
    REASSOC = 9,
    /// 802.11 REASSOC indication
    REASSOC_IND = 10,
    /// 802.11 DISASSOC request
    DISASSOC = 11,
    /// 802.11 DISASSOC indication
    DISASSOC_IND = 12,
    /// 802.11h Quiet period started
    QUIET_START = 13,
    /// 802.11h Quiet period ended
    QUIET_END = 14,
    /// BEACONS received/lost indication
    BEACON_RX = 15,
    /// generic link indication
    LINK = 16,
    /// TKIP MIC error occurred
    MIC_ERROR = 17,
    /// NDIS style link indication
    NDIS_LINK = 18,
    /// roam attempt occurred: indicate status & reason
    ROAM = 19,
    /// change in dot11FailedCount (txfail)
    TXFAIL = 20,
    /// WPA2 pmkid cache indication
    PMKID_CACHE = 21,
    /// current AP's TSF value went backward
    RETROGRADE_TSF = 22,
    /// AP was pruned from join list for reason
    PRUNE = 23,
    /// report AutoAuth table entry match for join attempt
    AUTOAUTH = 24,
    /// Event encapsulating an EAPOL message
    EAPOL_MSG = 25,
    /// Scan results are ready or scan was aborted
    SCAN_COMPLETE = 26,
    /// indicate to host addts fail/success
    ADDTS_IND = 27,
    /// indicate to host delts fail/success
    DELTS_IND = 28,
    /// indicate to host of beacon transmit
    BCNSENT_IND = 29,
    /// Send the received beacon up to the host
    BCNRX_MSG = 30,
    /// indicate to host loss of beacon
    BCNLOST_MSG = 31,
    /// before attempting to roam
    ROAM_PREP = 32,
    /// PFN network found event
    PFN_NET_FOUND = 33,
    /// PFN network lost event
    PFN_NET_LOST = 34,
    RESET_COMPLETE = 35,
    JOIN_START = 36,
    ROAM_START = 37,
    ASSOC_START = 38,
    IBSS_ASSOC = 39,
    RADIO = 40,
    /// PSM microcode watchdog fired
    PSM_WATCHDOG = 41,
    /// CCX association start
    CCX_ASSOC_START = 42,
    /// CCX association abort
    CCX_ASSOC_ABORT = 43,
    /// probe request received
    PROBREQ_MSG = 44,
    SCAN_CONFIRM_IND = 45,
    /// WPA Handshake
    PSK_SUP = 46,
    COUNTRY_CODE_CHANGED = 47,
    /// WMMAC excedded medium time
    EXCEEDED_MEDIUM_TIME = 48,
    /// WEP ICV error occurred
    ICV_ERROR = 49,
    /// Unsupported unicast encrypted frame
    UNICAST_DECODE_ERROR = 50,
    /// Unsupported multicast encrypted frame
    MULTICAST_DECODE_ERROR = 51,
    TRACE = 52,
    /// BT-AMP HCI event
    BTA_HCI_EVENT = 53,
    /// I/F change (for wlan host notification)
    IF = 54,
    /// P2P Discovery listen state expires
    P2P_DISC_LISTEN_COMPLETE = 55,
    /// indicate RSSI change based on configured levels
    RSSI = 56,
    /// PFN best network batching event
    PFN_BEST_BATCHING = 57,
    EXTLOG_MSG = 58,
    /// Action frame reception
    ACTION_FRAME = 59,
    /// Action frame Tx complete
    ACTION_FRAME_COMPLETE = 60,
    /// assoc request received
    PRE_ASSOC_IND = 61,
    /// re-assoc request received
    PRE_REASSOC_IND = 62,
    /// channel adopted (xxx: obsoleted)
    CHANNEL_ADOPTED = 63,
    /// AP started
    AP_STARTED = 64,
    /// AP stopped due to DFS
    DFS_AP_STOP = 65,
    /// AP resumed due to DFS
    DFS_AP_RESUME = 66,
    /// WAI stations event
    WAI_STA_EVENT = 67,
    /// event encapsulating an WAI message
    WAI_MSG = 68,
    /// escan result event
    ESCAN_RESULT = 69,
    /// action frame off channel complete
    ACTION_FRAME_OFF_CHAN_COMPLETE = 70,
    /// probe response received
    PROBRESP_MSG = 71,
    /// P2P Probe request received
    P2P_PROBREQ_MSG = 72,
    DCS_REQUEST = 73,
    /// credits for D11 FIFOs. [AC0,AC1,AC2,AC3,BC_MC,ATIM]
    FIFO_CREDIT_MAP = 74,
    /// Received action frame event WITH wl_event_rx_frame_data_t header
    ACTION_FRAME_RX = 75,
    /// Wake Event timer fired, used for wake WLAN test mode
    WAKE_EVENT = 76,
    /// Radio measurement complete
    RM_COMPLETE = 77,
    /// Synchronize TSF with the host
    HTSFSYNC = 78,
    /// request an overlay IOCTL/iovar from the host
    OVERLAY_REQ = 79,
    CSA_COMPLETE_IND = 80,
    /// excess PM Wake Event to inform host
    EXCESS_PM_WAKE_EVENT = 81,
    /// no PFN networks around
    PFN_SCAN_NONE = 82,
    /// last found PFN network gets lost
    PFN_SCAN_ALLGONE = 83,
    GTK_PLUMBED = 84,
    /// 802.11 ASSOC indication for NDIS only
    ASSOC_IND_NDIS = 85,
    /// 802.11 REASSOC indication for NDIS only
    REASSOC_IND_NDIS = 86,
    ASSOC_REQ_IE = 87,
    ASSOC_RESP_IE = 88,
    /// association recreated on resume
    ASSOC_RECREATED = 89,
    /// rx action frame event for NDIS only
    ACTION_FRAME_RX_NDIS = 90,
    /// authentication request received
    AUTH_REQ = 91,
    /// fast assoc recreation failed
    SPEEDY_RECREATE_FAIL = 93,
    /// port-specific event and payload (e.g. NDIS)
    NATIVE = 94,
    /// event for tx pkt delay suddently jump
    PKTDELAY_IND = 95,
    /// AWDL AW period starts
    AWDL_AW = 96,
    /// AWDL Master/Slave/NE master role event
    AWDL_ROLE = 97,
    /// Generic AWDL event
    AWDL_EVENT = 98,
    /// NIC AF txstatus
    NIC_AF_TXS = 99,
    /// NAN event
    NAN = 100,
    BEACON_FRAME_RX = 101,
    /// desired service found
    SERVICE_FOUND = 102,
    /// GAS fragment received
    GAS_FRAGMENT_RX = 103,
    /// GAS sessions all complete
    GAS_COMPLETE = 104,
    /// New device found by p2p offload
    P2PO_ADD_DEVICE = 105,
    /// device has been removed by p2p offload
    P2PO_DEL_DEVICE = 106,
    /// WNM event to notify STA enter sleep mode
    WNM_STA_SLEEP = 107,
    /// Indication of MAC tx failures (exhaustion of 802.11 retries) exceeding threshold(s)
    TXFAIL_THRESH = 108,
    /// Proximity Detection event
    PROXD = 109,
    /// AWDL RX Probe response
    AWDL_RX_PRB_RESP = 111,
    /// AWDL RX Action Frames
    AWDL_RX_ACT_FRAME = 112,
    /// AWDL Wowl nulls
    AWDL_WOWL_NULLPKT = 113,
    /// AWDL Phycal status
    AWDL_PHYCAL_STATUS = 114,
    /// AWDL OOB AF status
    AWDL_OOB_AF_STATUS = 115,
    /// Interleaved Scan status
    AWDL_SCAN_STATUS = 116,
    /// AWDL AW Start
    AWDL_AW_START = 117,
    /// AWDL AW End
    AWDL_AW_END = 118,
    /// AWDL AW Extensions
    AWDL_AW_EXT = 119,
    AWDL_PEER_CACHE_CONTROL = 120,
    CSA_START_IND = 121,
    CSA_DONE_IND = 122,
    CSA_FAILURE_IND = 123,
    /// CCA based channel quality report
    CCA_CHAN_QUAL = 124,
    /// to report change in BSSID while roaming
    BSSID = 125,
    /// tx error indication
    TX_STAT_ERROR = 126,
    /// credit check for BCMC supported
    BCMC_CREDIT_SUPPORT = 127,
    /// psta primary interface indication
    PSTA_PRIMARY_INTF_IND = 128,
    /// Handover Request Initiated
    BT_WIFI_HANDOVER_REQ = 130,
    /// Southpaw TxInhibit notification
    SPW_TXINHIBIT = 131,
    /// FBT Authentication Request Indication
    FBT_AUTH_REQ_IND = 132,
    /// Enhancement addition for RSSI
    RSSI_LQM = 133,
    /// Full probe/beacon (IEs etc) results
    PFN_GSCAN_FULL_RESULT = 134,
    /// Significant change in rssi of bssids being tracked
    PFN_SWC = 135,
    /// a STA been authroized for traffic
    AUTHORIZED = 136,
    /// probe req with wl_event_rx_frame_data_t header
    PROBREQ_MSG_RX = 137,
    /// PFN completed scan of network list
    PFN_SCAN_COMPLETE = 138,
    /// RMC Event
    RMC_EVENT = 139,
    /// DPSTA interface indication
    DPSTA_INTF_IND = 140,
    /// RRM Event
    RRM = 141,
    /// ULP entry event
    ULP = 146,
    /// TCP Keep Alive Offload Event
    TKO = 151,
    /// authentication request received
    EXT_AUTH_REQ = 187,
    /// authentication request received
    EXT_AUTH_FRAME_RX = 188,
    /// mgmt frame Tx complete
    MGMT_FRAME_TXSTATUS = 189,
    /// highest val + 1 for range checking
    LAST = 190,
}

// TODO this PubSub can probably be replaced with shared memory to make it a bit more efficient.
pub type EventQueue = PubSubChannel<NoopRawMutex, Message, 2, 1, 1>;
pub type EventSubscriber<'a> = Subscriber<'a, NoopRawMutex, Message, 2, 1, 1>;

pub struct Events {
    pub queue: EventQueue,
    pub mask: SharedEventMask,
}

impl Events {
    pub fn new() -> Self {
        Self {
            queue: EventQueue::new(),
            mask: SharedEventMask::default(),
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    pub event_type: Event,
    pub status: u32,
}

#[derive(Clone, Copy)]
pub enum Payload {
    None,
    BssInfo(BssInfo),
}

#[derive(Clone, Copy)]

pub struct Message {
    pub header: Status,
    pub payload: Payload,
}

impl Message {
    pub fn new(status: Status, payload: Payload) -> Self {
        Self {
            header: status,
            payload,
        }
    }
}

#[derive(Default)]
struct EventMask {
    mask: [u32; Self::WORD_COUNT],
}

impl EventMask {
    const WORD_COUNT: usize = ((Event::LAST as u32 + (u32::BITS - 1)) / u32::BITS) as usize;

    fn enable(&mut self, event: Event) {
        let n = event as u32;
        let word = n / u32::BITS;
        let bit = n % u32::BITS;

        self.mask[word as usize] |= 1 << bit;
    }

    fn disable(&mut self, event: Event) {
        let n = event as u32;
        let word = n / u32::BITS;
        let bit = n % u32::BITS;

        self.mask[word as usize] &= !(1 << bit);
    }

    fn is_enabled(&self, event: Event) -> bool {
        let n = event as u32;
        let word = n / u32::BITS;
        let bit = n % u32::BITS;

        self.mask[word as usize] & (1 << bit) > 0
    }
}

#[derive(Default)]

pub struct SharedEventMask {
    mask: RefCell<EventMask>,
}

impl SharedEventMask {
    pub fn enable(&self, events: &[Event]) {
        let mut mask = self.mask.borrow_mut();
        for event in events {
            mask.enable(*event);
        }
    }

    #[allow(dead_code)]
    pub fn disable(&self, events: &[Event]) {
        let mut mask = self.mask.borrow_mut();
        for event in events {
            mask.disable(*event);
        }
    }

    pub fn disable_all(&self) {
        let mut mask = self.mask.borrow_mut();
        mask.mask = Default::default();
    }

    pub fn is_enabled(&self, event: Event) -> bool {
        let mask = self.mask.borrow();
        mask.is_enabled(event)
    }
}
