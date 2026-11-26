//! Shared mocks for device-side tests.

extern crate std;

use core::cell::Cell;
use core::future::pending;
use std::rc::Rc;

use crate::driver::{
    Bus, Direction, Driver, Endpoint, EndpointAddress, EndpointAllocError, EndpointConfig, EndpointDisabledError,
    EndpointError, EndpointIn, EndpointInfo, EndpointOut, EndpointType, Event, Unsupported,
};

#[derive(Default)]
pub(crate) struct TestDriver {
    bus: TestBus,
}

pub(crate) struct TestEndpoint {
    info: EndpointInfo,
    stalled: Rc<Cell<bool>>,
}

impl<'d> Driver<'d> for TestDriver {
    type EndpointOut = TestEndpoint;
    type EndpointIn = TestEndpoint;
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
        unreachable!()
    }

    fn is_enabled(&self) -> bool {
        true
    }

    async fn set_stalled(&mut self, stalled: bool) {
        self.stalled.set(stalled);
    }

    fn is_stalled(&self) -> bool {
        self.stalled.get()
    }
}

impl EndpointIn for TestEndpoint {
    async fn write_transfer(&mut self, _buf: &[u8], _needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        unreachable!()
    }
}

impl EndpointOut for TestEndpoint {
    async fn read_transfer(&mut self, _buf: &mut [u8]) -> Result<usize, EndpointError> {
        unreachable!()
    }
}

pub(crate) struct TestBus {
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
    pub(crate) enabled: [bool; 32],
    /// Shared with the endpoint handles, so a class stall is visible on the bus.
    pub(crate) stalled: [Rc<Cell<bool>>; 32],
    pub(crate) halt_calls: usize,
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
            enabled: [false; 32],
            stalled: core::array::from_fn(|_| Rc::new(Cell::new(false))),
            halt_calls: 0,
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
        Ok(TestEndpoint {
            info,
            stalled: self.stalled[slot].clone(),
        })
    }

    fn check_halt(&mut self, addr: EndpointAddress) -> usize {
        let slot = Self::slot(addr);
        let endpoint = self.endpoints[slot].unwrap();
        assert!(self.enabled[slot]);
        assert!(matches!(endpoint.ep_type, EndpointType::Bulk | EndpointType::Interrupt));
        self.halt_calls += 1;
        slot
    }
}

impl Bus for TestBus {
    async fn enable(&mut self) {}

    async fn disable(&mut self) {
        self.enabled.fill(false);
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
        self.enabled[slot] = enabled;
    }

    fn endpoint_is_enabled(&mut self, addr: EndpointAddress) -> bool {
        let slot = Self::slot(addr);
        self.enabled[slot]
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
pub(crate) enum Response {
    Accepted,
    Status(u16),
    Rejected,
}
