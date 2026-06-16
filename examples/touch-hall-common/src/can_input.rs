//! Latest CAN RX frame store and minp edge detection.
//!
//! Arm on debounced `00`, pulse `minp_active()` once on the next raw `0→1`.

use core::sync::atomic::{AtomicU32, AtomicU8, AtomicU16, Ordering};

use crate::{CAN_RX_DEBOUNCE_MS, MINP, MINP_COUNT};

const MAX_TRACKED_IDS: usize = 8;
const MAX_FRAME_BYTES: usize = 8;

static FRAME_IDS: [AtomicU16; MAX_TRACKED_IDS] = [const { AtomicU16::new(0) }; MAX_TRACKED_IDS];
static FRAME_LENS: [AtomicU8; MAX_TRACKED_IDS] = [const { AtomicU8::new(0) }; MAX_TRACKED_IDS];
static FRAME_DATA: [[AtomicU8; MAX_FRAME_BYTES]; MAX_TRACKED_IDS] =
    [const { [const { AtomicU8::new(0) }; MAX_FRAME_BYTES] }; MAX_TRACKED_IDS];

static TIME_MS: AtomicU32 = AtomicU32::new(0);
static MINP_LAST_RAW: [AtomicU8; MINP_COUNT] = [const { AtomicU8::new(0) }; MINP_COUNT];
static MINP_ZERO_SINCE: [AtomicU32; MINP_COUNT] = [const { AtomicU32::new(0) }; MINP_COUNT];
static MINP_ARMED: [AtomicU8; MINP_COUNT] = [const { AtomicU8::new(0) }; MINP_COUNT];
static MINP_PULSE: [AtomicU8; MINP_COUNT] = [const { AtomicU8::new(0) }; MINP_COUNT];
/// Set after the first real RX frame covers this minp bit (avoids startup false `0→1`).
static MINP_SYNCED: [AtomicU8; MINP_COUNT] = [const { AtomicU8::new(0) }; MINP_COUNT];

fn find_slot(id: u16) -> Option<usize> {
    for (i, slot) in FRAME_IDS.iter().enumerate() {
        if slot.load(Ordering::Relaxed) == id {
            return Some(i);
        }
    }
    None
}

fn alloc_slot(id: u16) -> Option<usize> {
    if let Some(slot) = find_slot(id) {
        return Some(slot);
    }
    for (i, slot) in FRAME_IDS.iter().enumerate() {
        if slot.load(Ordering::Relaxed) == 0 {
            slot.store(id, Ordering::Relaxed);
            return Some(i);
        }
    }
    None
}

fn raw_minp_bit(index: usize) -> Option<bool> {
    let entry = MINP.get(index)?;
    if entry.active_value == 0 {
        return Some(false);
    }
    if entry.byte_index as usize >= frame_len(entry.can_id) {
        return None;
    }
    Some(frame_bit(
        entry.can_id,
        entry.byte_index as usize,
        entry.bit_index,
    ))
}

fn poll_minp_edges() -> bool {
    let now = TIME_MS.load(Ordering::Relaxed);
    let debounce = CAN_RX_DEBOUNCE_MS;
    let mut pulsed = false;

    for i in 0..MINP_COUNT {
        let Some(raw) = raw_minp_bit(i) else {
            continue;
        };
        let raw = raw as u8;

        if MINP_SYNCED[i].load(Ordering::Relaxed) == 0 {
            MINP_SYNCED[i].store(1, Ordering::Relaxed);
            MINP_LAST_RAW[i].store(raw, Ordering::Relaxed);
            MINP_ZERO_SINCE[i].store(if raw == 0 { now } else { 0 }, Ordering::Relaxed);
            continue;
        }

        let prev_raw = MINP_LAST_RAW[i].load(Ordering::Relaxed);

        if raw == 0 {
            let since = MINP_ZERO_SINCE[i].load(Ordering::Relaxed);
            if since == 0 {
                MINP_ZERO_SINCE[i].store(now, Ordering::Relaxed);
            } else if debounce == 0 || now.saturating_sub(since) >= debounce as u32 {
                MINP_ARMED[i].store(1, Ordering::Relaxed);
            }
        } else {
            MINP_ZERO_SINCE[i].store(0, Ordering::Relaxed);
            if raw == 1 && prev_raw == 0 && MINP_ARMED[i].load(Ordering::Relaxed) != 0 {
                MINP_PULSE[i].store(1, Ordering::Relaxed);
                MINP_ARMED[i].store(0, Ordering::Relaxed);
                pulsed = true;
            }
        }

        MINP_LAST_RAW[i].store(raw, Ordering::Relaxed);
    }

    pulsed
}

/// Advance the debounce clock and evaluate minp edges.
/// Returns `true` when a minp one-shot pulse was raised.
pub fn advance_time_ms(now_ms: u32) -> bool {
    TIME_MS.store(now_ms, Ordering::Relaxed);
    poll_minp_edges()
}

fn frame_data_changed(slot: usize, data: &[u8]) -> bool {
    let len = data.len().min(MAX_FRAME_BYTES);
    let old_len = FRAME_LENS[slot].load(Ordering::Relaxed) as usize;
    if old_len != len {
        return true;
    }
    for (i, &byte) in data.iter().enumerate().take(len) {
        if FRAME_DATA[slot][i].load(Ordering::Relaxed) != byte {
            return true;
        }
    }
    false
}

/// Store one received CAN frame. Returns `true` if frame data or minp state changed.
pub fn store_rx(id: u16, data: &[u8]) -> bool {
    let Some(slot) = alloc_slot(id) else {
        return false;
    };
    let changed = frame_data_changed(slot, data);
    let len = data.len().min(MAX_FRAME_BYTES) as u8;
    FRAME_LENS[slot].store(len, Ordering::Relaxed);
    for (i, byte) in FRAME_DATA[slot].iter().enumerate().take(MAX_FRAME_BYTES) {
        byte.store(data.get(i).copied().unwrap_or(0), Ordering::Relaxed);
    }
    changed || poll_minp_edges()
}

pub fn frame_len(id: u16) -> usize {
    find_slot(id)
        .map(|slot| FRAME_LENS[slot].load(Ordering::Relaxed) as usize)
        .unwrap_or(0)
}

pub fn frame_byte(id: u16, index: usize) -> u8 {
    find_slot(id)
        .and_then(|slot| FRAME_DATA[slot].get(index))
        .map(|byte| byte.load(Ordering::Relaxed))
        .unwrap_or(0)
}

pub fn frame_bit(id: u16, byte_index: usize, bit_index: u8) -> bool {
    if bit_index >= 8 {
        return false;
    }
    (frame_byte(id, byte_index) >> bit_index) & 1 != 0
}

/// Current minp bit level from the last RX frame (sustained, not an edge).
pub fn minp_raw(index: usize) -> bool {
    raw_minp_bit(index).unwrap_or(false)
}

/// True while any minp one-shot pulse is pending (before `minp_in` consumes it).
pub fn any_minp_pending() -> bool {
    MINP_PULSE.iter().any(|pulse| pulse.load(Ordering::Relaxed) != 0)
}

/// One-shot minp pulse: `true` only on raw `0→1` after debounced `00`.
pub fn minp_active(index: usize) -> bool {
    MINP_PULSE
        .get(index)
        .map(|pulse| pulse.swap(0, Ordering::Relaxed) != 0)
        .unwrap_or(false)
}
