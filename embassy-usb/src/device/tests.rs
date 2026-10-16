use core::future::pending;

use embassy_futures::block_on;

use super::*;
use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{Endpoint, EndpointAllocError, EndpointError, EndpointIn, EndpointInfo, EndpointOut, Unsupported};

#[derive(Default)]
struct TestDriver {
    bus: TestBus,
}

struct TestEndpoint(EndpointInfo);

impl<'d> Driver<'d> for TestDriver {
    type EndpointOut = TestEndpoint;
    type EndpointIn = TestEndpoint;
    type ControlPipe = TestControlPipe;
    type Bus = TestBus;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<TestEndpoint, EndpointAllocError> {
        let addr = ep_addr.unwrap();
        assert!(addr.is_in());
        self.bus.allocate(EndpointInfo {
            addr,
            ep_type,
            max_packet_size,
            interval_ms,
        })
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<TestEndpoint, EndpointAllocError> {
        let addr = ep_addr.unwrap();
        assert!(addr.is_out());
        self.bus.allocate(EndpointInfo {
            addr,
            ep_type,
            max_packet_size,
            interval_ms,
        })
    }

    fn start(self, _control_max_packet_size: u16) -> (TestBus, TestControlPipe) {
        (self.bus, TestControlPipe::default())
    }
}

impl Endpoint for TestEndpoint {
    fn info(&self) -> &EndpointInfo {
        &self.0
    }

    async fn wait_enabled(&mut self) {
        unreachable!()
    }
}

impl EndpointIn for TestEndpoint {
    async fn write(&mut self, _buf: &[u8]) -> Result<(), EndpointError> {
        unreachable!()
    }
}

impl EndpointOut for TestEndpoint {
    async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, EndpointError> {
        unreachable!()
    }
}

#[derive(Default)]
struct TestBus {
    endpoints: [Option<EndpointInfo>; 32],
    enabled: [bool; 32],
    stalled: [bool; 32],
    halt_calls: usize,
}

impl TestBus {
    fn slot(addr: EndpointAddress) -> usize {
        // Model a controller with nine endpoint register pairs, including EP0.
        assert!(addr.index() < 9);
        addr.index() * 2 + usize::from(addr.is_in())
    }

    fn allocate(&mut self, info: EndpointInfo) -> Result<TestEndpoint, EndpointAllocError> {
        let slot = Self::slot(info.addr);
        assert!(self.endpoints[slot].replace(info).is_none());
        Ok(TestEndpoint(info))
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
        pending().await
    }

    fn endpoint_set_enabled(&mut self, addr: EndpointAddress, enabled: bool) {
        let slot = Self::slot(addr);
        assert!(self.endpoints[slot].is_some());
        self.enabled[slot] = enabled;
        self.stalled[slot] = false;
    }

    fn endpoint_set_stalled(&mut self, addr: EndpointAddress, stalled: bool) {
        let slot = self.check_halt(addr);
        self.stalled[slot] = stalled;
    }

    fn endpoint_is_stalled(&mut self, addr: EndpointAddress) -> bool {
        let slot = self.check_halt(addr);
        self.stalled[slot]
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Response {
    Accepted,
    Status(u16),
    Rejected,
}

#[derive(Default)]
struct TestControlPipe {
    response: Option<Response>,
}

impl ControlPipe for TestControlPipe {
    fn max_packet_size(&self) -> usize {
        64
    }

    async fn setup(&mut self) -> [u8; 8] {
        unreachable!()
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        buf.fill(0);
        Ok(buf.len())
    }

    async fn data_in(&mut self, data: &[u8], first: bool, last: bool) -> Result<(), EndpointError> {
        assert!(first && last);
        self.response = Some(Response::Status(u16::from_le_bytes(data.try_into().unwrap())));
        Ok(())
    }

    async fn accept(&mut self) {
        self.response = Some(Response::Accepted);
    }

    async fn reject(&mut self) {
        self.response = Some(Response::Rejected);
    }

    async fn accept_set_address(&mut self, _addr: u8) {
        self.accept().await;
    }
}

fn with_device(f: impl FnOnce(&mut UsbDevice<'_, TestDriver>)) {
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut control_buf = [0; 64];
    let mut builder = Builder::new(
        TestDriver::default(),
        Config::new(0x1234, 0x5678),
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );
    let mut function = builder.function(0xff, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xff, 0, 0, None);
    alt.endpoint_bulk_out(Some(0x01.into()), 64);
    alt.endpoint_bulk_in(Some(0x82.into()), 64);
    let mut alt = interface.alt_setting(0xff, 0, 0, None);
    alt.endpoint_interrupt_in(Some(0x83.into()), 64, 1);
    alt.endpoint_isochronous_out(
        Some(0x04.into()),
        64,
        1,
        SynchronizationType::Asynchronous,
        UsageType::DataEndpoint,
        &[],
    );
    drop(function);
    f(&mut builder.build());
}

fn request(
    usb: &mut UsbDevice<'_, TestDriver>,
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
    length: u16,
) -> Response {
    let value = value.to_le_bytes();
    let index = index.to_le_bytes();
    let length = length.to_le_bytes();
    block_on(usb.handle_control([
        request_type,
        request,
        value[0],
        value[1],
        index[0],
        index[1],
        length[0],
        length[1],
    ]));
    usb.control.response.take().unwrap()
}

fn endpoint_request(usb: &mut UsbDevice<'_, TestDriver>, code: u8, index: u16) -> Response {
    if code == Request::GET_STATUS {
        request(usb, 0x82, code, 0, index, 2)
    } else {
        request(usb, 0x02, code, 0, index, 0)
    }
}

fn assert_rejected(usb: &mut UsbDevice<'_, TestDriver>, code: u8, index: u16) {
    let calls = usb.inner.bus.halt_calls;
    assert_eq!(endpoint_request(usb, code, index), Response::Rejected);
    assert_eq!(usb.inner.bus.halt_calls, calls);
}

fn configure(usb: &mut UsbDevice<'_, TestDriver>) {
    block_on(usb.inner.handle_bus_event(Event::PowerDetected));
    block_on(usb.inner.handle_bus_event(Event::Reset));
    assert_eq!(request(usb, 0, Request::SET_ADDRESS, 1, 0, 0), Response::Accepted);
    assert_eq!(request(usb, 0, Request::SET_CONFIGURATION, 1, 0, 0), Response::Accepted);
}

#[test]
fn test_handle_control_endpoint_out_of_range() {
    with_device(|usb| {
        configure(usb);
        // Endpoint 9 is outside the controller's register array.
        block_on(usb.handle_control([0x82, 0, 0, 0, 0x89, 0, 2, 0]));
        assert_eq!(usb.control.response.take(), Some(Response::Rejected));
        assert_eq!(usb.inner.bus.halt_calls, 0);
    });
}

#[test]
fn test_handle_control_endpoint_index() {
    with_device(|usb| {
        configure(usb);
        for index in 0..=u16::MAX {
            // Only these endpoints exist in the default alternate setting.
            if matches!(index, 0 | 0x80 | 1 | 0x82) {
                continue;
            }
            for code in [Request::GET_STATUS, Request::SET_FEATURE, Request::CLEAR_FEATURE] {
                assert_rejected(usb, code, index);
            }
        }
    });
}

#[test]
fn test_handle_control_endpoint_halt() {
    with_device(|usb| {
        configure(usb);
        for index in [1, 0x82] {
            assert_eq!(endpoint_request(usb, Request::GET_STATUS, index), Response::Status(0));
            assert_eq!(endpoint_request(usb, Request::SET_FEATURE, index), Response::Accepted);
            assert_eq!(endpoint_request(usb, Request::GET_STATUS, index), Response::Status(1));
            assert_eq!(endpoint_request(usb, Request::CLEAR_FEATURE, index), Response::Accepted);
            assert_eq!(endpoint_request(usb, Request::GET_STATUS, index), Response::Status(0));
        }
        assert_eq!(usb.inner.bus.halt_calls, 10);
    });
}

#[test]
fn test_handle_control_endpoint_state() {
    with_device(|usb| {
        let assert_unconfigured = |usb: &mut UsbDevice<'_, TestDriver>| {
            for index in [1, 0x82, 0x83, 4] {
                for code in [Request::GET_STATUS, Request::SET_FEATURE, Request::CLEAR_FEATURE] {
                    assert_rejected(usb, code, index);
                }
            }
        };
        assert_unconfigured(usb);
        block_on(usb.inner.handle_bus_event(Event::PowerDetected));
        assert_unconfigured(usb);
        assert_eq!(request(usb, 0, Request::SET_ADDRESS, 1, 0, 0), Response::Accepted);
        assert_unconfigured(usb);
        assert_eq!(request(usb, 0, Request::SET_CONFIGURATION, 1, 0, 0), Response::Accepted);
        assert_eq!(endpoint_request(usb, Request::GET_STATUS, 1), Response::Status(0));
        assert_eq!(request(usb, 0, Request::SET_CONFIGURATION, 0, 0, 0), Response::Accepted);
        assert_unconfigured(usb);
        configure(usb);
        block_on(usb.inner.handle_bus_event(Event::Reset));
        assert_unconfigured(usb);
        configure(usb);
        block_on(usb.inner.handle_bus_event(Event::PowerRemoved));
        assert_unconfigured(usb);
        configure(usb);
        block_on(usb.disable());
        assert_unconfigured(usb);
    });
}

#[test]
fn test_handle_control_endpoint_alt_setting() {
    with_device(|usb| {
        configure(usb);
        assert_eq!(request(usb, 1, Request::SET_INTERFACE, 1, 0, 0), Response::Accepted);
        for code in [Request::GET_STATUS, Request::SET_FEATURE, Request::CLEAR_FEATURE] {
            for index in [1, 0x82, 3, 0x84] {
                assert_rejected(usb, code, index);
            }
        }
        assert_eq!(endpoint_request(usb, Request::SET_FEATURE, 0x83), Response::Accepted);
        assert_eq!(endpoint_request(usb, Request::GET_STATUS, 0x83), Response::Status(1));
        assert_eq!(endpoint_request(usb, Request::CLEAR_FEATURE, 0x83), Response::Accepted);
        assert_eq!(endpoint_request(usb, Request::GET_STATUS, 0x83), Response::Status(0));

        // Isochronous endpoints exist, but have no halt feature.
        let calls = usb.inner.bus.halt_calls;
        assert_eq!(endpoint_request(usb, Request::GET_STATUS, 4), Response::Status(0));
        assert_rejected(usb, Request::SET_FEATURE, 4);
        assert_rejected(usb, Request::CLEAR_FEATURE, 4);
        assert_eq!(usb.inner.bus.halt_calls, calls);

        assert_eq!(request(usb, 1, Request::SET_INTERFACE, 0, 0, 0), Response::Accepted);
        for code in [Request::GET_STATUS, Request::SET_FEATURE, Request::CLEAR_FEATURE] {
            assert_rejected(usb, code, 0x83);
            assert_rejected(usb, code, 4);
        }
        assert_eq!(endpoint_request(usb, Request::GET_STATUS, 1), Response::Status(0));
    });
}

#[test]
fn test_handle_control_endpoint_zero() {
    with_device(|usb| {
        block_on(usb.inner.handle_bus_event(Event::PowerDetected));
        for index in [0, 0x80] {
            assert_rejected(usb, Request::GET_STATUS, index);
        }
        assert_eq!(request(usb, 0, Request::SET_ADDRESS, 1, 0, 0), Response::Accepted);
        for configuration in [0, 1] {
            assert_eq!(
                request(usb, 0, Request::SET_CONFIGURATION, configuration, 0, 0),
                Response::Accepted
            );
            for index in [0, 0x80] {
                assert_eq!(endpoint_request(usb, Request::GET_STATUS, index), Response::Status(0));
                assert_rejected(usb, Request::SET_FEATURE, index);
                assert_rejected(usb, Request::CLEAR_FEATURE, index);
            }
        }
        assert_eq!(usb.inner.bus.halt_calls, 0);
    });
}

#[test]
fn test_handle_control_endpoint_request_fields() {
    with_device(|usb| {
        configure(usb);
        for (value, length) in [(1, 2), (0, 0), (0, 1), (0, 3)] {
            assert_eq!(
                request(usb, 0x82, Request::GET_STATUS, value, 1, length),
                Response::Rejected
            );
        }
        for code in [Request::SET_FEATURE, Request::CLEAR_FEATURE] {
            for (value, length) in [(1, 0), (0, 1)] {
                assert_eq!(request(usb, 0x02, code, value, 1, length), Response::Rejected);
            }
        }
        assert_eq!(usb.inner.bus.halt_calls, 0);
    });
}
