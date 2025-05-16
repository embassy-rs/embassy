//! CMSIS-DAP V2 class implementation.

use core::mem::MaybeUninit;

use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::StringIndex;
use crate::{msos, Builder, Handler};

/// State for the CMSIS-DAP v2 USB class.
pub struct State {
    control: MaybeUninit<Control>,
}

struct Control {
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

impl State {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
        }
    }
}

/// USB device class for CMSIS-DAP v2 probes.
pub struct CmsisDapV2Class<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    trace_ep: Option<D::EndpointIn>,
    max_packet_size: u16,
}

impl<'d, D: Driver<'d>> CmsisDapV2Class<'d, D> {
    /// Creates a new CmsisDapV2Class with the provided UsbBus and `max_packet_size` in bytes. For
    /// full-speed devices, `max_packet_size` has to be 64.
    ///
    /// The `trace` parameter enables the trace output endpoint. This is optional and can be
    /// disabled if the probe does not support trace output.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State, max_packet_size: u16, trace: bool) -> Self {
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
        let mut alt = interface.alt_setting(0xFF, 0, 0, Some(iface_string));
        let read_ep = alt.endpoint_bulk_out(max_packet_size);
        let write_ep = alt.endpoint_bulk_in(max_packet_size);
        let trace_ep = if trace {
            Some(alt.endpoint_bulk_in(max_packet_size))
        } else {
            None
        };
        drop(function);

        builder.handler(state.control.write(Control { iface_string }));

        CmsisDapV2Class {
            read_ep,
            write_ep,
            trace_ep,
            max_packet_size,
        }
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Write data to the host.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        for chunk in data.chunks(self.max_packet_size as usize) {
            self.write_ep.write(chunk).await?;
        }
        if data.len() % self.max_packet_size as usize == 0 {
            self.write_ep.write(&[]).await?;
        }
        Ok(())
    }

    /// Write data to the host via the trace output endpoint.
    ///
    /// Returns `EndpointError::Disabled` if the trace output endpoint is not enabled.
    pub async fn write_trace(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let Some(ep) = self.trace_ep.as_mut() else {
            return Err(EndpointError::Disabled);
        };

        for chunk in data.chunks(self.max_packet_size as usize) {
            ep.write(chunk).await?;
        }
        if data.len() % self.max_packet_size as usize == 0 {
            ep.write(&[]).await?;
        }
        Ok(())
    }

    /// Read data from the host.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        let mut n = 0;

        loop {
            let i = self.read_ep.read(&mut data[n..]).await?;
            n += i;
            if i < self.max_packet_size as usize {
                return Ok(n);
            }
        }
    }
}
