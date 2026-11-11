//! CMSIS-DAP V2 class implementation.

use core::mem::MaybeUninit;

use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, EndpointOptions, Handler, InterfaceHandler, msos};

/// Configuration for [`CmsisDapV2Class`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Whether to add the trace output endpoint. This is optional and can be disabled if the
    /// probe does not support trace output.
    pub trace: bool,
    /// Options for the bulk OUT endpoint, which receives the commands the host sends.
    pub out_endpoint_options: EndpointOptions,
    /// Options for the bulk IN endpoint, which sends the responses to the host.
    pub in_endpoint_options: EndpointOptions,
    /// Options for the trace output bulk IN endpoint.
    pub trace_endpoint_options: EndpointOptions,
}

impl Config {
    /// Create a configuration without trace output and with default endpoint options.
    pub const fn new() -> Self {
        Self {
            trace: false,
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
            trace_endpoint_options: EndpointOptions::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the CMSIS-DAP v2 USB class.
pub struct State {
    control: MaybeUninit<Control>,
}

struct Control {
    iface: InterfaceNumber,
    iface_string: StringIndex,
}

impl Handler for Control {
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        if index == self.iface_string {
            Some("CMSIS-DAP v2 Interface")
        } else {
            warn!("unknown string index requested");
            None
        }
    }
}

impl InterfaceHandler for Control {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.iface
    }
}

impl State {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// USB device class for CMSIS-DAP v2 probes.
pub struct CmsisDapV2Class<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    trace_ep: Option<D::EndpointIn>,
}

impl<'d, D: Driver<'d>> CmsisDapV2Class<'d, D> {
    /// Creates a new CmsisDapV2Class with the provided configuration.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State, config: Config) -> Self {
        // DAP - Custom Class 0
        let iface_string = builder.string();
        let mut function = builder.function(0xFF, 0, 0);
        function.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
        function.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            // CMSIS-DAP standard GUID, from https://arm-software.github.io/CMSIS_5/DAP/html/group__DAP__ConfigUSB__gr.html
            msos::PropertyData::RegMultiSz(&["{CDB3B5AD-293B-4663-AA36-1AAE46463776}"]),
        ));
        let mut interface = function.interface();
        let interface_number = interface.interface_number();
        let mut alt = interface.alt_setting(0xFF, 0, 0, Some(iface_string));
        let read_ep = alt.endpoint_bulk_out(None, config.out_endpoint_options);
        let write_ep = alt.endpoint_bulk_in(None, config.in_endpoint_options);
        let trace_ep = if config.trace {
            Some(alt.endpoint_bulk_in(None, config.trace_endpoint_options))
        } else {
            None
        };
        drop(function);

        builder.interface_handler(state.control.write(Control {
            iface: interface_number,
            iface_string,
        }));

        CmsisDapV2Class {
            read_ep,
            write_ep,
            trace_ep,
        }
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Write data to the host.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write_transfer(data, true).await
    }

    /// Write data to the host via the trace output endpoint.
    ///
    /// Returns `EndpointError::Disabled` if the trace output endpoint is not enabled.
    pub async fn write_trace(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let Some(ep) = self.trace_ep.as_mut() else {
            return Err(EndpointError::Disabled);
        };
        ep.write_transfer(data, true).await
    }

    /// Read data from the host.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read_transfer(data).await
    }
}
