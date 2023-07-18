use embassy_embedded_hal::flash::partition::Partition;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage_async::nor_flash::NorFlash;

use crate::BootLoaderConfig;

pub struct AsyncTestFlash<ACTIVE, DFU, STATE>
where
    ACTIVE: NorFlash,
    DFU: NorFlash,
    STATE: NorFlash,
{
    active: Mutex<NoopRawMutex, ACTIVE>,
    dfu: Mutex<NoopRawMutex, DFU>,
    state: Mutex<NoopRawMutex, STATE>,
}

impl<ACTIVE, DFU, STATE> AsyncTestFlash<ACTIVE, DFU, STATE>
where
    ACTIVE: NorFlash,
    DFU: NorFlash,
    STATE: NorFlash,
{
    pub fn new(config: BootLoaderConfig<ACTIVE, DFU, STATE>) -> Self {
        Self {
            active: Mutex::new(config.active),
            dfu: Mutex::new(config.dfu),
            state: Mutex::new(config.state),
        }
    }

    pub fn active(&self) -> Partition<NoopRawMutex, ACTIVE> {
        Self::create_partition(&self.active)
    }

    pub fn dfu(&self) -> Partition<NoopRawMutex, DFU> {
        Self::create_partition(&self.dfu)
    }

    pub fn state(&self) -> Partition<NoopRawMutex, STATE> {
        Self::create_partition(&self.state)
    }

    fn create_partition<T: NorFlash>(mutex: &Mutex<NoopRawMutex, T>) -> Partition<NoopRawMutex, T> {
        Partition::new(mutex, 0, mutex.try_lock().unwrap().capacity() as u32)
    }
}

impl<ACTIVE, DFU, STATE> AsyncTestFlash<ACTIVE, DFU, STATE>
where
    ACTIVE: NorFlash + embedded_storage::nor_flash::NorFlash,
    DFU: NorFlash + embedded_storage::nor_flash::NorFlash,
    STATE: NorFlash + embedded_storage::nor_flash::NorFlash,
{
    pub fn into_blocking(self) -> super::BlockingTestFlash<ACTIVE, DFU, STATE> {
        let config = BootLoaderConfig {
            active: self.active.into_inner(),
            dfu: self.dfu.into_inner(),
            state: self.state.into_inner(),
        };
        super::BlockingTestFlash::new(config)
    }
}
