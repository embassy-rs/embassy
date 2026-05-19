//! Constant Tone Extension (CTE) event types — BT 5.1+ Direction Finding.
//!
//! stm32wb_hci drops LE meta subevents 0x15 and 0x16 as unknown events.
//! This module intercepts those raw bytes and parses them before stm32wb_hci
//! discards them, enabling direction finding applications.

/// One IQ sample from an LE Connection IQ Report.
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IqSample {
    pub i: i8,
    pub q: i8,
}

/// LE meta subevent 0x15 — LE Connection IQ Report.
///
/// Delivered to the locator (central) when the peripheral sends a CTE-bearing packet.
/// Contains up to 82 IQ samples collected during the CTE.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LeConnectionIqReport {
    pub conn_handle: u16,
    /// RX PHY: 0x01=1M, 0x02=2M
    pub rx_phy: u8,
    pub data_channel_index: u8,
    /// RSSI in units of 0.1 dBm
    pub rssi: i16,
    pub rssi_antenna_id: u8,
    /// CTE type: 0x00=AoA, 0x01=AoD 1μs, 0x02=AoD 2μs
    pub cte_type: u8,
    /// Slot durations: 0x01=1μs, 0x02=2μs
    pub slot_durations: u8,
    /// 0x00=CRC OK, 0x01=CRC no match, 0xFF=insufficient resources
    pub packet_status: u8,
    pub connection_event_counter: u16,
    pub sample_count: u8,
    pub samples: [IqSample; 82],
}

/// LE meta subevent 0x16 — LE CTE Request Failed.
///
/// Delivered to the locator when the peripheral could not satisfy a CTE request.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LeCteRequestFailed {
    pub status: u8,
    pub conn_handle: u16,
}

/// A CTE direction finding event, parsed from LE meta subevents 0x15/0x16.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CteEvent {
    ConnectionIqReport(LeConnectionIqReport),
    CteRequestFailed(LeCteRequestFailed),
}

// HCI / LE meta subevent codes for CTE events
const HCI_EVENT_INDICATOR: u8 = 0x04;
pub(crate) const LE_META_EVENT: u8 = 0x3E;
pub(crate) const LE_CONNECTION_IQ_REPORT: u8 = 0x15;
pub(crate) const LE_CTE_REQUEST_FAILED: u8 = 0x16;

/// Try to parse a CTE event from a full raw HCI packet.
///
/// `raw` must be a complete HCI packet:
/// `[0x04][event_code][param_len][params...]`
///
/// Returns `Some(CteEvent)` for subevent 0x15 or 0x16, `None` otherwise.
pub fn parse_cte_event(raw: &[u8]) -> Option<CteEvent> {
    if raw.len() < 4 {
        return None;
    }
    if raw[0] != HCI_EVENT_INDICATOR || raw[1] != LE_META_EVENT {
        return None;
    }
    let subevent = raw[3];
    let params = &raw[4..];

    match subevent {
        LE_CONNECTION_IQ_REPORT => parse_connection_iq_report(params),
        LE_CTE_REQUEST_FAILED => parse_cte_request_failed(params),
        _ => None,
    }
}

fn parse_connection_iq_report(params: &[u8]) -> Option<CteEvent> {
    // conn_handle(2) + rx_phy(1) + channel(1) + rssi(2) + rssi_ant(1) +
    // cte_type(1) + slot_dur(1) + pkt_status(1) + event_ctr(2) + sample_count(1) = 13 bytes
    if params.len() < 13 {
        return None;
    }
    let conn_handle = u16::from_le_bytes([params[0], params[1]]);
    let rx_phy = params[2];
    let data_channel_index = params[3];
    let rssi = i16::from_le_bytes([params[4], params[5]]);
    let rssi_antenna_id = params[6];
    let cte_type = params[7];
    let slot_durations = params[8];
    let packet_status = params[9];
    let connection_event_counter = u16::from_le_bytes([params[10], params[11]]);
    let sample_count = params[12] as usize;

    if params.len() < 13 + sample_count * 2 {
        return None;
    }
    let count = sample_count.min(82);
    let mut samples = [IqSample::default(); 82];
    for i in 0..count {
        samples[i] = IqSample {
            i: params[13 + i * 2] as i8,
            q: params[14 + i * 2] as i8,
        };
    }

    Some(CteEvent::ConnectionIqReport(LeConnectionIqReport {
        conn_handle,
        rx_phy,
        data_channel_index,
        rssi,
        rssi_antenna_id,
        cte_type,
        slot_durations,
        packet_status,
        connection_event_counter,
        sample_count: count as u8,
        samples,
    }))
}

fn parse_cte_request_failed(params: &[u8]) -> Option<CteEvent> {
    if params.len() < 3 {
        return None;
    }
    Some(CteEvent::CteRequestFailed(LeCteRequestFailed {
        status: params[0],
        conn_handle: u16::from_le_bytes([params[1], params[2]]),
    }))
}
