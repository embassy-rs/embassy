//! Async USB host stack.
//!
//! This module provides USB host enumeration, descriptor parsing, and class driver support on top of
//! the [`embassy_usb_driver::host`] hardware traits.

#![allow(async_fn_in_trait)]

pub mod control;
pub mod descriptor;
pub mod handler;

use core::cell::RefCell;
use core::marker::PhantomData;

use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex as AsyncMutex;
use embassy_usb_driver::host::{
    AddressSet, DeviceEvent, HostError, PipeError, UsbHostAllocator, UsbHostController, UsbPipe, pipe,
};
pub use embassy_usb_driver::host::{SplitInfo, SplitSpeed};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType, Speed};

use crate::host::control::{ControlPipeExt, SetupPacket};
use crate::host::descriptor::{ConfigurationDescriptor, DeviceDescriptor, USBDescriptor};
pub use crate::host::handler::BusRoute;
use crate::host::handler::EnumerationInfo;

/// USB host enumeration error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EnumerationError {
    /// Transfer failed during enumeration.
    Transfer(PipeError),
    /// Invalid or unexpected descriptor received.
    InvalidDescriptor,
    /// Configuration buffer too small
    ConfigBufferTooSmall(usize),
    /// No free pipe for EP0 or no free device address.
    NoPipe,
    /// The device did not respond to a control request after retries.
    RequestFailed,
}

impl From<PipeError> for EnumerationError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl From<HostError> for EnumerationError {
    fn from(e: HostError) -> Self {
        match e {
            HostError::PipeError(e) => Self::Transfer(e),
            HostError::InvalidDescriptor => Self::InvalidDescriptor,
            HostError::RequestFailed => Self::RequestFailed,
            _ => Self::NoPipe,
        }
    }
}

impl core::fmt::Display for EnumerationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_e) => write!(f, "Transfer error during enumeration"),
            Self::InvalidDescriptor => write!(f, "Invalid descriptor"),
            Self::ConfigBufferTooSmall(size) => {
                write!(f, "Configuration buffer too small: device requires {} bytes", size)
            }
            Self::NoPipe => write!(f, "No free pipe or no free device address"),
            Self::RequestFailed => write!(f, "Device did not respond"),
        }
    }
}

impl core::error::Error for EnumerationError {}

/// Shared bus-wide state used by a [`BusHandle`].
///
/// Holds the in-use USB device addresses (1–127) and an async mutex used to
/// serialise enumerations on a single bus.
///
/// Addresses greater than 127 are not issued because their behavior is not specified (USB 2.0 §9.4.6).
pub struct BusState {
    devices: BlockingMutex<CriticalSectionRawMutex, RefCell<DeviceState>>,
    enum_lock: AsyncMutex<CriticalSectionRawMutex, ()>,
}

struct DeviceState {
    /// Per address, `None` when free, else the hub it hangs off (`0` for the root port).
    parent: [Option<u8>; ADDRESS_COUNT as usize],
    /// Addresses remain allocated until their removal has been reported.
    removing: AddressSet,
}

impl DeviceState {
    fn set_parent(&mut self, addr: u8, hub: u8) {
        if hub < ADDRESS_COUNT && self.parent.get(addr as usize).is_some_and(|parent| parent.is_some()) {
            self.parent[addr as usize] = Some(hub);
            if self.removing.contains(hub) {
                self.removing.insert(addr);
            }
        }
    }

    fn device_tree(&self, addr: u8) -> Option<AddressSet> {
        if addr != 0 && self.parent.get(addr as usize).copied().flatten().is_none() {
            return None;
        }

        let mut addresses = AddressSet::default();
        addresses.insert(addr);
        while let Some(child) = (1..ADDRESS_COUNT)
            .find(|&a| !addresses.contains(a) && self.parent[a as usize].is_some_and(|p| addresses.contains(p)))
        {
            addresses.insert(child);
        }
        Some(addresses)
    }
}

/// Number of USB device addresses, 0 to 127. Address 0 is the default address (USB 2.0 §9.4.6).
const ADDRESS_COUNT: u8 = 128;

impl BusState {
    /// Create new, empty bus state.
    pub const fn new() -> Self {
        Self {
            devices: BlockingMutex::new(RefCell::new(DeviceState {
                parent: [None; ADDRESS_COUNT as usize],
                removing: AddressSet::new(),
            })),
            enum_lock: AsyncMutex::new(()),
        }
    }

    /// Allocate the next free device address (1–127),
    /// marking it as in use.
    ///
    /// Returns `None` when every address in the 1–127 range is already
    /// taken.
    fn alloc_address(&self) -> Option<u8> {
        self.devices.lock(|d| {
            let mut d = d.borrow_mut();
            let addr = (1..ADDRESS_COUNT).find(|&addr| d.parent[addr as usize].is_none())?;
            d.parent[addr as usize] = Some(0);
            Some(addr)
        })
    }

    /// Record that device `addr` hangs off hub `hub`.
    pub(crate) fn set_parent(&self, addr: u8, hub: u8) {
        self.devices.lock(|d| d.borrow_mut().set_parent(addr, hub));
    }

    /// Release an address that has no devices behind it.
    pub(crate) fn free_address(&self, addr: u8) {
        self.devices.lock(|d| {
            let mut d = d.borrow_mut();
            if !d.removing.contains(addr) {
                if let Some(slot) = d.parent.get_mut(addr as usize) {
                    *slot = None;
                }
            }
        });
    }

    /// Remove `addr` and its descendants, or all devices for address 0.
    ///
    /// A concurrent removal is queued for the caller already notifying the allocator.
    /// Its addresses remain allocated until that caller finishes.
    pub(crate) fn remove_address<'d>(&self, alloc: &impl UsbHostAllocator<'d>, addr: u8) {
        let busy = self.devices.lock(|d| {
            let mut d = d.borrow_mut();
            let Some(removed) = d.device_tree(addr) else {
                return true;
            };
            let busy = !d.removing.is_empty();
            d.removing.union(removed);
            busy
        });

        if busy {
            // `removed` has been added to `devices` and will be picked up
            // by notify loop already running
            return;
        }

        // Loops here to ensure that queued removals are processed
        // this avoids calling `device_removed` within a CS
        let mut notified = AddressSet::default();
        loop {
            let pending = self.devices.lock(|d| {
                let mut d = d.borrow_mut();
                let next = d.removing.exclude(notified);
                if next.is_empty() {
                    for addr in d.removing {
                        d.parent[addr as usize] = None;
                    }
                    d.removing = AddressSet::new();
                }
                next
            });
            if pending.is_empty() {
                return;
            }
            alloc.device_removed(pending);
            notified.union(pending);
        }
    }
}

impl Default for BusState {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds a device address for the duration of an enumeration, releasing
/// it again unless the enumeration succeeds.
///
/// `enumerate` can fail at a dozen points, and every one of them has to
/// hand the address back or the bus leaks addresses until it runs out of
/// them at 127. Doing that at each `return` is a standing invitation to
/// miss one — several were missed — so ownership expresses it instead:
/// the address is freed on drop, and only a successful enumeration takes
/// it back out with [`AddressGuard::release`].
struct AddressGuard<'a> {
    state: &'a BusState,
    addr: u8,
    /// Whether dropping still frees the address. Cleared by
    /// [`AddressGuard::release`] once the device owns the address.
    armed: bool,
}

impl<'a> AddressGuard<'a> {
    fn new(state: &'a BusState, addr: u8) -> Self {
        Self {
            state,
            addr,
            armed: true,
        }
    }

    /// The address this guard is holding.
    fn addr(&self) -> u8 {
        self.addr
    }

    /// Gives up ownership: the device now holds this address, so it must
    /// not be returned to the pool.
    fn release(&mut self) -> u8 {
        self.armed = false;
        self.addr
    }
}

impl Drop for AddressGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.state.free_address(self.addr);
        }
    }
}

/// Bus-level controller for a single root USB controller.
///
/// Owns the [`UsbHostController`] implementation and exposes the
/// bus-wide operations that must be serialised against each other
/// (root-port event waiting, bus reset).
///
/// Pipe allocation and device enumeration live on the companion
/// [`BusHandle`] returned alongside a `BusController` by the [`bus`]
/// constructor.
pub struct BusController<'d, C: UsbHostController<'d>> {
    driver: C,
    state: &'d BusState,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, C: UsbHostController<'d>> BusController<'d, C> {
    /// Get a reference to the underlying controller.
    pub fn controller(&self) -> &C {
        &self.driver
    }

    /// Get a mutable reference to the underlying controller.
    pub fn controller_mut(&mut self) -> &mut C {
        &mut self.driver
    }

    /// Wait for a root-port attach/detach.
    ///
    /// On attach, the implementation drives a bus reset to completion
    /// before returning and reports the speed the device settled on.
    ///
    /// On detach, removal of every device address is requested. If another
    /// removal is being reported, those addresses stay allocated until it finishes.
    pub async fn wait_for_device_event(&mut self) -> DeviceEvent {
        let event = self.driver.wait_for_device_event().await;
        if event == DeviceEvent::Disconnected {
            self.state.remove_address(&self.driver.allocator(), 0);
        }
        event
    }

    /// Wait for a device to connect on the root port.
    ///
    /// Issues a bus reset internally and returns the detected speed.
    /// Spurious disconnects, overcurrent events, and other non-attach
    /// events are silently absorbed.
    pub async fn wait_for_connection(&mut self) -> Speed {
        loop {
            match self.wait_for_device_event().await {
                DeviceEvent::Connected(speed) => {
                    info!("USB device connected, speed: {:?}", speed);
                    return speed;
                }
                DeviceEvent::Disconnected => continue,
                _ => continue,
            }
        }
    }
}

/// Shareable handle for pipe allocation and device enumeration.
///
/// A `BusHandle` bundles a [`UsbHostAllocator`] produced by a
/// [`UsbHostController`] with a reference to the bus-wide
/// [`BusState`].
///
/// `BusHandle` itself implements [`UsbHostAllocator`] by forwarding to
/// its inner allocator, so it can be passed directly to class driver
/// constructors.
#[derive(Clone)]
pub struct BusHandle<'d, A: UsbHostAllocator<'d>> {
    alloc: A,
    state: &'d BusState,
}

impl<'d, A: UsbHostAllocator<'d>> BusHandle<'d, A> {
    /// Borrow the shared bus state.
    pub fn state(&self) -> &'d BusState {
        self.state
    }

    /// Release a previously allocated device address.
    ///
    /// Also releases its descendants. A concurrent removal may queue the
    /// notification and delay release until its callback finishes.
    pub fn free_address(&self, addr: u8) {
        if addr != 0 {
            self.state.remove_address(&self.alloc, addr);
        }
    }

    /// Enumerate a connected device.
    ///
    /// Performs the standard enumeration sequence:
    /// 1. Get device descriptor (first 8 bytes) to learn EP0 max packet size
    /// 2. SET_ADDRESS to assign a unique address
    /// 3. Get full device descriptor
    /// 4. Get configuration descriptor
    /// 5. SET_CONFIGURATION
    ///
    /// `route` describes how the device is reached on the bus (directly
    /// at its native speed, or via split transactions / legacy `PRE`
    /// through a hub's transaction translator).
    ///
    /// Enumerations are serialised bus-wide through the
    /// [`BusState`]'s enumeration mutex.
    ///
    /// # Preconditions
    ///
    /// The caller must have placed the device into the default
    /// (address 0) state *before* calling this method. For a root-port
    /// device that means an upstream bus reset has completed; for a
    /// hub-attached device, the parent hub's port reset must have
    /// completed and [`BusRoute::Translated`] must carry the
    /// appropriate [`SplitInfo`].
    ///
    /// Returns the [`EnumerationInfo`] for the device and the number of
    /// bytes written to `config_buf`.
    ///
    /// [`SplitInfo`]: embassy_usb_driver::host::SplitInfo
    pub async fn enumerate(
        &self,
        route: BusRoute,
        config_buf: &mut [u8],
    ) -> Result<(EnumerationInfo, usize), EnumerationError> {
        use embassy_time::Timer;

        use crate::host::descriptor::DeviceDescriptorPartial;

        // Serialise enumerations against other concurrent callers on
        // the same bus: the default (address 0) state is bus-global.
        let _enum_guard = self.state.enum_lock.lock().await;

        let mut addr = AddressGuard::new(self.state, self.state.alloc_address().ok_or(EnumerationError::NoPipe)?);

        // use smallest size "8", since some devices use lower than default for given speed.
        const DEFAULT_MAX_PACKET_SIZE: u16 = 8;

        let ep0_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: DEFAULT_MAX_PACKET_SIZE,
            interval_ms: 0,
        };

        let mut ch = self
            .alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(0, &ep0_info, route.split())
            .map_err(|_| EnumerationError::NoPipe)?;

        trace!("[enum] Getting max_packet_size for new device");
        let max_packet_size0 = {
            let mut max_retries = 10;
            loop {
                match ch
                    .request_descriptor::<DeviceDescriptorPartial, { DeviceDescriptorPartial::BUF_SIZE }>(0, false)
                    .await
                {
                    Ok(desc) => break desc.max_packet_size0,
                    Err(e) => {
                        warn!("Request descriptor error: {:?}, retries: {}", e, max_retries);
                        if max_retries > 0 {
                            max_retries -= 1;
                            Timer::after_millis(1).await;
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            }
        };
        // USB 2.0 §9.6.1: legal EP0 max packet sizes are 8, 16, 32, 64.
        if !matches!(max_packet_size0, 8 | 16 | 32 | 64) {
            return Err(EnumerationError::InvalidDescriptor);
        }

        ch.device_set_address(addr.addr()).await?;
        // USB 2.0 §9.2.6.3: allow the device a 2ms recovery interval after SET_ADDRESS.
        Timer::after_millis(2).await;

        // From this point on, the device will answer to the assigned address, even if
        // enumeration fails. Take the address and release the guard.
        let assigned_addr = addr.release();

        // Drop pipe to re-allocate with new address and correct max_packet_size.
        drop(ch);

        let ep0_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: max_packet_size0 as u16,
            interval_ms: 0,
        };

        let mut ch = self
            .alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(assigned_addr, &ep0_info, route.split())
            .map_err(|_| EnumerationError::NoPipe)?;

        // Retried on any error, not only on a timeout as this read used
        // to be. A device flaky enough to STALL a descriptor read is the
        // case the retry exists for, and a stall took the `v => return v`
        // arm straight out of the loop — so the read with the largest
        // retry budget in the function was also the one that gave up
        // first on the most likely failure.
        let dev_desc = crate::host::handler::retry_descriptor(async || {
            ch.request_descriptor::<DeviceDescriptor, { DeviceDescriptor::BUF_SIZE }>(0, false)
                .await
        })
        .await?;

        info!(
            "Device: VID={:04x} PID={:04x} class={:02x}",
            dev_desc.vendor_id, dev_desc.product_id, dev_desc.device_class
        );

        // Step 4: Get configuration descriptor header (9 bytes).
        let setup = SetupPacket::get_config_descriptor(0, 9);
        let n = crate::host::handler::retry_descriptor(async || {
            ch.control_in(&setup.to_bytes(), &mut config_buf[..9]).await
        })
        .await?;

        if n < 9 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let config_header = ConfigurationDescriptor::try_from_bytes(&config_buf[..9])
            .map_err(|_| EnumerationError::InvalidDescriptor)?;
        let total_len = config_header.total_len as usize;

        if total_len > config_buf.len() {
            return Err(EnumerationError::ConfigBufferTooSmall(total_len));
        }

        // Get full configuration descriptor.
        let setup = SetupPacket::get_config_descriptor(0, total_len as u16);
        let n = crate::host::handler::retry_descriptor(async || {
            ch.control_in(&setup.to_bytes(), &mut config_buf[..total_len]).await
        })
        .await?;

        // USB 2.0 §9.4.3: the device must return exactly total_len bytes for a full config descriptor.
        if n != total_len {
            return Err(EnumerationError::InvalidDescriptor);
        }

        trace!("Config descriptor: {} bytes", n);

        // Step 5: SET_CONFIGURATION.
        let setup = SetupPacket::set_configuration(config_header.configuration_value);
        ch.control_out(&setup.to_bytes(), &[]).await?;

        info!("Device configured (config={})", config_header.configuration_value);

        // Pipe is released on drop.
        drop(ch);

        Ok((
            EnumerationInfo {
                device_address: assigned_addr,
                route,
                device_desc: dev_desc,
            },
            n,
        ))
    }
}

impl<'d, A: UsbHostAllocator<'d>> UsbHostAllocator<'d> for BusHandle<'d, A> {
    type Pipe<T: pipe::Type, D: pipe::Direction> = A::Pipe<T, D>;

    fn alloc_pipe<T: pipe::Type, D: pipe::Direction>(
        &self,
        addr: u8,
        endpoint: &EndpointInfo,
        split: Option<SplitInfo>,
    ) -> Result<Self::Pipe<T, D>, HostError> {
        self.alloc.alloc_pipe::<T, D>(addr, endpoint, split)
    }
}

/// Split a [`UsbHostController`] into a bus controller / bus handle pair.
///
/// The returned [`BusController`] drives root-port events and bus
/// resets. The [`BusHandle`] owns pipe allocation and device enumeration
/// and can be freely shared, handed to class drivers, hub handlers, or
/// other concurrent tasks while the controller task is blocked inside
/// [`wait_for_device_event`](BusController::wait_for_device_event).
pub fn bus<'d, C: UsbHostController<'d>>(
    driver: C,
    state: &'d BusState,
) -> (BusController<'d, C>, BusHandle<'d, C::Allocator>) {
    let alloc = driver.allocator();
    (
        BusController {
            driver,
            state,
            _phantom: PhantomData,
        },
        BusHandle { alloc, state },
    )
}

#[cfg(test)]
mod tests {
    use embassy_usb_driver::host::TimeoutConfig;

    use super::*;

    /// Allocator that records the set passed to `device_removed`.
    #[derive(Clone)]
    struct Recorder<'a>(&'a core::cell::Cell<AddressSet>, Option<&'a dyn Fn(AddressSet)>);

    enum NoPipe {}

    impl<T: pipe::Type, D: pipe::Direction> UsbPipe<T, D> for NoPipe {
        async fn control_in(&mut self, _: &[u8; 8], _: &mut [u8]) -> Result<usize, PipeError>
        where
            T: pipe::IsControl,
            D: pipe::IsIn,
        {
            match *self {}
        }

        async fn control_out(&mut self, _: &[u8; 8], _: &[u8]) -> Result<(), PipeError>
        where
            T: pipe::IsControl,
            D: pipe::IsOut,
        {
            match *self {}
        }

        async fn request_in(&mut self, _: &mut [u8]) -> Result<usize, PipeError>
        where
            D: pipe::IsIn,
        {
            match *self {}
        }

        async fn request_out(&mut self, _: &[u8], _: bool) -> Result<(), PipeError>
        where
            D: pipe::IsOut,
        {
            match *self {}
        }

        fn set_timeout(&mut self, _: TimeoutConfig)
        where
            T: pipe::IsControl,
        {
            match *self {}
        }

        fn reset_data_toggle(&mut self)
        where
            T: pipe::IsBulkOrInterrupt,
        {
            match *self {}
        }
    }

    impl<'a> UsbHostAllocator<'a> for Recorder<'a> {
        type Pipe<T: pipe::Type, D: pipe::Direction> = NoPipe;

        fn alloc_pipe<T: pipe::Type, D: pipe::Direction>(
            &self,
            _: u8,
            _: &EndpointInfo,
            _: Option<SplitInfo>,
        ) -> Result<NoPipe, HostError> {
            Err(HostError::OutOfPipes)
        }

        fn device_removed(&self, addrs: AddressSet) {
            self.0.set(addrs);
            if let Some(on_removed) = self.1 {
                on_removed(addrs);
            }
        }
    }

    #[test]
    fn freeing_a_hub_frees_the_devices_behind_it() {
        let state = BusState::new();
        let hub = state.alloc_address().unwrap();
        let child = state.alloc_address().unwrap();
        let grandchild = state.alloc_address().unwrap();
        let other = state.alloc_address().unwrap();
        state.set_parent(child, hub);
        state.set_parent(grandchild, child);

        state.remove_address(&Recorder(&Default::default(), None), hub);

        // The unrelated device keeps its address, and the freed ones are free again.
        assert_eq!(state.alloc_address(), Some(hub));
        assert_eq!(state.alloc_address(), Some(child));
        assert_eq!(state.alloc_address(), Some(grandchild));
        assert_eq!(state.alloc_address(), Some(other + 1));
    }

    #[test]
    fn freeing_an_unused_address_does_nothing() {
        let state = BusState::new();
        let addr = state.alloc_address().unwrap();

        let seen = core::cell::Cell::new(AddressSet::default());

        let alloc = Recorder(&seen, None);
        state.remove_address(&alloc, addr + 1);
        state.remove_address(&alloc, 200);

        assert!(seen.get().into_iter().next().is_none());
        assert_eq!(state.alloc_address(), Some(addr + 1));
    }

    #[test]
    fn removing_notifies_the_removed_addresses() {
        let state = BusState::new();
        let hub = state.alloc_address().unwrap();
        let child = state.alloc_address().unwrap();
        let other = state.alloc_address().unwrap();
        state.set_parent(child, hub);
        let notified = core::cell::Cell::new(AddressSet::default());
        let alloc = Recorder(&notified, None);

        state.remove_address(&alloc, hub);
        assert!(notified.get().into_iter().eq([hub, child]));

        state.remove_address(&alloc, 0);
        assert!(notified.get().into_iter().eq([0, other]));
        assert_eq!(state.alloc_address(), Some(1));
    }

    #[test]
    fn removal_callback_can_queue_another_removal() {
        let state = BusState::new();
        let hub = state.alloc_address().unwrap();
        let child = state.alloc_address().unwrap();
        let other = state.alloc_address().unwrap();
        state.set_parent(child, hub);

        let seen = core::cell::Cell::new(AddressSet::default());
        let calls = core::cell::Cell::new(0);
        let on_removed = |addrs: AddressSet| {
            calls.set(calls.get() + 1);
            if !addrs.contains(0) {
                assert!(addrs.into_iter().eq([hub, child]));
                state.free_address(hub);
                assert_eq!(state.alloc_address(), Some(other + 1));
                state.remove_address(&Recorder(&seen, None), 0);
            }
        };
        let alloc = Recorder(&seen, Some(&on_removed));
        state.remove_address(&alloc, hub);

        assert_eq!(calls.get(), 2);
        assert!(seen.get().into_iter().eq([0, other, other + 1]));
        assert_eq!(state.alloc_address(), Some(hub));
    }
}
