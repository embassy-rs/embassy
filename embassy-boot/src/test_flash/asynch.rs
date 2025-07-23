use embassy_embedded_hal::flash::partition::Partition;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage_async::nor_flash::NorFlash;

use crate::BootLoaderConfig;

pub struct AsyncTestFlash<ACTIVE, DFU, STATE, SAFE>
where
    ACTIVE: NorFlash,
    DFU: NorFlash,
    STATE: NorFlash,
    SAFE: NorFlash,
{
    active: Mutex<NoopRawMutex, ACTIVE>,
    dfu: Mutex<NoopRawMutex, DFU>,
    state: Mutex<NoopRawMutex, STATE>,
    safe: Mutex<NoopRawMutex, SAFE>,
}

impl<ACTIVE, DFU, STATE, SAFE> AsyncTestFlash<ACTIVE, DFU, STATE, SAFE>
where
    ACTIVE: NorFlash,
    DFU: NorFlash,
    STATE: NorFlash,
    SAFE: NorFlash,
{
    pub fn new(config: BootLoaderConfig<ACTIVE, DFU, STATE, SAFE>) -> Self {
        Self {
            active: Mutex::new(config.active),
            dfu: Mutex::new(config.dfu),
            state: Mutex::new(config.state),
            safe: Mutex::new(config.safe),
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

    pub fn safe(&self) -> Partition<NoopRawMutex, SAFE> {
        Self::create_partition(&self.safe)
    }

    fn create_partition<T: NorFlash>(mutex: &Mutex<NoopRawMutex, T>) -> Partition<NoopRawMutex, T> {
        Partition::new(mutex, 0, unwrap!(mutex.try_lock()).capacity() as u32)
    }
}

impl<ACTIVE, DFU, STATE, SAFE> AsyncTestFlash<ACTIVE, DFU, STATE, SAFE>
where
    ACTIVE: NorFlash + embedded_storage::nor_flash::NorFlash,
    DFU: NorFlash + embedded_storage::nor_flash::NorFlash,
    STATE: NorFlash + embedded_storage::nor_flash::NorFlash,
    SAFE: NorFlash + embedded_storage::nor_flash::NorFlash,
{
    pub fn into_blocking(self) -> super::BlockingTestFlash<ACTIVE, DFU, STATE, SAFE> {
        let config = BootLoaderConfig {
            active: self.active.into_inner(),
            dfu: self.dfu.into_inner(),
            state: self.state.into_inner(),
            safe: self.safe.into_inner(),
        };
        super::BlockingTestFlash::new(config)
    }
}
