use core::cell::Cell;
use core::future::pending;

use embassy_futures::block_on;

use super::*;
use crate::control::{OutResponse, Recipient};
use crate::descriptor::{EndpointInterval, SynchronizationType, UsageType};
use crate::driver::{
    Direction, Endpoint, EndpointAllocError, EndpointConfig, EndpointError, EndpointIn, EndpointInfo, EndpointOut,
    Unsupported,
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
        self.alloc_endpoint_in_with_config(EndpointConfig::new(ep_addr, ep_type, max_packet_size, interval_ms))
    }

    fn alloc_endpoint_in_with_config(&mut self, config: EndpointConfig) -> Result<TestEndpoint, EndpointAllocError> {
        let addr = config.addr.unwrap_or_else(|| {
            let index = (1..9).find(|&i| self.bus.endpoints[i * 2 + 1].is_none()).unwrap();
            EndpointAddress::from_parts(index, Direction::In)
        });
        assert!(addr.is_in());
        self.bus.allocate(addr, config)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<TestEndpoint, EndpointAllocError> {
        self.alloc_endpoint_out_with_config(EndpointConfig::new(ep_addr, ep_type, max_packet_size, interval_ms))
    }

    fn alloc_endpoint_out_with_config(&mut self, config: EndpointConfig) -> Result<TestEndpoint, EndpointAllocError> {
        let addr = config.addr.unwrap_or_else(|| {
            let index = (1..9).find(|&i| self.bus.endpoints[i * 2].is_none()).unwrap();
            EndpointAddress::from_parts(index, Direction::Out)
        });
        assert!(addr.is_out());
        self.bus.allocate(addr, config)
    }

    fn start(self, control_max_packet_size: u16) -> (TestBus, TestControlPipe) {
        let control = TestControlPipe {
            started_with: control_max_packet_size,
            ..TestControlPipe::default()
        };
        (self.bus, control)
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
    configs: [Option<EndpointConfig>; 32],
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

    fn allocate(&mut self, addr: EndpointAddress, config: EndpointConfig) -> Result<TestEndpoint, EndpointAllocError> {
        let info = EndpointInfo {
            addr,
            ep_type: config.ep_type,
            max_packet_size: config.max_packet_size,
            interval_ms: config.interval_ms,
        };
        let slot = Self::slot(addr);
        assert!(self.endpoints[slot].replace(info).is_none());
        self.configs[slot] = Some(config);
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
    /// The control endpoint size passed to `Driver::start`.
    started_with: u16,
    /// The data stage of the next control OUT request.
    out_data: [u8; 8],
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
        let len = buf.len().min(self.out_data.len());
        buf[..len].copy_from_slice(&self.out_data[..len]);
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

fn with_device_fullspeed(f: impl FnOnce(&mut UsbDevice<'_, TestDriver>)) {
    with_config(Config::new(0x1234, 0x5678), f)
}

fn with_device_superspeed(f: impl FnOnce(&mut UsbDevice<'_, TestDriver>)) {
    with_config(superspeed(UsbDeviceSpeed::Super, UsbVersion::ThreeTwo, 15), f)
}

fn with_config(config: Config<'_>, f: impl FnOnce(&mut UsbDevice<'_, TestDriver>)) {
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut control_buf = [0; 64];
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
    with_device_fullspeed(|usb| {
        configure(usb);
        // Endpoint 9 is outside the controller's register array.
        block_on(usb.handle_control([0x82, 0, 0, 0, 0x89, 0, 2, 0]));
        assert_eq!(usb.control.response.take(), Some(Response::Rejected));
        assert_eq!(usb.inner.bus.halt_calls, 0);
    });
}

#[test]
fn test_handle_control_endpoint_index() {
    with_device_fullspeed(|usb| {
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
    with_device_fullspeed(|usb| {
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
    with_device_fullspeed(|usb| {
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
    with_device_fullspeed(|usb| {
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
    with_device_fullspeed(|usb| {
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
    with_device_fullspeed(|usb| {
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

fn superspeed(speed: UsbDeviceSpeed, bcd_usb: UsbVersion, max_burst: u8) -> Config<'static> {
    let mut config = Config::new(0x1234, 0x5678);
    config.max_speed = speed;
    config.max_packet_size_0 = 9;
    config.bcd_usb = bcd_usb;
    config.max_burst = max_burst;
    config.max_power = 900;
    config
}

#[test]
fn test_superspeed_descriptors() {
    // SuperSpeed USB Device Capability
    const SS_CAP: [u8; 10] = [
        10, 0x10, 0x03, // bLength, bDescriptorType, bDevCapabilityType
        0x00, // bmAttributes
        0x08, 0x00, // wSpeedsSupported: Gen 1
        0x03, // bFunctionalitySupport
        0x0a, // bU1DevExitLat
        0xff, 0x07, // wU2DevExitLat
    ];
    // SuperSpeedPlus USB Device Capability
    const SSP_CAP: [u8; 20] = [
        20, 0x10, 0x0a, // bLength, bDescriptorType, bDevCapabilityType
        0,    // bReserved
        0x01, 0, 0, 0, // bmAttributes: 2 sublink speed attributes
        0x00, 0x11, // wFunctionalitySupport: 1 RX and 1 TX lane
        0, 0, // wReserved
        0x30, 0x40, 0x0a, 0, // bmSublinkSpeedAttr[0]: RX, 10 Gb/s
        0xb0, 0x40, 0x0a, 0, // bmSublinkSpeedAttr[1]: TX, 10 Gb/s
    ];

    with_device_superspeed(|usb| {
        let (d, c, b) = (
            &usb.inner.device_descriptor,
            usb.inner.config_descriptor,
            usb.inner.bos_descriptor,
        );
        // Device: bcdUSB = 3.2, bMaxPacketSize0 = 9; the driver is started with 512.
        assert_eq!((&d[2..4], d[7], usb.control.started_with), (&[0x20, 0x03][..], 9, 512));
        // Configuration: wTotalLength, bMaxPower = 113 (900 mA in 8 mA units).
        assert_eq!((u16::from_le_bytes([c[2], c[3]]) as usize, c[8]), (c.len(), 113));
        // Alternate setting 0, after the configuration, IAD and interface descriptors.
        assert_eq!(
            &c[26..52],
            &[
                7, 5, 0x01, 2, 64, 0, 0, // endpoint: bulk OUT, wMaxPacketSize = 64
                6, 0x30, 15, 0, 0, 0, // SS endpoint companion: bMaxBurst = 15
                7, 5, 0x82, 2, 64, 0, 0, // endpoint: bulk IN, wMaxPacketSize = 64
                6, 0x30, 15, 0, 0, 0, // SS endpoint companion: bMaxBurst = 15
            ]
        );
        // Alternate setting 1, after its interface descriptor.
        assert_eq!(
            &c[61..],
            &[
                7, 5, 0x83, 3, 64, 0, 4, // endpoint: interrupt IN, bInterval = 4
                6, 0x30, 0, 0, 64, 0, // SS endpoint companion: wBytesPerInterval = 64
                7, 5, 0x04, 5, 64, 0, 4, // endpoint: isochronous OUT, bInterval = 4
                6, 0x30, 0, 0, 64, 0, // SS endpoint companion: wBytesPerInterval = 64
            ]
        );
        // BOS: wTotalLength = 22, bNumDeviceCaps = 2; SS capability after the USB 2.0 extension.
        assert_eq!((&b[2..5], &b[12..]), (&[22, 0, 2][..], &SS_CAP[..]));
        // The driver allocated each endpoint with the bMaxBurst and wBytesPerInterval of its
        // companion descriptor.
        assert_eq!(
            superspeed_fields(usb, [0x01, 0x82, 0x83, 0x04]),
            [(15, 0), (15, 0), (0, 64), (0, 64)]
        );
    });

    with_config(superspeed(UsbDeviceSpeed::SuperPlus, UsbVersion::ThreeOne, 0), |usb| {
        let b = usb.inner.bos_descriptor;
        // BOS: wTotalLength = 42, bNumDeviceCaps = 3; SS then SSP capability.
        assert_eq!(
            (&b[2..5], &b[12..22], &b[22..]),
            (&[42, 0, 3][..], &SS_CAP[..], &SSP_CAP[..])
        );
    });
}

#[test]
fn test_superspeed_requests() {
    let qualifier = u16::from(descriptor_type::DEVICE_QUALIFIER) << 8;
    with_device_superspeed(|usb| {
        configure(usb);
        assert_eq!(
            request(usb, 0x80, Request::GET_DESCRIPTOR, qualifier, 0, 10),
            Response::Rejected
        );
        assert_eq!(request(usb, 0, Request::SET_SEL, 0, 0, 6), Response::Accepted);
        assert_eq!(request(usb, 0, Request::SET_SEL, 0, 0, 5), Response::Rejected);
        assert_eq!(request(usb, 0, Request::SET_ISOCH_DELAY, 40, 0, 0), Response::Accepted);
        let ltm = Request::FEATURE_DEVICE_LTM_ENABLE;
        assert_eq!(request(usb, 0, Request::SET_FEATURE, ltm, 0, 0), Response::Rejected);
        assert_eq!(request(usb, 0, Request::CLEAR_FEATURE, ltm, 0, 0), Response::Rejected);
    });
    with_device_fullspeed(|usb| {
        configure(usb);
        assert_eq!(request(usb, 0, Request::SET_SEL, 0, 0, 6), Response::Rejected);
        assert_eq!(request(usb, 0, Request::SET_ISOCH_DELAY, 40, 0, 0), Response::Rejected);
    });
}

fn allocated<'d, D: Driver<'d, Bus = TestBus>, const N: usize>(
    usb: &UsbDevice<'d, D>,
    endpoints: [u8; N],
) -> [EndpointConfig; N] {
    endpoints.map(|addr| usb.inner.bus.configs[TestBus::slot(addr.into())].unwrap())
}

/// `max_burst` and `bytes_per_interval` of the configuration the driver allocated each endpoint with.
fn superspeed_fields<const N: usize>(usb: &UsbDevice<'_, TestDriver>, endpoints: [u8; N]) -> [(u8, u32); N] {
    allocated(usb, endpoints).map(|config| {
        assert_eq!((config.max_streams, config.mult), (0, 0));
        (config.max_burst, config.bytes_per_interval)
    })
}

#[test]
fn test_superspeed_endpoint_config() {
    // Below SuperSpeed there are no companion descriptors and no SuperSpeed fields.
    let mut config = Config::new(0x1234, 0x5678);
    config.max_burst = 15;
    with_config(config, |usb| {
        // 63 bytes without the four 6-byte companion descriptors.
        assert_eq!(usb.inner.config_descriptor.len(), 63);
        assert_eq!(superspeed_fields(usb, [0x01, 0x82, 0x83, 0x04]), [(0, 0); 4]);
    });

    // Every way to allocate a bulk endpoint gives the driver the configured burst.
    for max_burst in [0, 7] {
        let (mut config_descriptor, mut bos_descriptor, mut control_buf) = ([0; 128], [0; 64], [0; 64]);
        let mut builder = Builder::new(
            TestDriver::default(),
            superspeed(UsbDeviceSpeed::Super, UsbVersion::Three, max_burst),
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut [],
            &mut control_buf,
        );
        let mut function = builder.function(0xff, 0, 0);
        let mut interface = function.interface();
        let mut alt = interface.alt_setting(0xff, 0, 0, None);
        alt.endpoint_bulk_out_double_buffered(Some(0x01.into()), 1024);
        alt.endpoint_bulk_in_double_buffered(Some(0x81.into()), 1024);
        let endpoint = alt.alloc_endpoint_out(EndpointType::Bulk, Some(0x02.into()), 1024, 0);
        alt.endpoint_descriptor(
            endpoint.info(),
            SynchronizationType::NoSynchronization,
            UsageType::DataEndpoint,
            &[],
        );
        drop(function);
        let usb = builder.build();
        let c = usb.inner.config_descriptor;
        // bMaxBurst of the three companion descriptors, each after a 7-byte endpoint descriptor.
        assert_eq!([c[35], c[48], c[61]], [max_burst; 3]);
        assert_eq!(superspeed_fields(&usb, [0x01, 0x81, 0x02]), [(max_burst, 0); 3]);
        let double_buffered = allocated(&usb, [0x01, 0x81, 0x02]).map(|config| config.double_buffered);
        assert_eq!(double_buffered, [true, true, false]);
    }
}

/// Has only the allocation methods of a driver without SuperSpeed, so the `_with_config` methods
/// are the defaults.
#[derive(Default)]
struct PlainDriver(TestDriver);

impl<'d> Driver<'d> for PlainDriver {
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
        self.0.alloc_endpoint_in(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<TestEndpoint, EndpointAllocError> {
        self.0
            .alloc_endpoint_out(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_bulk_in_double_buffered(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
    ) -> Result<TestEndpoint, EndpointAllocError> {
        let mut config = EndpointConfig::new(ep_addr, EndpointType::Bulk, max_packet_size, 0);
        config.double_buffered = true;
        self.0.alloc_endpoint_in_with_config(config)
    }

    fn alloc_endpoint_bulk_out_double_buffered(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
    ) -> Result<TestEndpoint, EndpointAllocError> {
        let mut config = EndpointConfig::new(ep_addr, EndpointType::Bulk, max_packet_size, 0);
        config.double_buffered = true;
        self.0.alloc_endpoint_out_with_config(config)
    }

    fn start(self, control_max_packet_size: u16) -> (TestBus, TestControlPipe) {
        self.0.start(control_max_packet_size)
    }
}

#[test]
fn test_default_endpoint_config() {
    // The defaults keep double buffering and drop the SuperSpeed fields.
    let (mut config_descriptor, mut bos_descriptor, mut control_buf) = ([0; 128], [0; 64], [0; 64]);
    let mut builder = Builder::new(
        PlainDriver::default(),
        superspeed(UsbDeviceSpeed::Super, UsbVersion::Three, 7),
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );
    let mut function = builder.function(0xff, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xff, 0, 0, None);
    alt.endpoint_bulk_out_double_buffered(Some(0x01.into()), 1024);
    alt.endpoint_bulk_in_double_buffered(Some(0x81.into()), 1024);
    alt.endpoint_bulk_out(Some(0x02.into()), 1024);
    drop(function);
    let usb = builder.build();
    let allocated = allocated(&usb, [0x01, 0x81, 0x02]).map(|config| (config.double_buffered, config.max_burst));
    assert_eq!(allocated, [(true, 0), (true, 0), (false, 0)]);
}

#[test]
#[should_panic(expected = "SuperSpeed USB requires bcd_usb")]
fn test_superspeed_requires_usb3() {
    with_config(superspeed(UsbDeviceSpeed::Super, UsbVersion::TwoOne, 0), |_| {});
}

#[test]
#[should_panic(expected = "SuperSpeedPlus USB requires bcd_usb")]
fn test_superspeed_plus_requires_usb31() {
    with_config(superspeed(UsbDeviceSpeed::SuperPlus, UsbVersion::Three, 0), |_| {});
}

#[test]
#[should_panic(expected = "max_burst")]
fn test_superspeed_max_burst_limit() {
    with_config(superspeed(UsbDeviceSpeed::Super, UsbVersion::Three, 16), |_| {});
}

#[test]
#[should_panic(expected = "SuperSpeed USB requires max_packet_size_0 = 9")]
fn test_superspeed_requires_max_packet_size_0_9() {
    let mut config = superspeed(UsbDeviceSpeed::Super, UsbVersion::Three, 0);
    config.max_packet_size_0 = 64;
    with_config(config, |_| {});
}

#[test]
fn test_superspeed_callbacks() {
    #[derive(Default)]
    struct State {
        sel: Cell<Option<(u8, u8, u16, u16)>>,
        isoch_delay: Cell<Option<u16>>,
        u1: Cell<Option<bool>>,
        ltm: Cell<Option<bool>>,
    }
    struct LinkHandler<'a>(&'a State);
    impl Handler for LinkHandler<'_> {
        fn set_sel(&mut self, u1_sel: u8, u1_pel: u8, u2_sel: u16, u2_pel: u16) {
            self.0.sel.set(Some((u1_sel, u1_pel, u2_sel, u2_pel)));
        }
        fn set_isoch_delay(&mut self, delay_ns: u16) {
            self.0.isoch_delay.set(Some(delay_ns));
        }
        fn u1_enabled(&mut self, enabled: bool) -> bool {
            self.0.u1.set(Some(enabled));
            true
        }
        fn ltm_enabled(&mut self, enabled: bool) -> bool {
            self.0.ltm.set(Some(enabled));
            true
        }
    }

    let state = State::default();
    let mut handler = LinkHandler(&state);
    let mut config_descriptor = [0; 64];
    let mut bos_descriptor = [0; 64];
    let mut control_buf = [0; 64];
    let mut config = superspeed(UsbDeviceSpeed::Super, UsbVersion::Three, 0);
    config.ltm_capable = true;
    let mut builder = Builder::new(
        TestDriver::default(),
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );
    builder.handler(&mut handler);
    let usb = &mut builder.build();
    configure(usb);
    assert_eq!(usb.inner.bos_descriptor[15], 0x02); // SS capability bmAttributes: LTM capable

    usb.control.out_data = [1, 2, 0x34, 0x12, 0x78, 0x56, 0, 0];
    assert_eq!(request(usb, 0, Request::SET_SEL, 0, 0, 6), Response::Accepted);
    assert_eq!(state.sel.get(), Some((1, 2, 0x1234, 0x5678)));
    assert_eq!(request(usb, 0, Request::SET_ISOCH_DELAY, 40, 0, 0), Response::Accepted);
    assert_eq!(state.isoch_delay.get(), Some(40));

    // The handler accepts U1 but leaves U2 at the default, which refuses it.
    let (u1, u2) = (Request::FEATURE_DEVICE_U1_ENABLE, Request::FEATURE_DEVICE_U2_ENABLE);
    assert_eq!(request(usb, 0, Request::SET_FEATURE, u1, 0, 0), Response::Accepted);
    assert_eq!(state.u1.get(), Some(true));
    assert_eq!(request(usb, 0, Request::SET_FEATURE, u2, 0, 0), Response::Rejected);
    let ltm = Request::FEATURE_DEVICE_LTM_ENABLE;
    assert_eq!(request(usb, 0, Request::SET_FEATURE, ltm, 0, 0), Response::Accepted);
    assert_eq!(state.ltm.get(), Some(true));
    assert_eq!(
        request(usb, 0x80, Request::GET_STATUS, 0, 0, 2),
        Response::Status(0x0014)
    );

    assert_eq!(request(usb, 0, Request::CLEAR_FEATURE, u1, 0, 0), Response::Accepted);
    assert_eq!(request(usb, 0, Request::CLEAR_FEATURE, u2, 0, 0), Response::Accepted);
    assert_eq!(request(usb, 0, Request::CLEAR_FEATURE, ltm, 0, 0), Response::Accepted);
    assert_eq!((state.u1.get(), state.ltm.get()), (Some(false), Some(false)));
    assert_eq!(request(usb, 0x80, Request::GET_STATUS, 0, 0, 2), Response::Status(0));

    // A bus reset clears U1.
    assert_eq!(request(usb, 0, Request::SET_FEATURE, u1, 0, 0), Response::Accepted);
    block_on(usb.inner.handle_bus_event(Event::Reset));
    assert_eq!(request(usb, 0x80, Request::GET_STATUS, 0, 0, 2), Response::Status(0));
}
