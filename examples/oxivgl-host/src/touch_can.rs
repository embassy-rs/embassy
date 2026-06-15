//! CAN press/hold/repeat for the SDL host hall demo (Linux SocketCAN).

use core::sync::atomic::{AtomicU8, Ordering};
use std::sync::{mpsc, OnceLock};
use std::thread;
use std::time::Duration as StdDuration;

use log::{debug, info, warn};
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};
use touch_hall_common::can_bridge::{button_token, command_payload, handle_minp_frame, set_active_button};
use touch_hall_common::rhai_state::StateMachine;
use touch_hall_common::{BUTTON_COUNT, BUTTON_TOKENS, CAN_CHANNEL, CAN_COMMAND_REPEAT_MS, CAN_ENABLED, CAN_RECV_TIMEOUT_MS, CAN_TX_ID, MINP_RX_ID, STATE_SCRIPT_ENABLED};

const MAX_BUTTONS: usize = 64;

enum Action {
    Press(u8),
    Release,
}

static ACTION_TX: OnceLock<mpsc::Sender<Action>> = OnceLock::new();
static BUTTON_STATUS: [AtomicU8; MAX_BUTTONS] = [const { AtomicU8::new(0) }; MAX_BUTTONS];

pub fn on_button_press(index: u8) {
    if let Some(tx) = ACTION_TX.get() {
        let _ = tx.send(Action::Press(index));
    }
}

pub fn on_button_release() {
    if let Some(tx) = ACTION_TX.get() {
        let _ = tx.send(Action::Release);
    }
}

pub fn button_status(index: usize) -> bool {
    BUTTON_STATUS
        .get(index)
        .map(|s| s.load(Ordering::Relaxed) != 0)
        .unwrap_or(false)
}

fn write_frame(socket: &CanSocket, button_index: u8, payload: &[u8], repeat: bool) -> bool {
    log_tx_payload(button_index, payload, repeat);
    let Some(id) = StandardId::new(CAN_TX_ID) else {
        return false;
    };
    let Some(frame) = CanFrame::new(id, payload) else {
        return false;
    };
    socket.write_frame(&frame).is_ok()
}

fn log_tx_payload(button_index: u8, payload: &[u8], repeat: bool) {
    if repeat {
        debug!(
            "CAN TX id=0x{CAN_TX_ID:03x} btn={button_index} token={} repeat data={:02x?}",
            button_token(button_index as usize),
            &payload[..payload.len().min(6)],
        );
    } else if button_index == 255 {
        info!(
            "CAN TX id=0x{CAN_TX_ID:03x} release data={:02x?}",
            &payload[..payload.len().min(6)],
        );
    } else {
        info!(
            "CAN TX id=0x{CAN_TX_ID:03x} btn={button_index} token={} data={:02x?}",
            button_token(button_index as usize),
            &payload[..payload.len().min(6)],
        );
    }
}

fn log_rx_frame(id: u16, data: &[u8]) {
    debug!("CAN RX id=0x{id:03x} len={} data={:02x?}", data.len(), data);
}

fn log_minp_changes(before: &[u8], after: &[u8]) {
    for (i, (prev, next)) in before.iter().zip(after.iter()).enumerate() {
        if prev != next {
            info!(
                "CAN minp btn={i} token={} active {} -> {}",
                button_token(i),
                *prev != 0,
                *next != 0,
            );
        }
    }
}

fn spawn_tx_thread(rx: mpsc::Receiver<Action>) {
    thread::spawn(move || {
        let Ok(socket) = CanSocket::open(CAN_CHANNEL) else {
            warn!("failed to open CAN channel {CAN_CHANNEL}");
            return;
        };

        for (i, token) in BUTTON_TOKENS.iter().enumerate() {
            info!("button[{i}] = {token}");
        }
        info!(
            "CAN TX on {CAN_CHANNEL}: id=0x{CAN_TX_ID:03x}, repeat={CAN_COMMAND_REPEAT_MS}ms"
        );

        let repeat = StdDuration::from_millis(CAN_COMMAND_REPEAT_MS);

        while let Ok(first) = rx.recv() {
            match first {
                Action::Press(index) => {
                    set_active_button(Some(index));
                    if !write_frame(&socket, index, &command_payload(index), false) {
                        warn!("CAN send_command failed for button {index}");
                    }

                    let mut held = index;
                    loop {
                        match rx.recv_timeout(repeat) {
                            Ok(Action::Release) => {
                                set_active_button(None);
                                if !write_frame(&socket, 255, &[0u8; 6], false) {
                                    warn!("CAN send_release failed");
                                }
                                break;
                            }
                            Ok(Action::Press(new_index)) => {
                                held = new_index;
                                set_active_button(Some(new_index));
                                if !write_frame(&socket, new_index, &command_payload(new_index), false) {
                                    warn!("CAN send_command failed for button {new_index}");
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                if !write_frame(&socket, held, &command_payload(held), true) {
                                    warn!("CAN repeat failed for button {held}");
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => return,
                        }
                    }
                }
                Action::Release => {
                    set_active_button(None);
                    let _ = write_frame(&socket, 255, &[0u8; 6], false);
                }
            }
        }
    });
}

fn spawn_rx_thread() {
    thread::spawn(move || {
        let Ok(socket) = CanSocket::open(CAN_CHANNEL) else {
            return;
        };
        info!("CAN RX on {CAN_CHANNEL}: minp id=0x{MINP_RX_ID:03x}");
        if STATE_SCRIPT_ENABLED {
            info!("Rhai state script enabled for button feedback");
        }

        let mut state_machine = StateMachine::new();
        let mut scratch = [0u8; MAX_BUTTONS];
        let timeout = StdDuration::from_millis(CAN_RECV_TIMEOUT_MS);

        loop {
            match socket.read_frame_timeout(timeout) {
                Ok(frame) => {
                    let id = match frame.id() {
                        socketcan::Id::Standard(id) => id.as_raw(),
                        _ => continue,
                    };
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
                Err(_) => {}
            }
        }
    });
}

/// Start background SocketCAN TX/RX threads when enabled in config.
pub fn start() {
    if !CAN_ENABLED {
        info!("CAN disabled in config — UI only");
        return;
    }

    let (tx, rx) = mpsc::channel();
    let _ = ACTION_TX.set(tx);
    spawn_tx_thread(rx);
    spawn_rx_thread();
}
