//! CAN press/hold/repeat tasks for the hall touch UI.

use core::sync::atomic::{AtomicU8, Ordering};

use defmt::{debug, info, warn};
use embassy_futures::select::{select, Either};
use embassy_stm32::can::{CanRx, CanTx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use touch_hall_common::can_bridge::{button_token, command_payload, handle_minp_frame, set_active_button};
use touch_hall_common::rhai_state::StateMachine;
use touch_hall_common::{BUTTON_COUNT, CAN_ENABLED, CAN_TX_ID, MINP_RX_ID, STATE_SCRIPT_ENABLED};

const MAX_BUTTONS: usize = 64;

pub enum Action {
    Press(u8),
    Release,
}

static ACTIONS: Channel<CriticalSectionRawMutex, Action, 8> = Channel::new();
static BUTTON_STATUS: [AtomicU8; MAX_BUTTONS] = [const { AtomicU8::new(0) }; MAX_BUTTONS];

pub fn on_button_press(index: u8) {
    let _ = ACTIONS.try_send(Action::Press(index));
}

pub fn on_button_release() {
    let _ = ACTIONS.try_send(Action::Release);
}

pub fn button_status(index: usize) -> bool {
    BUTTON_STATUS
        .get(index)
        .map(|s| s.load(Ordering::Relaxed) != 0)
        .unwrap_or(false)
}

async fn send_command(tx: &mut CanTx<'static>, button_index: u8, repeat: bool) -> bool {
    let payload = command_payload(button_index);
    log_tx_payload(button_index, &payload, repeat);
    let frame = embassy_stm32::can::frame::Frame::new_standard(CAN_TX_ID, &payload);
    match frame {
        Ok(frame) => tx.write(&frame).await.is_some(),
        Err(_) => false,
    }
}

async fn send_release(tx: &mut CanTx<'static>) -> bool {
    let payload = [0u8; 6];
    log_tx_payload(255, &payload, false);
    let frame = embassy_stm32::can::frame::Frame::new_standard(CAN_TX_ID, &payload);
    match frame {
        Ok(frame) => tx.write(&frame).await.is_some(),
        Err(_) => false,
    }
}

fn log_tx_payload(button_index: u8, payload: &[u8; 6], repeat: bool) {
    if repeat {
        debug!(
            "CAN TX id=0x{:03x} btn={} token={} repeat data={:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            CAN_TX_ID,
            button_index,
            button_token(button_index as usize),
            payload[0],
            payload[1],
            payload[2],
            payload[3],
            payload[4],
            payload[5],
        );
    } else if button_index == 255 {
        info!(
            "CAN TX id=0x{:03x} release data={:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            CAN_TX_ID, payload[0], payload[1], payload[2], payload[3], payload[4], payload[5],
        );
    } else {
        info!(
            "CAN TX id=0x{:03x} btn={} token={} data={:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            CAN_TX_ID,
            button_index,
            button_token(button_index as usize),
            payload[0],
            payload[1],
            payload[2],
            payload[3],
            payload[4],
            payload[5],
        );
    }
}

fn log_rx_frame(id: u16, data: &[u8]) {
    debug!(
        "CAN RX id=0x{:03x} len={} data={:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
        id,
        data.len(),
        data.first().copied().unwrap_or(0),
        data.get(1).copied().unwrap_or(0),
        data.get(2).copied().unwrap_or(0),
        data.get(3).copied().unwrap_or(0),
        data.get(4).copied().unwrap_or(0),
        data.get(5).copied().unwrap_or(0),
        data.get(6).copied().unwrap_or(0),
        data.get(7).copied().unwrap_or(0),
    );
}

fn log_minp_changes(before: &[u8], after: &[u8]) {
    for (i, (prev, next)) in before.iter().zip(after.iter()).enumerate() {
        if prev != next {
            info!(
                "CAN minp btn={} token={} active {} -> {}",
                i,
                button_token(i),
                *prev != 0,
                *next != 0,
            );
        }
    }
}

fn frame_standard_id(frame: &embassy_stm32::can::frame::Frame) -> Option<u16> {
    match frame.header().id() {
        embedded_can::Id::Standard(id) => Some(id.as_raw()),
        _ => None,
    }
}

#[embassy_executor::task]
pub async fn tx_task(tx: &'static mut CanTx<'static>) {
    if !CAN_ENABLED {
        info!("CAN disabled in config — UI only");
        loop {
            Timer::after_secs(60).await;
        }
    }

    for (i, token) in touch_hall_common::BUTTON_TOKENS.iter().enumerate() {
        info!("button[{}] = {}", i, token);
    }
    info!(
        "CAN TX: id=0x{:03x}, repeat={}ms",
        CAN_TX_ID,
        touch_hall_common::CAN_COMMAND_REPEAT_MS,
    );

    let repeat = Duration::from_millis(touch_hall_common::CAN_COMMAND_REPEAT_MS);

    loop {
        match ACTIONS.receive().await {
            Action::Press(index) => {
                set_active_button(Some(index));
                if !send_command(tx, index, false).await {
                    warn!("CAN send_command failed for button {}", index);
                }

                let mut held = index;
                loop {
                    match select(ACTIONS.receive(), Timer::after(repeat)).await {
                        Either::First(Action::Release) => {
                            set_active_button(None);
                            if !send_release(tx).await {
                                warn!("CAN send_release failed");
                            }
                            break;
                        }
                        Either::First(Action::Press(new_index)) => {
                            held = new_index;
                            set_active_button(Some(new_index));
                            if !send_command(tx, new_index, false).await {
                                warn!("CAN send_command failed for button {}", new_index);
                            }
                        }
                        Either::Second(()) => {
                            if !send_command(tx, held, true).await {
                                warn!("CAN repeat failed for button {}", held);
                            }
                        }
                    }
                }
            }
            Action::Release => {
                set_active_button(None);
                if !send_release(tx).await {
                    warn!("CAN send_release failed");
                }
            }
        }
    }
}

#[embassy_executor::task]
pub async fn rx_task(rx: &'static mut CanRx<'static>) {
    if !CAN_ENABLED {
        loop {
            Timer::after_secs(60).await;
        }
    }

    info!("CAN RX: minp id=0x{:03x}", MINP_RX_ID);
    if STATE_SCRIPT_ENABLED {
        info!("Rhai state script enabled for button feedback");
    }

    let mut state_machine = StateMachine::new();
    let mut scratch = [0u8; MAX_BUTTONS];

    loop {
        if let Ok(envelope) = rx.read().await {
            let (frame, _) = envelope.parts();
            if let Some(id) = frame_standard_id(&frame) {
                let data = frame.data();
                log_rx_frame(id, data);
                let mut before = [0u8; MAX_BUTTONS];
                for (i, atom) in BUTTON_STATUS.iter().enumerate().take(BUTTON_COUNT) {
                    before[i] = atom.load(Ordering::Relaxed);
                    scratch[i] = before[i];
                }

                if let Some(machine) = state_machine.as_mut() {
                    machine.on_can_rx(id, data);
                    machine.eval_button_status(&mut scratch[..BUTTON_COUNT]);
                } else if id == MINP_RX_ID {
                    handle_minp_frame(id, data, &mut scratch[..BUTTON_COUNT]);
                }

                log_minp_changes(&before[..BUTTON_COUNT], &scratch[..BUTTON_COUNT]);
                for (i, value) in scratch.iter().enumerate().take(BUTTON_COUNT) {
                    BUTTON_STATUS[i].store(*value, Ordering::Relaxed);
                }
            }
        }
    }
}
