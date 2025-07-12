use heapless::Vec;

use crate::config::MAX_HANDLER_COUNT;
use crate::descriptor::{BosWriter, DescriptorWriter, SynchronizationType, UsageType};
use crate::driver::{Driver, Endpoint, EndpointInfo, EndpointType};
use crate::msos::{DeviceLevelDescriptor, FunctionLevelDescriptor, MsOsDescriptorWriter};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Handler, Interface, UsbDevice, MAX_INTERFACE_COUNT, STRING_INDEX_CUSTOM_START};

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
/// Allows Configuring the Bcd USB version below 2.1
pub enum UsbVersion {
    /// Usb version 2.0
    Two = 0x0200,
    /// Usb version 2.1
    TwoOne = 0x0210,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
/// Configuration used when creating [`UsbDevice`].
pub struct Config<'a> {
    pub(crate) vendor_id: u16,
    pub(crate) product_id: u16,

    /// Device BCD USB version.
    ///
    /// Default: `0x0210` ("2.1")
    pub bcd_usb: UsbVersion,

    /// Device class code assigned by USB.org. Set to `0xff` for vendor-specific
    /// devices that do not conform to any class.
    ///
    /// Default: `0xEF`
    /// See also: `composite_with_iads`
    pub device_class: u8,

    /// Device sub-class code. Depends on class.
    ///
    /// Default: `0x02`
    /// See also: `composite_with_iads`
    pub device_sub_class: u8,

    /// Device protocol code. Depends on class and sub-class.
    ///
    /// Default: `0x01`
    /// See also: `composite_with_iads`
    pub device_protocol: u8,

    /// Device release version in BCD.
    ///
    /// Default: `0x0010` ("0.1")
    pub device_release: u16,

    /// Maximum packet size in bytes for the control endpoint 0.
    ///
    /// Valid values depend on the speed at which the bus is enumerated.
    /// - low speed: 8
    /// - full speed: 8, 16, 32, or 64
    /// - high speed: 64
    ///
    /// Default: 64 bytes
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
    /// If set to `true` (default), the following fields should have the given values:
    ///
    /// - `device_class` = `0xEF`
    /// - `device_sub_class` = `0x02`
    /// - `device_protocol` = `0x01`
    ///
    /// If set to `false`, those fields must be set correctly for the classes that will be
    /// installed on the USB device.
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
    /// Create default configuration with the provided vid and pid values.
    pub const fn new(vid: u16, pid: u16) -> Self {
        Self {
            device_class: 0xEF,
            device_sub_class: 0x02,
            device_protocol: 0x01,
            max_packet_size_0: 64,
            vendor_id: vid,
            product_id: pid,
            device_release: 0x0010,
            bcd_usb: UsbVersion::TwoOne,
            manufacturer: None,
            product: None,
            serial_number: None,
            self_powered: false,
            supports_remote_wakeup: false,
            composite_with_iads: true,
            max_power: 100,
        }
    }
}

/// [`UsbDevice`] builder.
pub struct Builder<'d, D: Driver<'d>> {
    config: Config<'d>,
    handlers: Vec<&'d mut dyn Handler, MAX_HANDLER_COUNT>,
    interfaces: Vec<Interface, MAX_INTERFACE_COUNT>,
    control_buf: &'d mut [u8],

    driver: D,
    next_string_index: u8,

    config_descriptor: DescriptorWriter<'d>,
    bos_descriptor: BosWriter<'d>,

    msos_descriptor: MsOsDescriptorWriter<'d>,
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
        config_descriptor_buf: &'d mut [u8],
        bos_descriptor_buf: &'d mut [u8],
        msos_descriptor_buf: &'d mut [u8],
        control_buf: &'d mut [u8],
    ) -> Self {
        // Magic values specified in USB-IF ECN on IADs.
        if config.composite_with_iads
            && (config.device_class != 0xEF || config.device_sub_class != 0x02 || config.device_protocol != 0x01)
        {
            panic!("if composite_with_iads is set, you must set device_class = 0xEF, device_sub_class = 0x02, device_protocol = 0x01");
        }

        assert!(
            config.max_power <= 500,
            "The maximum allowed value for `max_power` is 500mA"
        );

        match config.max_packet_size_0 {
            8 | 16 | 32 | 64 => {}
            _ => panic!("invalid max_packet_size_0, the allowed values are 8, 16, 32 or 64"),
        }

        let mut config_descriptor = DescriptorWriter::new(config_descriptor_buf);
        let mut bos_descriptor = BosWriter::new(DescriptorWriter::new(bos_descriptor_buf));

        config_descriptor.configuration(&config);
        bos_descriptor.bos();

        Builder {
            driver,
            config,
            interfaces: Vec::new(),
            handlers: Vec::new(),
            control_buf,
            next_string_index: STRING_INDEX_CUSTOM_START,

            config_descriptor,
            bos_descriptor,

            msos_descriptor: MsOsDescriptorWriter::new(msos_descriptor_buf),
        }
    }

    /// Creates the [`UsbDevice`] instance with the configuration in this builder.
    pub fn build(mut self) -> UsbDevice<'d, D> {
        let msos_descriptor = self.msos_descriptor.build(&mut self.bos_descriptor);

        self.config_descriptor.end_configuration();
        self.bos_descriptor.end_bos();

        // Log the number of allocator bytes actually used in descriptor buffers
        trace!("USB: config_descriptor used: {}", self.config_descriptor.position());
        trace!("USB: bos_descriptor used: {}", self.bos_descriptor.writer.position());
        trace!("USB: msos_descriptor used: {}", msos_descriptor.len());
        trace!("USB: control_buf size: {}", self.control_buf.len());

        UsbDevice::build(
            self.driver,
            self.config,
            self.handlers,
            self.config_descriptor.into_buf(),
            self.bos_descriptor.writer.into_buf(),
            msos_descriptor,
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
    pub fn function(&mut self, class: u8, subclass: u8, protocol: u8) -> FunctionBuilder<'_, 'd, D> {
        let first_interface = InterfaceNumber::new(self.interfaces.len() as u8);
        let iface_count_index = if self.config.composite_with_iads {
            self.config_descriptor
                .iad(first_interface, 0, class, subclass, protocol);

            Some(self.config_descriptor.position() - 5)
        } else {
            None
        };

        FunctionBuilder {
            builder: self,
            iface_count_index,

            first_interface,
        }
    }

    /// Add a Handler.
    ///
    /// The Handler is called on some USB bus events, and to handle all control requests not already
    /// handled by the USB stack.
    pub fn handler(&mut self, handler: &'d mut dyn Handler) {
        assert!(
            self.handlers.push(handler).is_ok(),
            "embassy-usb: handler list full. Increase the `max_handler_count` compile-time setting. Current value: {}",
            MAX_HANDLER_COUNT
        );
    }

    /// Allocates a new string index.
    pub fn string(&mut self) -> StringIndex {
        let index = self.next_string_index;
        self.next_string_index += 1;
        StringIndex::new(index)
    }

    /// Add an MS OS 2.0 Descriptor Set.
    ///
    /// Panics if called more than once.
    pub fn msos_descriptor(&mut self, windows_version: u32, vendor_code: u8) {
        self.msos_descriptor.header(windows_version, vendor_code);
    }

    /// Add an MS OS 2.0 Device Level Feature Descriptor.
    pub fn msos_feature<T: DeviceLevelDescriptor>(&mut self, desc: T) {
        self.msos_descriptor.device_feature(desc);
    }

    /// Gets the underlying [`MsOsDescriptorWriter`] to allow adding subsets and features for classes that
    /// do not add their own.
    pub fn msos_writer(&mut self) -> &mut MsOsDescriptorWriter<'d> {
        &mut self.msos_descriptor
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

    first_interface: InterfaceNumber,
}

impl<'a, 'd, D: Driver<'d>> Drop for FunctionBuilder<'a, 'd, D> {
    fn drop(&mut self) {
        self.builder.msos_descriptor.end_function();
    }
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
            current_alt_setting: 0,
            num_alt_settings: 0,
        };

        assert!(self.builder.interfaces.push(iface).is_ok(),
            "embassy-usb: interface list full. Increase the `max_interface_count` compile-time setting. Current value: {}",
            MAX_INTERFACE_COUNT
        );

        InterfaceBuilder {
            builder: self.builder,
            interface_number: InterfaceNumber::new(number),
            next_alt_setting_number: 0,
        }
    }

    /// Add an MS OS 2.0 Function Level Feature Descriptor.
    pub fn msos_feature<T: FunctionLevelDescriptor>(&mut self, desc: T) {
        if !self.builder.msos_descriptor.is_in_config_subset() {
            self.builder.msos_descriptor.configuration(0);
        }

        if !self.builder.msos_descriptor.is_in_function_subset() {
            self.builder.msos_descriptor.function(self.first_interface);
        }

        self.builder.msos_descriptor.function_feature(desc);
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
    pub const fn interface_number(&self) -> InterfaceNumber {
        self.interface_number
    }

    /// Allocates a new string index.
    pub fn string(&mut self) -> StringIndex {
        self.builder.string()
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
        interface_string: Option<StringIndex>,
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
            interface_string,
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
    pub const fn interface_number(&self) -> InterfaceNumber {
        self.interface_number
    }

    /// Get the alternate setting number.
    pub const fn alt_setting_number(&self) -> u8 {
        self.alt_setting_number
    }

    /// Add a custom descriptor to this alternate setting.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn descriptor(&mut self, descriptor_type: u8, descriptor: &[u8]) {
        self.builder.config_descriptor.write(descriptor_type, descriptor, &[]);
    }

    /// Add a custom Binary Object Store (BOS) descriptor to this alternate setting.
    pub fn bos_capability(&mut self, capability_type: u8, capability: &[u8]) {
        self.builder.bos_descriptor.capability(capability_type, capability);
    }

    /// Write a custom endpoint descriptor for a certain endpoint.
    ///
    /// This can be necessary, if the endpoint descriptors can only be written
    /// after the endpoint was created. As an example, an endpoint descriptor
    /// may contain the address of an endpoint that was allocated earlier.
    pub fn endpoint_descriptor(
        &mut self,
        endpoint: &EndpointInfo,
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
        extra_fields: &[u8],
    ) {
        self.builder
            .config_descriptor
            .endpoint(endpoint, synchronization_type, usage_type, extra_fields);
    }

    /// Allocate an IN endpoint, without writing its descriptor.
    ///
    /// Used for granular control over the order of endpoint and descriptor creation.
    pub fn alloc_endpoint_in(&mut self, ep_type: EndpointType, max_packet_size: u16, interval_ms: u8) -> D::EndpointIn {
        let ep = self
            .builder
            .driver
            .alloc_endpoint_in(ep_type, max_packet_size, interval_ms)
            .expect("alloc_endpoint_in failed");

        ep
    }

    fn endpoint_in(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
        extra_fields: &[u8],
    ) -> D::EndpointIn {
        let ep = self.alloc_endpoint_in(ep_type, max_packet_size, interval_ms);
        self.endpoint_descriptor(ep.info(), synchronization_type, usage_type, extra_fields);

        ep
    }

    /// Allocate an OUT endpoint, without writing its descriptor.
    ///
    /// Use for granular control over the order of endpoint and descriptor creation.
    pub fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> D::EndpointOut {
        let ep = self
            .builder
            .driver
            .alloc_endpoint_out(ep_type, max_packet_size, interval_ms)
            .expect("alloc_endpoint_out failed");

        ep
    }

    fn endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
        extra_fields: &[u8],
    ) -> D::EndpointOut {
        let ep = self.alloc_endpoint_out(ep_type, max_packet_size, interval_ms);
        self.endpoint_descriptor(ep.info(), synchronization_type, usage_type, extra_fields);

        ep
    }

    /// Allocate a BULK IN endpoint and write its descriptor.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn endpoint_bulk_in(&mut self, max_packet_size: u16) -> D::EndpointIn {
        self.endpoint_in(
            EndpointType::Bulk,
            max_packet_size,
            0,
            SynchronizationType::NoSynchronization,
            UsageType::DataEndpoint,
            &[],
        )
    }

    /// Allocate a BULK OUT endpoint and write its descriptor.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn endpoint_bulk_out(&mut self, max_packet_size: u16) -> D::EndpointOut {
        self.endpoint_out(
            EndpointType::Bulk,
            max_packet_size,
            0,
            SynchronizationType::NoSynchronization,
            UsageType::DataEndpoint,
            &[],
        )
    }

    /// Allocate a INTERRUPT IN endpoint and write its descriptor.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn endpoint_interrupt_in(&mut self, max_packet_size: u16, interval_ms: u8) -> D::EndpointIn {
        self.endpoint_in(
            EndpointType::Interrupt,
            max_packet_size,
            interval_ms,
            SynchronizationType::NoSynchronization,
            UsageType::DataEndpoint,
            &[],
        )
    }

    /// Allocate a INTERRUPT OUT endpoint and write its descriptor.
    pub fn endpoint_interrupt_out(&mut self, max_packet_size: u16, interval_ms: u8) -> D::EndpointOut {
        self.endpoint_out(
            EndpointType::Interrupt,
            max_packet_size,
            interval_ms,
            SynchronizationType::NoSynchronization,
            UsageType::DataEndpoint,
            &[],
        )
    }

    /// Allocate a ISOCHRONOUS IN endpoint and write its descriptor.
    ///
    /// Descriptors are written in the order builder functions are called. Note that some
    /// classes care about the order.
    pub fn endpoint_isochronous_in(
        &mut self,
        max_packet_size: u16,
        interval_ms: u8,
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
        extra_fields: &[u8],
    ) -> D::EndpointIn {
        self.endpoint_in(
            EndpointType::Isochronous,
            max_packet_size,
            interval_ms,
            synchronization_type,
            usage_type,
            extra_fields,
        )
    }

    /// Allocate a ISOCHRONOUS OUT endpoint and write its descriptor.
    pub fn endpoint_isochronous_out(
        &mut self,
        max_packet_size: u16,
        interval_ms: u8,
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
        extra_fields: &[u8],
    ) -> D::EndpointOut {
        self.endpoint_out(
            EndpointType::Isochronous,
            max_packet_size,
            interval_ms,
            synchronization_type,
            usage_type,
            extra_fields,
        )
    }
}
