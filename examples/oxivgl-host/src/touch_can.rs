//! CAN press/hold/repeat for the SDL host hall demo (Linux SocketCAN).

use std::sync::{mpsc, OnceLock};
use std::thread;
use std::time::{Duration as StdDuration, Instant};

use log::{debug, info, warn};
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};
use touch_hall_common::button_status;
use touch_hall_common::can_bridge::{handle_minp_frame, payload_is_release, tx_payload};
use touch_hall_common::can_input;
use touch_hall_common::can_refresh::CAN_REFRESH_MS;
use touch_hall_common::can_scheduler;
use touch_hall_common::input_state;
use touch_hall_common::touch_feedback;
use touch_hall_common::touch_hold;
use touch_hall_common::rhai_state::Plc;
use touch_hall_common::{
    BUTTON_COUNT, BUTTON_TOKENS, CAN_CHANNEL, CAN_COMMAND_REPEAT_MS, CAN_ENABLED, CAN_RX_DEBOUNCE_MS,
    CAN_RX_POLL_MS, CAN_TX_ID, LONG_PRESS_MS, MINP_RX_ID, STATE_SCRIPT_ENABLED,
};

const MAX_BUTTONS: usize = 64;

enum Action {
    Press(u8),
    Release,
    CanRx,
}

static ACTION_TX: OnceLock<mpsc::Sender<Action>> = OnceLock::new();

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
    if let Some(active) = touch_feedback::hold_highlight(index) {
        return active;
    }
    button_status::plc_active(index)
}

fn send_plc_payload(socket: &CanSocket, payload: &[u8; 6], refresh: bool) -> bool {
    if payload_is_release(payload)
        && !refresh
        && (touch_hold::is_latched() || input_state::any_held())
    {
        return false;
    }
    write_frame(socket, payload, refresh)
}

fn send_plc_cmd(socket: &CanSocket, button_index: u8, refresh: bool) -> bool {
    send_plc_payload(socket, &tx_payload(button_index), refresh)
}

fn send_cycle_output(socket: &CanSocket, payload: Option<[u8; 6]>, refresh: bool) {
    if let Some(p) = payload {
        if !send_plc_payload(socket, &p, refresh) {
            warn!("CAN output failed data={:02x?}", p);
        }
    }
}

fn write_frame(socket: &CanSocket, payload: &[u8; 6], refresh: bool) -> bool {
    log_tx_payload_bytes(payload, refresh);
    let Some(id) = StandardId::new(CAN_TX_ID) else {
        return false;
    };
    let Some(frame) = CanFrame::new(id, payload) else {
        return false;
    };
    let ok = socket.write_frame(&frame).is_ok();
    if ok {
        touch_hall_common::can_refresh::note_tx_payload(payload);
    }
    ok
}

fn log_tx_payload_bytes(payload: &[u8; 6], refresh: bool) {
    if refresh {
        info!(
            "CAN TX id=0x{CAN_TX_ID:03x} refresh data={:02x?}",
            &payload[..payload.len().min(6)],
        );
    } else if payload_is_release(payload) {
        info!(
            "CAN TX id=0x{CAN_TX_ID:03x} release data={:02x?}",
            &payload[..payload.len().min(6)],
        );
    } else {
        info!(
            "CAN TX id=0x{CAN_TX_ID:03x} data={:02x?}",
            &payload[..payload.len().min(6)],
        );
    }
}

fn log_rx_frame(id: u16, data: &[u8]) {
    debug!("CAN RX id=0x{id:03x} len={} data={:02x?}", data.len(), data);
}

fn spawn_plc_tx_loop(rx: mpsc::Receiver<Action>, socket: CanSocket, mut plc: Plc) {
    let touch_tick = StdDuration::from_millis(CAN_COMMAND_REPEAT_MS);
    let refresh_every = StdDuration::from_millis(CAN_REFRESH_MS as u64);
    let mut scratch = [0u8; MAX_BUTTONS];
    let tx_state = can_scheduler::PlcTxState::new();

    loop {
        match rx.recv_timeout(refresh_every) {
            Ok(first) => match first {
                Action::CanRx => {
                    send_cycle_output(
                        &socket,
                        can_scheduler::on_can_rx(&mut plc, &mut scratch, &tx_state),
                        false,
                    );
                }
                Action::Press(index) => {
                    send_cycle_output(
                        &socket,
                        can_scheduler::on_touch_press(&mut plc, &mut scratch, &tx_state, index),
                        false,
                    );

                    let mut press_start = Instant::now();
                    let mut long_fired = false;
                    let mut held = index;
                    let mut next_refresh = Instant::now() + refresh_every;

                    loop {
                        let until_refresh = next_refresh.saturating_duration_since(Instant::now());
                        let wait = until_refresh.min(touch_tick);
                        let wait = if wait.is_zero() {
                            StdDuration::from_nanos(1)
                        } else {
                            wait
                        };

                        match rx.recv_timeout(wait) {
                            Ok(Action::CanRx) => {
                                send_cycle_output(
                                    &socket,
                                    can_scheduler::on_can_rx(&mut plc, &mut scratch, &tx_state),
                                    false,
                                );
                            }
                            Ok(Action::Release) => {
                                send_cycle_output(
                                    &socket,
                                    can_scheduler::on_touch_release(
                                        &mut plc,
                                        &mut scratch,
                                        &tx_state,
                                        held,
                                        long_fired,
                                    ),
                                    true,
                                );
                                break;
                            }
                            Ok(Action::Press(new_index)) => {
                                let prev = held;
                                held = new_index;
                                press_start = Instant::now();
                                long_fired = false;
                                send_cycle_output(
                                    &socket,
                                    can_scheduler::on_touch_switch(
                                        &mut plc,
                                        &mut scratch,
                                        &tx_state,
                                        prev,
                                        new_index,
                                    ),
                                    false,
                                );
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                if Instant::now() >= next_refresh {
                                    send_cycle_output(
                                        &socket,
                                        can_scheduler::on_periodic_refresh(
                                            &mut plc,
                                            &mut scratch,
                                            &tx_state,
                                        ),
                                        false,
                                    );
                                    next_refresh = Instant::now() + refresh_every;
                                } else {
                                    send_cycle_output(
                                        &socket,
                                        can_scheduler::on_touch_hold_tick(
                                            &mut plc,
                                            &mut scratch,
                                            &tx_state,
                                            held,
                                            press_start.elapsed().as_millis() as u64,
                                            &mut long_fired,
                                        ),
                                        false,
                                    );
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => return,
                        }
                    }
                }
                Action::Release => {
                    send_cycle_output(
                        &socket,
                        can_scheduler::on_idle_release(&mut plc, &mut scratch, &tx_state),
                        false,
                    );
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                send_cycle_output(
                    &socket,
                    can_scheduler::on_periodic_refresh(&mut plc, &mut scratch, &tx_state),
                    true,
                );
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn spawn_legacy_tx_loop(rx: mpsc::Receiver<Action>, socket: CanSocket) {
    let tick = StdDuration::from_millis(CAN_COMMAND_REPEAT_MS);

    while let Ok(first) = rx.recv() {
        match first {
            Action::CanRx => {}
            Action::Press(index) => {
                can_scheduler::touch_begin(index);
                let _ = send_plc_cmd(&socket, index, false);

                let mut press_start = Instant::now();
                let mut long_fired = false;
                let mut held = index;

                loop {
                    match rx.recv_timeout(tick) {
                        Ok(Action::CanRx) => {}
                        Ok(Action::Release) => {
                            can_scheduler::touch_end(held, long_fired);
                            let _ = send_plc_cmd(&socket, 255, false);
                            break;
                        }
                        Ok(Action::Press(new_index)) => {
                            let prev = held;
                            held = new_index;
                            can_scheduler::touch_switch(prev, new_index);
                            press_start = Instant::now();
                            long_fired = false;
                            let _ = send_plc_cmd(&socket, new_index, false);
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            can_scheduler::touch_maybe_long(
                                held,
                                press_start.elapsed().as_millis() as u64,
                                &mut long_fired,
                            );
                            let _ = send_plc_cmd(&socket, held, false);
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => return,
                    }
                }
            }
            Action::Release => {
                touch_hall_common::can_bridge::set_active_button(None);
                let _ = send_plc_cmd(&socket, 255, false);
            }
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
            "CAN TX on {CAN_CHANNEL}: id=0x{CAN_TX_ID:03x}, repeat={CAN_COMMAND_REPEAT_MS}ms, refresh={CAN_REFRESH_MS}ms idle, long_press={LONG_PRESS_MS}ms"
        );

        if STATE_SCRIPT_ENABLED {
            let Some(plc) = Plc::new() else {
                warn!("PLC script required but failed to load — TX disabled");
                return;
            };
            info!("PLC scan cycle active (fn cycle)");
            spawn_plc_tx_loop(rx, socket, plc);
        } else {
            spawn_legacy_tx_loop(rx, socket);
        }
    });
}

fn spawn_plc_rx_loop(socket: CanSocket, action_tx: mpsc::Sender<Action>) {
    let timeout = StdDuration::from_millis(CAN_RX_POLL_MS);
    let started = Instant::now();

    loop {
        let now_ms = started.elapsed().as_millis() as u32;
        if can_input::advance_time_ms(now_ms) {
            let _ = action_tx.send(Action::CanRx);
        }

        match socket.read_frame_timeout(timeout) {
            Ok(frame) => {
                let id = match frame.id() {
                    socketcan::Id::Standard(id) => id.as_raw(),
                    _ => continue,
                };
                let data = frame.data();
                log_rx_frame(id, data);
                if can_input::store_rx(id, data) {
                    let _ = action_tx.send(Action::CanRx);
                }
            }
            Err(_) => {}
        }
    }
}

fn spawn_legacy_rx_loop(socket: CanSocket) {
    let mut scratch = [0u8; MAX_BUTTONS];
    let timeout = StdDuration::from_millis(CAN_RX_POLL_MS);
    let started = Instant::now();

    loop {
        let now_ms = started.elapsed().as_millis() as u32;
        can_input::advance_time_ms(now_ms);

        match socket.read_frame_timeout(timeout) {
            Ok(frame) => {
                let id = match frame.id() {
                    socketcan::Id::Standard(id) => id.as_raw(),
                    _ => continue,
                };
                let data = frame.data();
                log_rx_frame(id, data);

                let mut before = [0u8; MAX_BUTTONS];
                button_status::snapshot(&mut before, BUTTON_COUNT);
                for (i, value) in scratch.iter_mut().enumerate().take(BUTTON_COUNT) {
                    *value = before[i];
                }

                if id == MINP_RX_ID {
                    handle_minp_frame(id, data, &mut scratch[..BUTTON_COUNT]);
                }

                can_scheduler::log_minp_scratch(&before[..BUTTON_COUNT], &scratch[..BUTTON_COUNT]);
                for (i, value) in scratch.iter().enumerate().take(BUTTON_COUNT) {
                    button_status::store(i, *value);
                }
            }
            Err(_) => {}
        }
    }
}

fn spawn_rx_thread(action_tx: mpsc::Sender<Action>) {
    thread::spawn(move || {
        let Ok(socket) = CanSocket::open(CAN_CHANNEL) else {
            return;
        };
        info!("CAN RX on {CAN_CHANNEL}: minp id=0x{MINP_RX_ID:03x}, debounce={CAN_RX_DEBOUNCE_MS}ms");

        if STATE_SCRIPT_ENABLED {
            info!("PLC scan cycle active (fn cycle)");
            spawn_plc_rx_loop(socket, action_tx);
        } else {
            spawn_legacy_rx_loop(socket);
        }
    });
}

/// Start background SocketCAN TX/RX threads when enabled in config.
pub fn start() {
    if !CAN_ENABLED {
        info!("CAN disabled in config — UI only");
        return;
    }

    let (action_tx, action_rx) = mpsc::channel();
    let _ = ACTION_TX.set(action_tx.clone());
    spawn_tx_thread(action_rx);
    spawn_rx_thread(action_tx);
}
