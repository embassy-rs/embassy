use core::cell::Cell;
use core::future::pending;

use embassy_futures::block_on;

use super::*;
use crate::control::{OutResponse, Recipient};
use crate::descriptor::{EndpointInterval, SynchronizationType, UsageType};
use crate::driver::{
    Direction, Endpoint, EndpointAllocError, EndpointError, EndpointIn, EndpointInfo, EndpointOut, Unsupported,
};
use crate::types::InterfaceNumber;

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
        let addr = ep_addr.unwrap_or_else(|| {
            let index = (1..9).find(|&i| self.bus.endpoints[i * 2 + 1].is_none()).unwrap();
            EndpointAddress::from_parts(index, Direction::In)
        });
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
        let addr = ep_addr.unwrap_or_else(|| {
            let index = (1..9).find(|&i| self.bus.endpoints[i * 2].is_none()).unwrap();
            EndpointAddress::from_parts(index, Direction::Out)
        });
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

#[cfg(all(feature = "device", feature = "host", feature = "class-ccid"))]
#[test]
fn test_ccid_device_and_host_interop() {
    let driver = TestDriver::default();
    let config = Config::new(0x1209, 0x0001);
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = crate::class::ccid::device::State::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let ccid_config = crate::class::ccid::device::Config {
        max_packet_size: 64,
        ..Default::default()
    };
    let _ccid = crate::class::ccid::device::CcidDevice::new(&mut builder, &mut state, ccid_config);
    let usb = builder.build();
    drop(usb);

    let info = crate::class::ccid::host::find_ccid(&config_descriptor).expect("CCID interface should be found");
    assert_eq!(info.interface_number, 0);
    assert_eq!(info.slots, 1);
    assert_eq!(info.bulk_in.1, 64);
    assert_eq!(info.bulk_out.1, 64);
}

#[cfg(all(feature = "device", feature = "class-bth"))]
#[test]
fn test_bluetooth_hci_device_descriptors() {
    let mut state = crate::class::bth::State::new();
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

    let bth = crate::class::bth::BluetoothHci::new(&mut builder, &mut state, crate::class::bth::Config::default());
    let _ = bth;
    let usb = builder.build();
    drop(usb);

    let desc = &config_descriptor[..];
    // Find Interface descriptor (bLength = 9, bDescriptorType = 4, class = 0xE0, subclass = 0x01, proto = 0x01)
    let iface = desc
        .windows(9)
        .find(|w| w[0] == 9 && w[1] == 4 && w[5] == 0xE0 && w[6] == 0x01 && w[7] == 0x01)
        .expect("Bluetooth HCI interface descriptor not found");
    assert_eq!(iface[4], 3); // 3 endpoints (Event IN, ACL OUT, ACL IN)
}

struct RecordingHandler<'a> {
    iface: InterfaceNumber,
    calls: &'a Cell<u32>,
    alt_calls: &'a Cell<u32>,
}

impl Handler for RecordingHandler<'_> {
    fn control_out(&mut self, req: Request, _data: &[u8]) -> Option<OutResponse> {
        if req.recipient == Recipient::Interface {
            self.calls.set(self.calls.get() + 1);
            Some(OutResponse::Accepted)
        } else {
            None
        }
    }

    fn set_alternate_setting(&mut self, iface: InterfaceNumber, _alternate_setting: u8) {
        if iface == self.iface {
            self.alt_calls.set(self.alt_calls.get() + 1);
        }
    }
}

impl InterfaceHandler for RecordingHandler<'_> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.iface
    }
}

#[test]
fn routed_handlers_only_see_their_interfaces() {
    let a_calls = Cell::new(0);
    let b_calls = Cell::new(0);
    let a_alts = Cell::new(0);
    let b_alts = Cell::new(0);
    let mut a = RecordingHandler {
        iface: InterfaceNumber::new(0),
        calls: &a_calls,
        alt_calls: &a_alts,
    };
    let mut b = RecordingHandler {
        iface: InterfaceNumber::new(1),
        calls: &b_calls,
        alt_calls: &b_alts,
    };

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

    let (iface_a, iface_b) = {
        let mut function = builder.function(0xff, 0, 0);
        let iface_a = {
            let mut iface = function.interface();
            let number = iface.interface_number();
            iface.alt_setting(0xff, 0, 0, None);
            number
        };
        let iface_b = {
            let mut iface = function.interface();
            let number = iface.interface_number();
            iface.alt_setting(0xff, 0, 0, None);
            number
        };
        (iface_a, iface_b)
    };

    builder.interface_handler(&mut a);
    builder.interface_handler(&mut b);
    let mut usb = builder.build();
    configure(&mut usb);

    a_calls.set(0);
    b_calls.set(0);
    a_alts.set(0);
    b_alts.set(0);

    assert_eq!(
        request(&mut usb, 0x21, 0x01, 0, iface_a.0 as u16, 0),
        Response::Accepted
    );
    assert_eq!(a_calls.get(), 1);
    assert_eq!(b_calls.get(), 0);

    assert_eq!(
        request(&mut usb, 0x21, 0x01, 0, iface_b.0 as u16, 0),
        Response::Accepted
    );
    assert_eq!(a_calls.get(), 1);
    assert_eq!(b_calls.get(), 1);

    // Device-recipient requests must not reach interface handlers.
    assert_eq!(request(&mut usb, 0x40, 0x01, 0, 0, 0), Response::Rejected);
    assert_eq!(a_calls.get(), 1);
    assert_eq!(b_calls.get(), 1);

    // A bus reset delivers set_alternate_setting only to the owning handler.
    block_on(usb.inner.handle_bus_event(Event::Reset));
    assert_eq!(a_alts.get(), 1);
    assert_eq!(b_alts.get(), 1);
}

#[test]
fn test_endpoint_interval_and_other_speed_config() {
    let mut config_descriptor = [0; 256];
    let mut other_speed_config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut control_buf = [0; 64];

    let mut config = Config::new(0x1234, 0x5678);
    config.max_speed = UsbDeviceSpeed::High;
    config.max_packet_size_0 = 64;

    let mut builder = Builder::new(
        TestDriver::default(),
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );
    builder.other_speed_config_descriptor(&mut other_speed_config_descriptor);

    let mut function = builder.function(0xff, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xff, 0, 0, None);

    // 1. Interrupt IN endpoint with 4 ms (Interval::Milliseconds(4))
    alt.endpoint_interrupt_in(Some(0x81.into()), 64, 4);

    // 2. High-speed sub-millisecond Isochronous IN endpoint with 250 µs (Microframes(2))
    alt.endpoint_isochronous_in_interval(
        Some(0x82.into()),
        64,
        EndpointInterval::Microframes(2),
        SynchronizationType::NoSynchronization,
        UsageType::DataEndpoint,
        &[],
    );

    // 3. Isochronous OUT endpoint with 4 ms (Milliseconds(4))
    alt.endpoint_isochronous_out(
        Some(0x03.into()),
        64,
        4,
        SynchronizationType::NoSynchronization,
        UsageType::DataEndpoint,
        &[],
    );

    // 4. Raw endpoint with bInterval = 42 (Raw(42))
    alt.endpoint_interrupt_out_interval(Some(0x04.into()), 64, EndpointInterval::Raw(42));

    drop(function);
    let mut usb = builder.build();

    // Check High-Speed Configuration Descriptor
    let hs_desc = usb.inner.config_descriptor;
    assert_eq!(hs_desc[1], descriptor_type::CONFIGURATION); // 0x02

    // Helper to find bInterval of endpoint address
    let find_interval = |desc: &[u8], ep_addr: u8| -> u8 {
        desc.windows(7)
            .find(|w| w[0] == 7 && w[1] == descriptor_type::ENDPOINT && w[2] == ep_addr)
            .map(|w| w[6])
            .expect("endpoint not found")
    };

    // 1. Interrupt IN: 4 ms -> 2^(bInterval-1) = 32 microframes -> bInterval = 6
    assert_eq!(find_interval(hs_desc, 0x81), 6);

    // 2. Isochronous IN: 2 microframes -> 2^(bInterval-1) = 2 microframes -> bInterval = 2
    assert_eq!(find_interval(hs_desc, 0x82), 2);

    // 3. Isochronous OUT: 4 ms -> 32 microframes -> bInterval = 6
    assert_eq!(find_interval(hs_desc, 0x03), 6);

    // 4. Raw Interrupt OUT: 42 -> unchanged
    assert_eq!(find_interval(hs_desc, 0x04), 42);

    // Check Other-Speed (Full-Speed) Configuration Descriptor
    let other_desc = usb
        .inner
        .other_speed_config_descriptor
        .expect("other speed desc present");
    assert_eq!(other_desc[1], descriptor_type::OTHER_SPEED_CONFIGURATION); // 0x07
    assert_eq!(other_desc.len(), hs_desc.len());

    // 1. Interrupt IN at FS: 4 ms linear -> bInterval = 4
    assert_eq!(find_interval(other_desc, 0x81), 4);

    // 2. Isochronous IN at FS: 2 microframes <= 8 microframes -> 1 frame -> 2^(bInterval-1) = 1 frame -> bInterval = 1
    assert_eq!(find_interval(other_desc, 0x82), 1);

    // 3. Isochronous OUT at FS: 4 ms exponential -> 2^(bInterval-1) = 4 frames -> bInterval = 3
    assert_eq!(find_interval(other_desc, 0x03), 3);

    // 4. Raw Interrupt OUT at FS: 42 -> unchanged
    assert_eq!(find_interval(other_desc, 0x04), 42);

    // Test GET_DESCRIPTOR for OTHER_SPEED_CONFIGURATION via control transfer
    let mut buf = [0u8; 256];
    let req = Request {
        direction: Direction::In,
        request_type: RequestType::Standard,
        recipient: Recipient::Device,
        request: Request::GET_DESCRIPTOR,
        value: ((descriptor_type::OTHER_SPEED_CONFIGURATION as u16) << 8) | 0,
        index: 0,
        length: 256,
    };
    match usb.inner.handle_control_in(req, &mut buf) {
        InResponse::Accepted(data) => {
            assert_eq!(data[1], descriptor_type::OTHER_SPEED_CONFIGURATION);
            assert_eq!(data.len(), other_desc.len());
        }
        _ => panic!("GET_DESCRIPTOR(OTHER_SPEED_CONFIGURATION) should be accepted"),
    }

    // Index != 0 should be rejected
    let req_invalid_idx = Request {
        direction: Direction::In,
        request_type: RequestType::Standard,
        recipient: Recipient::Device,
        request: Request::GET_DESCRIPTOR,
        value: ((descriptor_type::OTHER_SPEED_CONFIGURATION as u16) << 8) | 1,
        index: 0,
        length: 256,
    };
    assert!(matches!(
        usb.inner.handle_control_in(req_invalid_idx, &mut buf),
        InResponse::Rejected
    ));
}

#[test]
fn test_other_speed_config_rejected_on_full_speed_or_missing_buffer() {
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut control_buf = [0; 64];

    // High speed but no other_speed buffer
    let mut config = Config::new(0x1234, 0x5678);
    config.max_speed = UsbDeviceSpeed::High;
    config.max_packet_size_0 = 64;

    let builder = Builder::new(
        TestDriver::default(),
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );
    let mut usb = builder.build();

    let mut buf = [0u8; 256];
    let req = Request {
        direction: Direction::In,
        request_type: RequestType::Standard,
        recipient: Recipient::Device,
        request: Request::GET_DESCRIPTOR,
        value: ((descriptor_type::OTHER_SPEED_CONFIGURATION as u16) << 8) | 0,
        index: 0,
        length: 256,
    };
    assert!(matches!(
        usb.inner.handle_control_in(req, &mut buf),
        InResponse::Rejected
    ));

    // Full speed device
    let mut config_descriptor_fs = [0; 256];
    let mut bos_descriptor_fs = [0; 64];
    let mut control_buf_fs = [0; 64];
    let mut other_speed_buf_fs = [0; 256];

    let mut config_fs = Config::new(0x1234, 0x5678);
    config_fs.max_speed = UsbDeviceSpeed::Full;

    let mut builder_fs = Builder::new(
        TestDriver::default(),
        config_fs,
        &mut config_descriptor_fs,
        &mut bos_descriptor_fs,
        &mut [],
        &mut control_buf_fs,
    );
    builder_fs.other_speed_config_descriptor(&mut other_speed_buf_fs);
    let mut usb_fs = builder_fs.build();

    assert!(matches!(
        usb_fs.inner.handle_control_in(req, &mut buf),
        InResponse::Rejected
    ));
}

#[test]
fn test_manual_raw_descriptor_opt_out() {
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut control_buf = [0; 64];

    let mut config = Config::new(0x1234, 0x5678);
    config.max_speed = UsbDeviceSpeed::High;
    config.max_packet_size_0 = 64;

    let mut builder = Builder::new(
        TestDriver::default(),
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let mut function = builder.function(0xff, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xff, 0, 0, None);

    // Manually write an endpoint descriptor with raw descriptor() call
    alt.descriptor(
        descriptor_type::ENDPOINT,
        &[0x81, 0x03, 64, 0, 99], // Interrupt IN with bInterval = 99
    );

    drop(function);
    let usb = builder.build();

    let desc = usb.inner.config_descriptor;
    let find_interval = |desc: &[u8], ep_addr: u8| -> u8 {
        desc.windows(7)
            .find(|w| w[0] == 7 && w[1] == descriptor_type::ENDPOINT && w[2] == ep_addr)
            .map(|w| w[6])
            .expect("endpoint not found")
    };

    // The manual endpoint descriptor's bInterval must NOT be rewritten!
    assert_eq!(find_interval(desc, 0x81), 99);
}

#[test]
fn shared_endpoints_stay_enabled_across_alternate_settings() {
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

    // Alternate setting 0 references only the shared IN endpoint.
    let mut alt0 = interface.alt_setting(0xff, 0, 0, None);
    let shared_in = alt0.alloc_endpoint_in(EndpointType::Bulk, None, 64, 0);
    alt0.endpoint_descriptor(
        shared_in.info(),
        SynchronizationType::NoSynchronization,
        UsageType::DataEndpoint,
        &[],
    );

    // Alternate setting 1 reuses it and adds an OUT endpoint of its own.
    let mut alt1 = interface.alt_setting(0xff, 0, 0, None);
    alt1.endpoint_descriptor(
        shared_in.info(),
        SynchronizationType::NoSynchronization,
        UsageType::DataEndpoint,
        &[],
    );
    let alt1_out = alt1.alloc_endpoint_out(EndpointType::Bulk, None, 64, 0);
    alt1.endpoint_descriptor(
        alt1_out.info(),
        SynchronizationType::NoSynchronization,
        UsageType::DataEndpoint,
        &[],
    );

    drop(function);
    let mut usb = builder.build();

    let slot = |addr: EndpointAddress| addr.index() * 2 + usize::from(addr.is_in());
    let in_addr = shared_in.info().addr;
    let out_addr = alt1_out.info().addr;

    // Enumeration selects alternate setting 0: only the shared endpoint is enabled.
    configure(&mut usb);
    assert!(usb.inner.bus.enabled[slot(in_addr)]);
    assert!(!usb.inner.bus.enabled[slot(out_addr)]);

    // Selecting alternate setting 1 enables both endpoints.
    assert_eq!(
        request(&mut usb, 1, Request::SET_INTERFACE, 1, 0, 0),
        Response::Accepted
    );
    assert!(usb.inner.bus.enabled[slot(in_addr)]);
    assert!(usb.inner.bus.enabled[slot(out_addr)]);

    // Going back to alternate setting 0 must keep the shared endpoint enabled and
    // disable the endpoint that only exists in alternate setting 1.
    assert_eq!(
        request(&mut usb, 1, Request::SET_INTERFACE, 0, 0, 0),
        Response::Accepted
    );
    assert!(usb.inner.bus.enabled[slot(in_addr)]);
    assert!(!usb.inner.bus.enabled[slot(out_addr)]);
}

#[cfg(feature = "class-midi")]
#[test]
fn test_midi2_device_enumeration_and_control() {
    use crate::class::midi::device::{Midi2Config, MidiClass, MidiClassConfig, MidiClassState};
    use crate::class::midi::{
        CS_GR_TRM_BLOCK, GroupTerminalBlockConfig, GroupTerminalBlockType, MidiProtocol, parse_gtb_descriptors,
    };

    static GTBS: &[GroupTerminalBlockConfig<'static>] = &[
        GroupTerminalBlockConfig {
            block_id: 1,
            block_type: GroupTerminalBlockType::Bidirectional,
            first_group: 0,
            num_groups: 2,
            protocol: MidiProtocol::Midi2_0,
            name: Some("Synth Ports 1-2"),
            max_input_bandwidth: 0,
            max_output_bandwidth: 0,
        },
        GroupTerminalBlockConfig {
            block_id: 2,
            block_type: GroupTerminalBlockType::InputOnly,
            first_group: 2,
            num_groups: 1,
            protocol: MidiProtocol::Midi1_0_64,
            name: Some("Legacy DIN In"),
            max_input_bandwidth: 3125,
            max_output_bandwidth: 0,
        },
    ];

    let mut state = MidiClassState::new();
    let mut config_descriptor = [0; 512];
    let mut bos_descriptor = [0; 64];
    let mut control_buf = [0; 256];

    let mut config = Config::new(0x1234, 0x5678);
    config.max_packet_size_0 = 64;

    let mut builder = Builder::new(
        TestDriver::default(),
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let midi_config = MidiClassConfig {
        n_in_jacks: 1,
        n_out_jacks: 1,
        max_packet_size: 64,
        interface_name: Some("MIDI 2.0 Interface"),
        in_jack_names: &[Some("MIDI In")],
        out_jack_names: &[Some("MIDI Out")],
        midi2: Some(Midi2Config {
            group_terminal_blocks: GTBS,
        }),
    };

    let _midi = MidiClass::new_with_names(&mut builder, &mut state, midi_config);
    let mut usb = builder.build();

    // Both alternate settings must reference the same bulk endpoints. The
    // MIDI 2.0 spec expects Alt 1 to reuse Alt 0's endpoints, and a second
    // pair would waste scarce endpoint hardware on most MCUs.
    let desc = usb.inner.config_descriptor;
    let mut endpoints_by_alt = [[None; 2]; 2];
    let mut current_alt = None;
    let mut offset = 0;
    while offset + 1 < desc.len() {
        let len = desc[offset] as usize;
        if len < 2 || offset + len > desc.len() {
            break;
        }
        match desc[offset + 1] {
            descriptor_type::INTERFACE => {
                current_alt = (desc[offset + 2] == 1).then_some(desc[offset + 3] as usize);
            }
            descriptor_type::ENDPOINT => {
                if let Some(alt) = current_alt {
                    let addr = desc[offset + 2];
                    let direction = if addr & 0x80 != 0 { 0 } else { 1 };
                    endpoints_by_alt[alt][direction] = Some(addr);
                }
            }
            _ => {}
        }
        offset += len;
    }
    assert_eq!(
        endpoints_by_alt[0], endpoints_by_alt[1],
        "Alt 0 and Alt 1 must share the same endpoint addresses"
    );
    assert_eq!(endpoints_by_alt[0], [Some(0x81), Some(0x01)]);

    // Enumerate the device. Alternate Setting 0 is selected by default, and the
    // endpoints it shares with Alternate Setting 1 must be enabled.
    let set_config_req = Request {
        direction: Direction::Out,
        request_type: RequestType::Standard,
        recipient: Recipient::Device,
        request: Request::SET_CONFIGURATION,
        value: 1,
        index: 0,
        length: 0,
    };
    assert_eq!(usb.inner.handle_control_out(set_config_req, &[]), OutResponse::Accepted);

    let slot = |addr: u8| {
        let addr = EndpointAddress::from(addr);
        addr.index() * 2 + usize::from(addr.is_in())
    };
    assert!(usb.inner.bus.enabled[slot(0x81)]);
    assert!(usb.inner.bus.enabled[slot(0x01)]);

    // Switch interface 1 to Alternate Setting 1 (MIDI 2.0)
    let set_alt_req = Request {
        direction: Direction::Out,
        request_type: RequestType::Standard,
        recipient: Recipient::Interface,
        request: Request::SET_INTERFACE,
        value: 1,
        index: 1,
        length: 0,
    };
    assert_eq!(usb.inner.handle_control_out(set_alt_req, &[]), OutResponse::Accepted);
    assert!(usb.inner.bus.enabled[slot(0x81)]);
    assert!(usb.inner.bus.enabled[slot(0x01)]);

    // Switching back to MIDI 1.0 must not disable the shared endpoints.
    let set_alt0_req = Request {
        value: 0,
        ..set_alt_req
    };
    assert_eq!(usb.inner.handle_control_out(set_alt0_req, &[]), OutResponse::Accepted);
    assert!(usb.inner.bus.enabled[slot(0x81)]);
    assert!(usb.inner.bus.enabled[slot(0x01)]);

    // Switch interface 1 to invalid Alternate Setting 2
    let set_alt_invalid = Request {
        direction: Direction::Out,
        request_type: RequestType::Standard,
        recipient: Recipient::Interface,
        request: Request::SET_INTERFACE,
        value: 2,
        index: 1,
        length: 0,
    };
    assert_eq!(
        usb.inner.handle_control_out(set_alt_invalid, &[]),
        OutResponse::Rejected
    );

    // Query Group Terminal Block descriptors with the standard GET_DESCRIPTOR
    // request: wValue = CS_GR_TRM_BLOCK in the high byte, alternate setting in
    // the low byte; wIndex = interface number.
    let get_gtb_req = Request {
        direction: Direction::In,
        request_type: RequestType::Standard,
        recipient: Recipient::Interface,
        request: Request::GET_DESCRIPTOR,
        value: ((CS_GR_TRM_BLOCK as u16) << 8) | 1,
        index: 1,
        length: 256,
    };
    let mut resp_buf = [0u8; 256];

    // A class-specific request must not return the GTB descriptors.
    let class_gtb_req = Request {
        request_type: RequestType::Class,
        ..get_gtb_req
    };
    assert!(matches!(
        usb.inner.handle_control_in(class_gtb_req, &mut resp_buf),
        InResponse::Rejected
    ));

    // The alternate setting in wValue must be the MIDI 2.0 one (1).
    let wrong_alt_gtb_req = Request {
        value: (CS_GR_TRM_BLOCK as u16) << 8,
        ..get_gtb_req
    };
    assert!(matches!(
        usb.inner.handle_control_in(wrong_alt_gtb_req, &mut resp_buf),
        InResponse::Rejected
    ));

    match usb.inner.handle_control_in(get_gtb_req, &mut resp_buf) {
        InResponse::Accepted(data) => {
            let blocks: heapless::Vec<_, 4> = parse_gtb_descriptors(data).expect("valid GTB descriptors");
            assert_eq!(blocks.len(), 2);
            assert_eq!(blocks[0].block_id, 1);
            assert_eq!(blocks[0].first_group, 0);
            assert_eq!(blocks[0].num_groups, 2);
            assert_eq!(blocks[0].protocol, MidiProtocol::Midi2_0);
            assert_eq!(blocks[1].block_id, 2);
            assert_eq!(blocks[1].first_group, 2);
            assert_eq!(blocks[1].num_groups, 1);
            assert_eq!(blocks[1].protocol, MidiProtocol::Midi1_0_64);
        }
        InResponse::Rejected => panic!("GET_DESCRIPTOR for GTB was rejected"),
    }
}
