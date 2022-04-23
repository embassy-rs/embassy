use heapless::Vec;

use crate::{Interface, STRING_INDEX_CUSTOM_START};

use super::control::ControlHandler;
use super::descriptor::{BosWriter, DescriptorWriter};
use super::driver::{Driver, Endpoint};
use super::types::*;
use super::DeviceStateHandler;
use super::UsbDevice;
use super::MAX_INTERFACE_COUNT;

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

/// [`UsbDevice`] builder.
pub struct Builder<'d, D: Driver<'d>> {
    config: Config<'d>,
    handler: Option<&'d dyn DeviceStateHandler>,
    interfaces: Vec<Interface<'d>, MAX_INTERFACE_COUNT>,
    control_buf: &'d mut [u8],

    driver: D,
    next_string_index: u8,

    device_descriptor: DescriptorWriter<'d>,
    config_descriptor: DescriptorWriter<'d>,
    bos_descriptor: BosWriter<'d>,
}

impl<'d, D: Driver<'d>> Builder<'d, D> {
    /// Creates a builder for constructing a new [`UsbDevice`].
    ///
    /// `control_buf` is a buffer used for USB control request data. It should be sized
    /// large enough for the length of the largest control request (in or out)
    /// anticipated by any class added to the device.
    pub fn new(
        driver: D,
        config: Config<'d>,
        device_descriptor_buf: &'d mut [u8],
        config_descriptor_buf: &'d mut [u8],
        bos_descriptor_buf: &'d mut [u8],
        control_buf: &'d mut [u8],
        handler: Option<&'d dyn DeviceStateHandler>,
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

        device_descriptor.device(&config);
        config_descriptor.configuration(&config);
        bos_descriptor.bos();

        Builder {
            driver,
            handler,
            config,
            interfaces: Vec::new(),
            control_buf,
            next_string_index: STRING_INDEX_CUSTOM_START,

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
            self.driver,
            self.config,
            self.handler,
            self.device_descriptor.into_buf(),
            self.config_descriptor.into_buf(),
            self.bos_descriptor.writer.into_buf(),
            self.interfaces,
            self.control_buf,
        )
    }

    /// Returns the size of the control request data buffer. Can be used by
    /// classes to validate the buffer is large enough for their needs.
    pub fn control_buf_len(&self) -> usize {
        self.control_buf.len()
    }

    /// Add an USB function.
    ///
    /// If [`Config::composite_with_iads`] is set, this will add an IAD descriptor
    /// with the given class/subclass/protocol, associating all the child interfaces.
    ///
    /// If it's not set, no IAD descriptor is added.
    pub fn function(
        &mut self,
        class: u8,
        subclass: u8,
        protocol: u8,
    ) -> FunctionBuilder<'_, 'd, D> {
        let iface_count_index = if self.config.composite_with_iads {
            self.config_descriptor.iad(
                InterfaceNumber::new(self.interfaces.len() as _),
                0,
                class,
                subclass,
                protocol,
            );

            Some(self.config_descriptor.position() - 5)
        } else {
            None
        };

        FunctionBuilder {
            builder: self,
            iface_count_index,
        }
    }
}

/// Function builder.
///
/// A function is a logical grouping of interfaces that perform a given USB function.
/// If [`Config::composite_with_iads`] is set, each function will have an IAD descriptor.
/// If not, functions will not be visible as descriptors.
pub struct FunctionBuilder<'a, 'd, D: Driver<'d>> {
    builder: &'a mut Builder<'d, D>,
    iface_count_index: Option<usize>,
}

impl<'a, 'd, D: Driver<'d>> FunctionBuilder<'a, 'd, D> {
    /// Add an interface to the function.
    ///
    /// Interface numbers are guaranteed to be allocated consecutively, starting from 0.
    pub fn interface(&mut self) -> InterfaceBuilder<'_, 'd, D> {
        if let Some(i) = self.iface_count_index {
            self.builder.config_descriptor.buf[i] += 1;
        }

        let number = self.builder.interfaces.len() as _;
        let iface = Interface {
            handler: None,
            current_alt_setting: 0,
            num_alt_settings: 0,
            num_strings: 0,
        };

        if self.builder.interfaces.push(iface).is_err() {
            panic!("max interface count reached")
        }

        InterfaceBuilder {
            builder: self.builder,
            interface_number: InterfaceNumber::new(number),
            next_alt_setting_number: 0,
        }
    }
}

/// Interface builder.
pub struct InterfaceBuilder<'a, 'd, D: Driver<'d>> {
    builder: &'a mut Builder<'d, D>,
    interface_number: InterfaceNumber,
    next_alt_setting_number: u8,
}

impl<'a, 'd, D: Driver<'d>> InterfaceBuilder<'a, 'd, D> {
    /// Get the interface number.
    pub fn interface_number(&self) -> InterfaceNumber {
        self.interface_number
    }

    pub fn handler(&mut self, handler: &'d mut dyn ControlHandler) {
        self.builder.interfaces[self.interface_number.0 as usize].handler = Some(handler);
    }

    /// Allocates a new string index.
    pub fn string(&mut self) -> StringIndex {
        let index = self.builder.next_string_index;
        self.builder.next_string_index += 1;
        self.builder.interfaces[self.interface_number.0 as usize].num_strings += 1;

        StringIndex::new(index)
    }

    /// Add an alternate setting to the interface and write its descriptor.
    ///
    /// Alternate setting numbers are guaranteed to be allocated consecutively, starting from 0.
    ///
    /// The first alternate setting, with number 0, is the default one.
    pub fn alt_setting(
        &mut self,
        class: u8,
        subclass: u8,
        protocol: u8,
    ) -> InterfaceAltBuilder<'_, 'd, D> {
        let number = self.next_alt_setting_number;
        self.next_alt_setting_number += 1;
        self.builder.interfaces[self.interface_number.0 as usize].num_alt_settings += 1;

        self.builder.config_descriptor.interface_alt(
            self.interface_number,
            number,
            class,
            subclass,
            protocol,
            None,
        );

        InterfaceAltBuilder {
            builder: self.builder,
            interface_number: self.interface_number,
            alt_setting_number: number,
        }
    }
}

/// Interface alternate setting builder.
pub struct InterfaceAltBuilder<'a, 'd, D: Driver<'d>> {
    builder: &'a mut Builder<'d, D>,
    interface_number: InterfaceNumber,
    alt_setting_number: u8,
}

impl<'a, 'd, D: Driver<'d>> InterfaceAltBuilder<'a, 'd, D> {
    /// Get the interface number.
    pub fn interface_number(&self) -> InterfaceNumber {
        self.interface_number
    }

    /// Get the alternate setting number.
    pub fn alt_setting_number(&self) -> u8 {
        self.alt_setting_number
    }

    /// Add a custom descriptor to this alternate setting.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn descriptor(&mut self, descriptor_type: u8, descriptor: &[u8]) {
        self.builder
            .config_descriptor
            .write(descriptor_type, descriptor)
    }

    fn endpoint_in(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> D::EndpointIn {
        let ep = self
            .builder
            .driver
            .alloc_endpoint_in(ep_addr, ep_type, max_packet_size, interval)
            .expect("alloc_endpoint_in failed");

        self.builder.config_descriptor.endpoint(ep.info());

        ep
    }

    fn endpoint_out(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> D::EndpointOut {
        let ep = self
            .builder
            .driver
            .alloc_endpoint_out(ep_addr, ep_type, max_packet_size, interval)
            .expect("alloc_endpoint_out failed");

        self.builder.config_descriptor.endpoint(ep.info());

        ep
    }

    /// Allocate a BULK IN endpoint and write its descriptor.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn endpoint_bulk_in(&mut self, max_packet_size: u16) -> D::EndpointIn {
        self.endpoint_in(None, EndpointType::Bulk, max_packet_size, 0)
    }

    /// Allocate a BULK OUT endpoint and write its descriptor.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn endpoint_bulk_out(&mut self, max_packet_size: u16) -> D::EndpointOut {
        self.endpoint_out(None, EndpointType::Bulk, max_packet_size, 0)
    }

    /// Allocate a INTERRUPT IN endpoint and write its descriptor.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn endpoint_interrupt_in(&mut self, max_packet_size: u16, interval: u8) -> D::EndpointIn {
        self.endpoint_in(None, EndpointType::Interrupt, max_packet_size, interval)
    }

    /// Allocate a INTERRUPT OUT endpoint and write its descriptor.
    pub fn endpoint_interrupt_out(&mut self, max_packet_size: u16, interval: u8) -> D::EndpointOut {
        self.endpoint_out(None, EndpointType::Interrupt, max_packet_size, interval)
    }
}
