//! CAN press/hold/repeat tasks for the hall touch UI.

use core::sync::atomic::{AtomicU8, Ordering};

use defmt::{info, warn};
use embassy_futures::select::{select, Either};
use embassy_stm32::can::{CanRx, CanTx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use crate::can_bridge::{
    self, frame_standard_id, handle_minp_frame, send_command, send_release, set_active_button,
};
use crate::touch_config::{self, BUTTON_COUNT, CAN_ENABLED, MINP_RX_ID};

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

#[embassy_executor::task]
pub async fn tx_task(tx: &'static mut CanTx<'static>) {
    if !CAN_ENABLED {
        info!("CAN disabled in config — UI only");
        loop {
            Timer::after_secs(60).await;
        }
    }

    can_bridge::log_button_order();
    info!(
        "CAN TX: id=0x{:03x}, repeat={}ms",
        touch_config::CAN_TX_ID,
        touch_config::CAN_COMMAND_REPEAT_MS,
    );

    let repeat = Duration::from_millis(touch_config::CAN_COMMAND_REPEAT_MS);

    loop {
        match ACTIONS.receive().await {
            Action::Press(index) => {
                set_active_button(Some(index));
                if !send_command(tx, index).await {
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
                            if !send_command(tx, new_index).await {
                                warn!("CAN send_command failed for button {}", new_index);
                            }
                        }
                        Either::Second(()) => {
                            if !send_command(tx, held).await {
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

    let mut scratch = [0u8; MAX_BUTTONS];

    loop {
        if let Ok(envelope) = rx.read().await {
            let (frame, _) = envelope.parts();
            if let Some(id) = frame_standard_id(&frame) {
                if id == MINP_RX_ID {
                    for (i, atom) in BUTTON_STATUS.iter().enumerate().take(BUTTON_COUNT) {
                        scratch[i] = atom.load(Ordering::Relaxed);
                    }
                    handle_minp_frame(id, frame.data(), &mut scratch[..BUTTON_COUNT]);
                    for (i, value) in scratch.iter().enumerate().take(BUTTON_COUNT) {
                        BUTTON_STATUS[i].store(*value, Ordering::Relaxed);
                    }
                }
            }
        }
    }
}
