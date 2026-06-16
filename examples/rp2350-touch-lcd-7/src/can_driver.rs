//! Shared XL2515 access from TX/RX Embassy tasks.

use core::cell::RefCell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::xl2515::CanSpi;

static CAN: Mutex<CriticalSectionRawMutex, RefCell<Option<CanSpi>>> =
    Mutex::new(RefCell::new(None));

pub async fn install(mut can: CanSpi, bitrate: u32) {
    can.init(bitrate).await;
    CAN.lock(|cell| {
        *cell.borrow_mut() = Some(can);
    });
}

pub fn with_can<R>(f: impl FnOnce(&mut CanSpi) -> R) -> Option<R> {
    CAN.lock(|cell| cell.borrow_mut().as_mut().map(f))
}
