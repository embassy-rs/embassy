use super::descriptor::{BosWriter, DescriptorWriter};
use super::driver::{Driver, EndpointAllocError};
use super::types::*;
use super::UsbDevice;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config<'a> {
    pub(crate) vendor_id: u16,
    pub(crate) product_id: u16,

    /// Device class code assigned by USB.org. Set to `0xff` for vendor-specific
    /// devices that do not conform to any class.
    ///
    /// Default: `0x00` (class code specified by interfaces)
    pub device_class: u8,

    /// Device sub-class code. Depends on class.
    ///
    /// Default: `0x00`
    pub device_sub_class: u8,

    /// Device protocol code. Depends on class and sub-class.
    ///
    /// Default: `0x00`
    pub device_protocol: u8,

    /// Device release version in BCD.
    ///
    /// Default: `0x0010` ("0.1")
    pub device_release: u16,

    /// Maximum packet size in bytes for the control endpoint 0.
    ///
    /// Valid values are 8, 16, 32 and 64. There's generally no need to change this from the default
    /// value of 8 bytes unless a class uses control transfers for sending large amounts of data, in
    /// which case using a larger packet size may be more efficient.
    ///
    /// Default: 8 bytes
    pub max_packet_size_0: u8,

    /// Manufacturer name string descriptor.
    ///
    /// Default: (none)
    pub manufacturer: Option<&'a str>,

    /// Product name string descriptor.
    ///
    /// Default: (none)
    pub product: Option<&'a str>,

    /// Serial number string descriptor.
    ///
    /// Default: (none)
    pub serial_number: Option<&'a str>,

    /// Whether the device supports remotely waking up the host is requested.
    ///
    /// Default: `false`
    pub supports_remote_wakeup: bool,

    /// Configures the device as a composite device with interface association descriptors.
    ///
    /// If set to `true`, the following fields should have the given values:
    ///
    /// - `device_class` = `0xEF`
    /// - `device_sub_class` = `0x02`
    /// - `device_protocol` = `0x01`
    pub composite_with_iads: bool,

    /// Whether the device has its own power source.
    ///
    /// This should be set to `true` even if the device is sometimes self-powered and may not
    /// always draw power from the USB bus.
    ///
    /// Default: `false`
    ///
    /// See also: `max_power`
    pub self_powered: bool,

    /// Maximum current drawn from the USB bus by the device, in milliamps.
    ///
    /// The default is 100 mA. If your device always uses an external power source and never draws
    /// power from the USB bus, this can be set to 0.
    ///
    /// See also: `self_powered`
    ///
    /// Default: 100mA
    /// Max: 500mA
    pub max_power: u16,
}

impl<'a> Config<'a> {
    pub fn new(vid: u16, pid: u16) -> Self {
        Self {
            device_class: 0x00,
            device_sub_class: 0x00,
            device_protocol: 0x00,
            max_packet_size_0: 8,
            vendor_id: vid,
            product_id: pid,
            device_release: 0x0010,
            manufacturer: None,
            product: None,
            serial_number: None,
            self_powered: false,
            supports_remote_wakeup: false,
            composite_with_iads: false,
            max_power: 100,
        }
    }
}

/// Used to build new [`UsbDevice`]s.
pub struct UsbDeviceBuilder<'d, D: Driver<'d>> {
    config: Config<'d>,

    bus: D,
    next_interface_number: u8,
    next_string_index: u8,

    // TODO make not pub?
    pub device_descriptor: DescriptorWriter<'d>,
    pub config_descriptor: DescriptorWriter<'d>,
    pub bos_descriptor: BosWriter<'d>,
}

impl<'d, D: Driver<'d>> UsbDeviceBuilder<'d, D> {
    /// Creates a builder for constructing a new [`UsbDevice`].
    pub fn new(
        bus: D,
        config: Config<'d>,
        device_descriptor_buf: &'d mut [u8],
        config_descriptor_buf: &'d mut [u8],
        bos_descriptor_buf: &'d mut [u8],
    ) -> Self {
        // Magic values specified in USB-IF ECN on IADs.
        if config.composite_with_iads
            && (config.device_class != 0xEF
                || config.device_sub_class != 0x02
                || config.device_protocol != 0x01)
        {
            panic!("if composite_with_iads is set, you must set device_class = 0xEF, device_sub_class = 0x02, device_protocol = 0x01");
        }

        if config.max_power > 500 {
            panic!("The maximum allowed value for `max_power` is 500mA");
        }

        match config.max_packet_size_0 {
            8 | 16 | 32 | 64 => {}
            _ => panic!("invalid max_packet_size_0, the allowed values are 8, 16, 32 or 64"),
        }

        let mut device_descriptor = DescriptorWriter::new(device_descriptor_buf);
        let mut config_descriptor = DescriptorWriter::new(config_descriptor_buf);
        let mut bos_descriptor = BosWriter::new(DescriptorWriter::new(bos_descriptor_buf));

        device_descriptor.device(&config).unwrap();
        config_descriptor.configuration(&config).unwrap();
        bos_descriptor.bos().unwrap();

        UsbDeviceBuilder {
            bus,
            config,
            next_interface_number: 0,
            next_string_index: 4,

            device_descriptor,
            config_descriptor,
            bos_descriptor,
        }
    }

    /// Creates the [`UsbDevice`] instance with the configuration in this builder.
    pub fn build(mut self) -> UsbDevice<'d, D> {
        self.config_descriptor.end_configuration();
        self.bos_descriptor.end_bos();

        UsbDevice::build(
            self.bus,
            self.config,
            self.device_descriptor.into_buf(),
            self.config_descriptor.into_buf(),
            self.bos_descriptor.writer.into_buf(),
        )
    }

    /// Allocates a new interface number.
    pub fn alloc_interface(&mut self) -> InterfaceNumber {
        let number = self.next_interface_number;
        self.next_interface_number += 1;

        InterfaceNumber::new(number)
    }

    /// Allocates a new string index.
    pub fn alloc_string(&mut self) -> StringIndex {
        let index = self.next_string_index;
        self.next_string_index += 1;

        StringIndex::new(index)
    }

    /// Allocates an in endpoint.
    ///
    /// This directly delegates to [`Driver::alloc_endpoint_in`], so see that method for details. In most
    /// cases classes should call the endpoint type specific methods instead.
    pub fn alloc_endpoint_in(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<D::EndpointIn, EndpointAllocError> {
        self.bus
            .alloc_endpoint_in(ep_addr, ep_type, max_packet_size, interval)
    }

    /// Allocates an out endpoint.
    ///
    /// This directly delegates to [`Driver::alloc_endpoint_out`], so see that method for details. In most
    /// cases classes should call the endpoint type specific methods instead.
    pub fn alloc_endpoint_out(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<D::EndpointOut, EndpointAllocError> {
        self.bus
            .alloc_endpoint_out(ep_addr, ep_type, max_packet_size, interval)
    }

    /// Allocates a control in endpoint.
    ///
    /// This crate implements the control state machine only for endpoint 0. If classes want to
    /// support control requests in other endpoints, the state machine must be implemented manually.
    /// This should rarely be needed by classes.
    ///
    /// # Arguments
    ///
    /// * `max_packet_size` - Maximum packet size in bytes. Must be one of 8, 16, 32 or 64.
    ///
    /// # Panics
    ///
    /// Panics if endpoint allocation fails, because running out of endpoints or memory is not
    /// feasibly recoverable.
    #[inline]
    pub fn alloc_control_endpoint_in(&mut self, max_packet_size: u16) -> D::EndpointIn {
        self.alloc_endpoint_in(None, EndpointType::Control, max_packet_size, 0)
            .expect("alloc_ep failed")
    }

    /// Allocates a control out endpoint.
    ///
    /// This crate implements the control state machine only for endpoint 0. If classes want to
    /// support control requests in other endpoints, the state machine must be implemented manually.
    /// This should rarely be needed by classes.
    ///
    /// # Arguments
    ///
    /// * `max_packet_size` - Maximum packet size in bytes. Must be one of 8, 16, 32 or 64.
    ///
    /// # Panics
    ///
    /// Panics if endpoint allocation fails, because running out of endpoints or memory is not
    /// feasibly recoverable.
    #[inline]
    pub fn alloc_control_endpoint_out(&mut self, max_packet_size: u16) -> D::EndpointOut {
        self.alloc_endpoint_out(None, EndpointType::Control, max_packet_size, 0)
            .expect("alloc_ep failed")
    }

    /// Allocates a bulk in endpoint.
    ///
    /// # Arguments
    ///
    /// * `max_packet_size` - Maximum packet size in bytes. Must be one of 8, 16, 32 or 64.
    ///
    /// # Panics
    ///
    /// Panics if endpoint allocation fails, because running out of endpoints or memory is not
    /// feasibly recoverable.
    #[inline]
    pub fn alloc_bulk_endpoint_in(&mut self, max_packet_size: u16) -> D::EndpointIn {
        self.alloc_endpoint_in(None, EndpointType::Bulk, max_packet_size, 0)
            .expect("alloc_ep failed")
    }

    /// Allocates a bulk out endpoint.
    ///
    /// # Arguments
    ///
    /// * `max_packet_size` - Maximum packet size in bytes. Must be one of 8, 16, 32 or 64.
    ///
    /// # Panics
    ///
    /// Panics if endpoint allocation fails, because running out of endpoints or memory is not
    /// feasibly recoverable.
    #[inline]
    pub fn alloc_bulk_endpoint_out(&mut self, max_packet_size: u16) -> D::EndpointOut {
        self.alloc_endpoint_out(None, EndpointType::Bulk, max_packet_size, 0)
            .expect("alloc_ep failed")
    }

    /// Allocates a bulk in endpoint.
    ///
    /// # Arguments
    ///
    /// * `max_packet_size` - Maximum packet size in bytes. Cannot exceed 64 bytes.
    ///
    /// # Panics
    ///
    /// Panics if endpoint allocation fails, because running out of endpoints or memory is not
    /// feasibly recoverable.
    #[inline]
    pub fn alloc_interrupt_endpoint_in(
        &mut self,
        max_packet_size: u16,
        interval: u8,
    ) -> D::EndpointIn {
        self.alloc_endpoint_in(None, EndpointType::Interrupt, max_packet_size, interval)
            .expect("alloc_ep failed")
    }

    /// Allocates a bulk in endpoint.
    ///
    /// # Arguments
    ///
    /// * `max_packet_size` - Maximum packet size in bytes. Cannot exceed 64 bytes.
    ///
    /// # Panics
    ///
    /// Panics if endpoint allocation fails, because running out of endpoints or memory is not
    /// feasibly recoverable.
    #[inline]
    pub fn alloc_interrupt_endpoint_out(
        &mut self,
        max_packet_size: u16,
        interval: u8,
    ) -> D::EndpointOut {
        self.alloc_endpoint_out(None, EndpointType::Interrupt, max_packet_size, interval)
            .expect("alloc_ep failed")
    }
}
