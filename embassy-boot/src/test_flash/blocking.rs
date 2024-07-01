use core::cell::RefCell;

use embassy_embedded_hal::flash::partition::BlockingPartition;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embedded_storage::nor_flash::NorFlash;

use crate::BootLoaderConfig;

pub struct BlockingTestFlash<ACTIVE, DFU, STATE>
where
    ACTIVE: NorFlash,
    DFU: NorFlash,
    STATE: NorFlash,
{
    active: Mutex<NoopRawMutex, RefCell<ACTIVE>>,
    dfu: Mutex<NoopRawMutex, RefCell<DFU>>,
    state: Mutex<NoopRawMutex, RefCell<STATE>>,
}

impl<ACTIVE, DFU, STATE> BlockingTestFlash<ACTIVE, DFU, STATE>
where
    ACTIVE: NorFlash,
    DFU: NorFlash,
    STATE: NorFlash,
{
    pub fn new(config: BootLoaderConfig<ACTIVE, DFU, STATE>) -> Self {
        Self {
            active: Mutex::new(RefCell::new(config.active)),
            dfu: Mutex::new(RefCell::new(config.dfu)),
            state: Mutex::new(RefCell::new(config.state)),
        }
    }

    pub fn active(&self) -> BlockingPartition<NoopRawMutex, ACTIVE> {
        Self::create_partition(&self.active)
    }

    pub fn dfu(&self) -> BlockingPartition<NoopRawMutex, DFU> {
        Self::create_partition(&self.dfu)
    }

    pub fn state(&self) -> BlockingPartition<NoopRawMutex, STATE> {
        Self::create_partition(&self.state)
    }

    pub fn create_partition<T: NorFlash>(
        mutex: &Mutex<NoopRawMutex, RefCell<T>>,
    ) -> BlockingPartition<NoopRawMutex, T> {
        BlockingPartition::new(mutex, 0, mutex.lock(|f| f.borrow().capacity()) as u32)
    }
}

impl<ACTIVE, DFU, STATE> BlockingTestFlash<ACTIVE, DFU, STATE>
where
    ACTIVE: NorFlash + embedded_storage_async::nor_flash::NorFlash,
    DFU: NorFlash + embedded_storage_async::nor_flash::NorFlash,
    STATE: NorFlash + embedded_storage_async::nor_flash::NorFlash,
{
    pub fn into_async(self) -> super::AsyncTestFlash<ACTIVE, DFU, STATE> {
        let config = BootLoaderConfig {
            active: self.active.into_inner().into_inner(),
            dfu: self.dfu.into_inner().into_inner(),
            state: self.state.into_inner().into_inner(),
        };
        super::AsyncTestFlash::new(config)
    }
}
