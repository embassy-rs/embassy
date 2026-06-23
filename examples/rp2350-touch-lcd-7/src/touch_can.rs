//! CAN press/hold/repeat tasks for the hall touch UI (XL2515 backend).

use defmt::{debug, info, warn};
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use touch_hall_common::button_status;
use touch_hall_common::can_bridge::{active_button, handle_minp_frame, payload_is_release, tx_payload};
use touch_hall_common::can_input;
use touch_hall_common::can_refresh::CAN_REFRESH_MS;
use touch_hall_common::can_scheduler;
use touch_hall_common::input_state;
use touch_hall_common::touch_feedback;
use touch_hall_common::touch_hold;
use touch_hall_common::rhai_state::Plc;
use touch_hall_common::{
    BUTTON_COUNT, CAN_BAUD, CAN_COMMAND_REPEAT_MS, CAN_ENABLED, CAN_RX_DEBOUNCE_MS, CAN_RX_POLL_MS,
    CAN_TX_ID, LONG_PRESS_MS, MINP_RX_ID, STATE_SCRIPT_ENABLED,
};

use crate::can_driver;

const MAX_BUTTONS: usize = 64;
const MIN_HOLD_BEFORE_RELEASE_MS: u64 = 120;
const RELEASE_CONFIRM_MS: u64 = 120;

enum Action {
    Press(u8),
    Release,
    CanRx,
}

static ACTIONS: Channel<CriticalSectionRawMutex, Action, 8> = Channel::new();

pub fn on_button_press(index: u8) {
    let _ = ACTIONS.try_send(Action::Press(index));
}

pub fn on_button_release() {
    let _ = ACTIONS.try_send(Action::Release);
}

pub fn button_status(index: usize) -> bool {
    if let Some(active) = touch_feedback::hold_highlight(index) {
        return active;
    }
    button_status::plc_active(index)
}

fn send_plc_payload(payload: &[u8; 6], refresh: bool) -> bool {
    if payload_is_release(payload)
        && !refresh
        && (touch_hold::is_latched() || input_state::any_held() || active_button().is_some())
    {
        return false;
    }
    log_tx_payload_bytes(payload, refresh);
    let ok = can_driver::with_can(|can| {
        can.send_standard(CAN_TX_ID, payload);
    })
    .is_some();
    if ok {
        touch_hall_common::can_refresh::note_tx_payload(payload);
    }
    ok
}

fn send_plc_cmd(button_index: u8, refresh: bool) -> bool {
    send_plc_payload(&tx_payload(button_index), refresh)
}

fn send_cycle_output(payload: Option<[u8; 6]>, refresh: bool) {
    if let Some(p) = payload {
        if !send_plc_payload(&p, refresh) {
            warn!("CAN output failed");
        }
    }
}

fn drain_pending_releases() {
    loop {
        match ACTIONS.try_receive() {
            Ok(Action::Release) => {}
            Ok(Action::CanRx) => {
                let _ = ACTIONS.try_send(Action::CanRx);
                break;
            }
            Ok(other @ Action::Press(_)) => {
                let _ = ACTIONS.try_send(other);
                break;
            }
            Err(_) => break,
        }
    }
}

/// After a release TX, wait briefly for finger resume (slide to another button).
async fn wait_touch_resume() -> Option<u8> {
    let confirm_until = Instant::now() + Duration::from_millis(RELEASE_CONFIRM_MS);
    while Instant::now() < confirm_until {
        let remaining = confirm_until
            .checked_duration_since(Instant::now())
            .unwrap_or(Duration::from_millis(0));
        if remaining == Duration::from_millis(0) {
            break;
        }

        match select(ACTIONS.receive(), Timer::after(remaining)).await {
            Either::First(Action::Press(index)) => return Some(index),
            Either::First(Action::CanRx) => {
                let _ = ACTIONS.try_send(Action::CanRx);
            }
            Either::First(Action::Release) => {}
            Either::Second(()) => break,
        }
    }
    None
}

fn log_tx_payload_bytes(payload: &[u8; 6], refresh: bool) {
    if refresh {
        info!(
            "CAN TX id=0x{:03x} refresh data={:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            CAN_TX_ID,
            payload[0],
            payload[1],
            payload[2],
            payload[3],
            payload[4],
            payload[5],
        );
    } else if payload_is_release(payload) {
        info!(
            "CAN TX id=0x{:03x} release data={:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            CAN_TX_ID, payload[0], payload[1], payload[2], payload[3], payload[4], payload[5],
        );
    } else {
        info!(
            "CAN TX id=0x{:03x} data={:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            CAN_TX_ID,
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

fn poll_can_rx() -> bool {
    let Some((id, len, data)) = can_driver::with_can(|can| can.try_receive()).flatten() else {
        return false;
    };
    let data = &data[..len as usize];
    log_rx_frame(id, data);
    can_input::store_rx(id, data)
}

async fn plc_tx_loop(mut plc: Plc) {
    let touch_tick = Duration::from_millis(CAN_COMMAND_REPEAT_MS);
    let refresh_every = Duration::from_millis(CAN_REFRESH_MS as u64);
    let mut scratch = [0u8; MAX_BUTTONS];
    let tx_state = can_scheduler::PlcTxState::new();

    loop {
        match select(ACTIONS.receive(), Timer::after(refresh_every)).await {
            Either::Second(()) => {
                send_cycle_output(
                    can_scheduler::on_periodic_refresh(&mut plc, &mut scratch, &tx_state),
                    true,
                );
            }
            Either::First(first) => match first {
                Action::CanRx => {
                    send_cycle_output(can_scheduler::on_can_rx(&mut plc, &mut scratch, &tx_state), false);
                }
                Action::Press(index) => {
                    drain_pending_releases();
                    send_cycle_output(
                        can_scheduler::on_touch_press(&mut plc, &mut scratch, &tx_state, index),
                        false,
                    );

                    let mut press_start = Instant::now();
                    let mut long_fired = false;
                    let mut held = index;
                    let mut next_refresh = Instant::now() + refresh_every;

                    loop {
                        let until_refresh = next_refresh.saturating_duration_since(Instant::now());
                        let wait = if until_refresh < touch_tick {
                            until_refresh
                        } else {
                            touch_tick
                        };
                        let wait = if wait < Duration::from_millis(1) {
                            Duration::from_millis(1)
                        } else {
                            wait
                        };

                        match select(ACTIONS.receive(), Timer::after(wait)).await {
                            Either::First(Action::CanRx) => {
                                send_cycle_output(
                                    can_scheduler::on_can_rx(&mut plc, &mut scratch, &tx_state),
                                    false,
                                );
                            }
                            Either::First(Action::Release) => {
                                if press_start.elapsed()
                                    < Duration::from_millis(MIN_HOLD_BEFORE_RELEASE_MS)
                                {
                                    continue;
                                }
                                send_cycle_output(
                                    can_scheduler::on_touch_release(
                                        &mut plc,
                                        &mut scratch,
                                        &tx_state,
                                        held,
                                        long_fired,
                                    ),
                                    true,
                                );
                                if let Some(new_index) = wait_touch_resume().await {
                                    let prev = held;
                                    held = new_index;
                                    press_start = Instant::now();
                                    long_fired = false;
                                    send_cycle_output(
                                        can_scheduler::on_touch_switch(
                                            &mut plc,
                                            &mut scratch,
                                            &tx_state,
                                            prev,
                                            new_index,
                                        ),
                                        false,
                                    );
                                } else {
                                    break;
                                }
                            }
                            Either::First(Action::Press(new_index)) => {
                                let prev = held;
                                held = new_index;
                                press_start = Instant::now();
                                long_fired = false;
                                send_cycle_output(
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
                            Either::Second(()) => {
                                if Instant::now() >= next_refresh {
                                    send_cycle_output(
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
                        }
                    }
                }
                Action::Release => {
                    send_cycle_output(
                        can_scheduler::on_idle_release(&mut plc, &mut scratch, &tx_state),
                        false,
                    );
                }
            },
        }
    }
}

async fn legacy_tx_loop() {
    let tick = Duration::from_millis(CAN_COMMAND_REPEAT_MS);
    let refresh_every = Duration::from_millis(CAN_REFRESH_MS as u64);

    loop {
        match select(ACTIONS.receive(), Timer::after(refresh_every)).await {
            Either::Second(()) => {
                if let Some(payload) = touch_hall_common::can_refresh::idle_refresh_payload() {
                    let _ = send_plc_payload(&payload, true);
                }
            }
            Either::First(first) => match first {
                Action::CanRx => {}
                Action::Press(index) => {
                    drain_pending_releases();
                    can_scheduler::touch_begin(index);
                    let _ = send_plc_cmd(index, false);

                    let mut press_start = Instant::now();
                    let mut long_fired = false;
                    let mut held = index;

                    loop {
                        match select(ACTIONS.receive(), Timer::after(tick)).await {
                            Either::First(Action::CanRx) => {}
                            Either::First(Action::Release) => {
                                if press_start.elapsed()
                                    < Duration::from_millis(MIN_HOLD_BEFORE_RELEASE_MS)
                                {
                                    continue;
                                }
                                can_scheduler::touch_end(held, long_fired);
                                let _ = send_plc_cmd(255, true);
                                if let Some(new_index) = wait_touch_resume().await {
                                    let prev = held;
                                    held = new_index;
                                    can_scheduler::touch_switch(prev, new_index);
                                    press_start = Instant::now();
                                    long_fired = false;
                                    let _ = send_plc_cmd(new_index, false);
                                } else {
                                    break;
                                }
                            }
                            Either::First(Action::Press(new_index)) => {
                                let prev = held;
                                held = new_index;
                                can_scheduler::touch_switch(prev, new_index);
                                press_start = Instant::now();
                                long_fired = false;
                                let _ = send_plc_cmd(new_index, false);
                            }
                            Either::Second(()) => {
                                can_scheduler::touch_maybe_long(
                                    held,
                                    press_start.elapsed().as_millis() as u64,
                                    &mut long_fired,
                                );
                                let _ = send_plc_cmd(held, false);
                            }
                        }
                    }
                }
                Action::Release => {
                    if touch_hold::is_latched() {
                        continue;
                    }
                    touch_hall_common::can_bridge::set_active_button(None);
                    let _ = send_plc_cmd(255, false);
                }
            },
        }
    }
}

#[embassy_executor::task]
pub async fn tx_task() {
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
        "CAN TX: id=0x{:03x}, {} bit/s, repeat={}ms, refresh={}ms idle (00×6), long_press={}ms",
        CAN_TX_ID, CAN_BAUD, CAN_COMMAND_REPEAT_MS, CAN_REFRESH_MS, LONG_PRESS_MS,
    );

    if STATE_SCRIPT_ENABLED {
        let Some(plc) = Plc::new() else {
            warn!("PLC script required but failed to load — TX disabled");
            loop {
                Timer::after_secs(60).await;
            }
        };
        info!("PLC scan cycle active (fn cycle)");
        plc_tx_loop(plc).await;
    } else {
        legacy_tx_loop().await;
    }
}

async fn plc_rx_loop() {
    let poll = Duration::from_millis(CAN_RX_POLL_MS);

    loop {
        if can_input::advance_time_ms(Instant::now().as_millis() as u32) {
            let _ = ACTIONS.try_send(Action::CanRx);
        }

        match select(Timer::after(poll), async {
            if poll_can_rx() {
                let _ = ACTIONS.try_send(Action::CanRx);
            }
        })
        .await
        {
            Either::First(()) => {
                if poll_can_rx() {
                    let _ = ACTIONS.try_send(Action::CanRx);
                }
            }
            Either::Second(()) => {}
        }
    }
}

async fn legacy_rx_loop() {
    let mut scratch = [0u8; MAX_BUTTONS];
    let poll = Duration::from_millis(CAN_RX_POLL_MS);

    loop {
        can_input::advance_time_ms(Instant::now().as_millis() as u32);
        Timer::after(poll).await;

        let Some((id, len, data)) =
            can_driver::with_can(|can| can.try_receive()).flatten()
        else {
            continue;
        };
        let data = &data[..len as usize];
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
}

#[embassy_executor::task]
pub async fn rx_task() {
    if !CAN_ENABLED {
        loop {
            Timer::after_secs(60).await;
        }
    }

    info!("CAN RX: minp id=0x{:03x}, debounce={}ms", MINP_RX_ID, CAN_RX_DEBOUNCE_MS);

    if STATE_SCRIPT_ENABLED {
        plc_rx_loop().await;
    } else {
        legacy_rx_loop().await;
    }
}
