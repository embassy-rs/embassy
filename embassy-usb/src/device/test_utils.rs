//! Shared mocks for device-side tests.
#![allow(missing_docs)]

extern crate std;

use core::cell::{Cell, RefCell};
use core::future::{pending, poll_fn};
use core::task::{Poll, Waker};
use std::collections::VecDeque;
use std::rc::Rc;
use std::vec::Vec;

use crate::driver::{
    Bus, Direction, Driver, Endpoint, EndpointAddress, EndpointAllocError, EndpointConfig, EndpointDisabledError,
    EndpointError, EndpointIn, EndpointInfo, EndpointInterval, EndpointOut, EndpointType, Event, IsoEndpointConfig,
    IsoEndpointIn, IsoEndpointOut, IsoInSlot, IsoWriteError, Unsupported,
};

#[derive(Default)]
pub struct TestDriver {
    bus: TestBus,
}

pub struct TestEndpoint {
    info: EndpointInfo,
    stalled: Rc<Cell<bool>>,
    io: TestIo,
}

impl<'d> Driver<'d> for TestDriver {
    type EndpointOut = TestEndpoint;
    type EndpointIn = TestEndpoint;
    type IsoEndpointOut = TestEndpoint;
    type IsoEndpointIn = TestEndpoint;
    type Bus = TestBus;

    fn alloc_endpoint_in(&mut self, config: &EndpointConfig) -> Result<TestEndpoint, EndpointAllocError> {
        let addr = config.addr.unwrap_or_else(|| {
            let index = (1..9).find(|&i| self.bus.endpoints[i * 2 + 1].is_none()).unwrap();
            EndpointAddress::from_parts(index, Direction::In)
        });
        assert!(addr.is_in());
        self.bus.allocate(addr, *config)
    }

    fn alloc_endpoint_out(&mut self, config: &EndpointConfig) -> Result<TestEndpoint, EndpointAllocError> {
        let addr = config.addr.unwrap_or_else(|| {
            let index = (1..9).find(|&i| self.bus.endpoints[i * 2].is_none()).unwrap();
            EndpointAddress::from_parts(index, Direction::Out)
        });
        assert!(addr.is_out());
        self.bus.allocate(addr, *config)
    }

    fn alloc_iso_endpoint_out(&mut self, config: &IsoEndpointConfig) -> Result<TestEndpoint, EndpointAllocError> {
        self.alloc_endpoint_out(config)
    }

    fn alloc_iso_endpoint_in(&mut self, config: &IsoEndpointConfig) -> Result<TestEndpoint, EndpointAllocError> {
        self.alloc_endpoint_in(config)
    }

    fn start(mut self, control_max_packet_size: u16) -> TestBus {
        self.bus.started_with = control_max_packet_size;
        self.bus
    }
}

impl Endpoint for TestEndpoint {
    fn info(&self) -> EndpointInfo {
        self.info
    }

    async fn wait_enabled(&mut self) {
        self.io.wait_enabled(self.info.addr).await
    }

    fn is_enabled(&self) -> bool {
        self.io.is_enabled(self.info.addr)
    }

    async fn set_stalled(&mut self, stalled: bool) {
        self.stalled.set(stalled);
        self.io.stall(self.info.addr, stalled);
    }

    fn is_stalled(&self) -> bool {
        self.stalled.get()
    }
}

impl EndpointIn for TestEndpoint {
    async fn write_transfer(&mut self, buf: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        self.io.write(self.info.addr, buf, needs_zlp)
    }
}

impl EndpointOut for TestEndpoint {
    async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.io.read(self.info.addr, buf)
    }
}

pub struct TestSlot<'a>(&'a TestEndpoint);

impl IsoInSlot for TestSlot<'_> {
    fn write(self, data: &[u8]) -> Result<(), IsoWriteError> {
        self.0
            .io
            .write(self.0.info.addr, data, false)
            .map_err(|_| IsoWriteError::TooLate)
    }
}

impl IsoEndpointIn for TestEndpoint {
    type Slot<'a> = TestSlot<'a>;

    async fn ready(&mut self) -> Result<(Option<core::num::NonZeroU32>, TestSlot<'_>), EndpointDisabledError> {
        if !self.io.is_enabled(self.info.addr) {
            return Err(EndpointDisabledError);
        }
        Ok((None, TestSlot(self)))
    }
}

impl IsoEndpointOut for TestEndpoint {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.io.read(self.info.addr, buf)
    }
}

pub struct TestBus {
    /// The response to the last control request.
    pub(crate) response: Option<Response>,
    /// The control endpoint size passed to `Driver::start`.
    pub(crate) started_with: u16,
    /// The data stage of the next control OUT request.
    pub(crate) out_data: [u8; 8],
    /// Events that `poll()` returns, before it waits forever.
    pub(crate) events: std::collections::VecDeque<Event>,
    pub(crate) endpoints: [Option<EndpointInfo>; 32],
    pub(crate) configs: [Option<EndpointConfig>; 32],
    /// Shared with the endpoint handles, so a class stall is visible on the bus.
    pub(crate) stalled: [Rc<Cell<bool>>; 32],
    pub(crate) halt_calls: usize,
    pub(crate) io: TestIo,
}

impl Default for TestBus {
    fn default() -> Self {
        Self {
            response: None,
            started_with: 0,
            out_data: [0; 8],
            events: Default::default(),
            endpoints: [None; 32],
            configs: [None; 32],
            stalled: core::array::from_fn(|_| Rc::new(Cell::new(false))),
            halt_calls: 0,
            io: TestIo::default(),
        }
    }
}

impl TestBus {
    pub(crate) fn slot(addr: EndpointAddress) -> usize {
        // Model a controller with nine endpoint register pairs, including EP0.
        assert!(addr.index() < 9);
        addr.index() * 2 + usize::from(addr.is_in())
    }

    pub(crate) fn allocate(
        &mut self,
        addr: EndpointAddress,
        config: EndpointConfig,
    ) -> Result<TestEndpoint, EndpointAllocError> {
        let info = {
            let mut info = EndpointInfo::new(addr, config.ep_type, config.max_packet_size, config.interval);
            info.max_burst = config.max_burst;
            info.additional_transactions = config.additional_transactions;
            info
        };
        let slot = Self::slot(addr);
        assert!(self.endpoints[slot].replace(info).is_none());
        self.configs[slot] = Some(config);
        self.io.set_enabled(addr, false);
        Ok(TestEndpoint {
            info,
            stalled: self.stalled[slot].clone(),
            io: self.io.clone(),
        })
    }

    fn check_halt(&mut self, addr: EndpointAddress) -> usize {
        let slot = Self::slot(addr);
        let endpoint = self.endpoints[slot].unwrap();
        assert!(self.io.is_enabled(addr));
        assert!(matches!(endpoint.ep_type, EndpointType::Bulk | EndpointType::Interrupt));
        self.halt_calls += 1;
        slot
    }
}

impl Bus for TestBus {
    async fn enable(&mut self) {}

    async fn disable(&mut self) {
        for info in self.endpoints.iter().flatten() {
            self.io.set_enabled(info.addr, false);
        }
    }

    async fn poll(&mut self) -> Event {
        match self.events.pop_front() {
            Some(event) => event,
            None => pending().await,
        }
    }

    fn control_max_packet_size(&self) -> usize {
        64
    }

    async fn control_data_out(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        // Drivers wait for a data packet here, so a request without a data stage must not call it.
        assert!(
            !buf.is_empty(),
            "control_data_out called for a request without a data stage"
        );
        buf.fill(0);
        let len = buf.len().min(self.out_data.len());
        buf[..len].copy_from_slice(&self.out_data[..len]);
        Ok(buf.len())
    }

    async fn control_data_in(&mut self, data: &[u8], _needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        let status = match data {
            [b] => u16::from(*b),
            [lo, hi] => u16::from_le_bytes([*lo, *hi]),
            _ => panic!("control IN test response must be 1 or 2 bytes, got {}", data.len()),
        };
        self.response = Some(Response::Status(status));
        Ok(())
    }

    async fn control_accept(&mut self) {
        self.response = Some(Response::Accepted);
    }

    async fn control_reject(&mut self) {
        self.response = Some(Response::Rejected);
    }

    async fn control_accept_set_address(&mut self, _addr: u8) {
        self.control_accept().await;
    }

    async fn endpoint_set_enabled(&mut self, addr: EndpointAddress, enabled: bool) {
        let slot = Self::slot(addr);
        assert!(self.endpoints[slot].is_some());
        self.io.set_enabled(addr, enabled);
    }

    fn endpoint_is_enabled(&mut self, addr: EndpointAddress) -> bool {
        let slot = Self::slot(addr);
        self.endpoints[slot].is_some() && self.io.is_enabled(addr)
    }

    async fn endpoint_set_stalled(&mut self, addr: EndpointAddress, stalled: bool) {
        let slot = self.check_halt(addr);
        self.stalled[slot].set(stalled);
    }

    fn endpoint_is_stalled(&mut self, addr: EndpointAddress) -> bool {
        let slot = self.check_halt(addr);
        self.stalled[slot].get()
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    Accepted,
    Status(u16),
    Rejected,
}

impl TestDriver {
    /// Shared endpoint state.
    pub fn io(&self) -> TestIo {
        self.bus.io.clone()
    }

    /// Marks `addr` as allocated.
    pub fn reserve(mut self, addr: u8) -> Self {
        let addr = EndpointAddress::from(addr);
        let info = EndpointInfo::new(addr, EndpointType::Bulk, 0, EndpointInterval::NONE);
        assert!(self.bus.endpoints[TestBus::slot(addr)].replace(info).is_none());
        self
    }
}

impl TestEndpoint {
    /// A standalone endpoint.
    pub fn new(io: &TestIo, addr: u8, ep_type: EndpointType, max_packet_size: u16) -> Self {
        Self {
            info: EndpointInfo::new(addr.into(), ep_type, max_packet_size, EndpointInterval::NONE),
            stalled: Rc::default(),
            io: io.clone(),
        }
    }
}

/// Scripted OUT reads, logged IN writes and stalls, per endpoint.
#[derive(Clone, Default)]
pub struct TestIo(Rc<TestIoInner>);

#[derive(Default)]
struct TestIoInner {
    reads: RefCell<VecDeque<(EndpointAddress, Result<Vec<u8>, EndpointError>)>>,
    read_count: Cell<usize>,
    writes: RefCell<Vec<(EndpointAddress, Vec<u8>, bool)>>,
    stalls: RefCell<Vec<(EndpointAddress, bool)>>,
    disabled: RefCell<Vec<EndpointAddress>>,
    enable_wakers: RefCell<Vec<Waker>>,
}

impl TestIo {
    /// Queues one OUT transfer on `addr`.
    pub fn push_read(&self, addr: u8, data: &[u8]) {
        self.0.reads.borrow_mut().push_back((addr.into(), Ok(data.to_vec())));
    }

    /// Queues an OUT read error on `addr`.
    pub fn push_read_error(&self, addr: u8, error: EndpointError) {
        self.0.reads.borrow_mut().push_back((addr.into(), Err(error)));
    }

    /// Number of OUT reads so far.
    pub fn read_count(&self) -> usize {
        self.0.read_count.get()
    }

    /// IN transfers written to `addr`, with their `needs_zlp`.
    pub fn writes(&self, addr: u8) -> Vec<(Vec<u8>, bool)> {
        let addr = EndpointAddress::from(addr);
        let writes = self.0.writes.borrow();
        writes
            .iter()
            .filter(|w| w.0 == addr)
            .map(|w| (w.1.clone(), w.2))
            .collect()
    }

    /// Data of the IN transfers written to `addr`.
    pub fn data(&self, addr: u8) -> Vec<Vec<u8>> {
        self.writes(addr).into_iter().map(|(data, _)| data).collect()
    }

    /// Endpoints start enabled.
    pub fn set_enabled(&self, addr: impl Into<EndpointAddress>, enabled: bool) {
        let addr = addr.into();
        let mut disabled = self.0.disabled.borrow_mut();
        disabled.retain(|a| *a != addr);
        if !enabled {
            disabled.push(addr);
        }
        drop(disabled);
        for waker in self.0.enable_wakers.take() {
            waker.wake();
        }
    }

    pub fn is_enabled(&self, addr: impl Into<EndpointAddress>) -> bool {
        !self.0.disabled.borrow().contains(&addr.into())
    }

    /// Number of `set_stalled` calls on `addr`.
    pub fn stall_calls(&self, addr: u8) -> usize {
        let addr = EndpointAddress::from(addr);
        self.0.stalls.borrow().iter().filter(|s| s.0 == addr).count()
    }

    /// Last `set_stalled` value on `addr`.
    pub fn is_stalled(&self, addr: u8) -> bool {
        let addr = EndpointAddress::from(addr);
        self.0
            .stalls
            .borrow()
            .iter()
            .rev()
            .find(|s| s.0 == addr)
            .is_some_and(|s| s.1)
    }

    async fn wait_enabled(&self, addr: EndpointAddress) {
        poll_fn(|cx| {
            if self.is_enabled(addr) {
                return Poll::Ready(());
            }
            let mut wakers = self.0.enable_wakers.borrow_mut();
            if !wakers.iter().any(|w| w.will_wake(cx.waker())) {
                wakers.push(cx.waker().clone());
            }
            Poll::Pending
        })
        .await
    }

    fn read(&self, addr: EndpointAddress, buf: &mut [u8]) -> Result<usize, EndpointError> {
        if !self.is_enabled(addr) {
            return Err(EndpointError::Disabled);
        }
        let io = &self.0;
        io.read_count.set(io.read_count.get() + 1);
        let mut reads = io.reads.borrow_mut();
        let i = reads.iter().position(|r| r.0 == addr).ok_or(EndpointError::Disabled)?;
        let data = reads.remove(i).unwrap().1?;
        let dst = buf.get_mut(..data.len()).ok_or(EndpointError::BufferOverflow)?;
        dst.copy_from_slice(&data);
        Ok(data.len())
    }

    fn write(&self, addr: EndpointAddress, data: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        if !self.is_enabled(addr) {
            return Err(EndpointDisabledError);
        }
        self.0.writes.borrow_mut().push((addr, data.to_vec(), needs_zlp));
        Ok(())
    }

    fn stall(&self, addr: EndpointAddress, stalled: bool) {
        self.0.stalls.borrow_mut().push((addr, stalled));
    }
}
