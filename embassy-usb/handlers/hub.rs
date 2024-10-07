//! Host driver for USB hubs.
//!
//! It has it's own enumerate implementation to deal with the deferred `bus_reset` and state/speed detection.
//! It requires the usb-driver to implement/support `Interrupt` `ChannelIn` endpoints (which resolves a call to `[ChannelIn::read]`).

use crate::host::{ConfigurationDescriptor, InterfaceDescriptor};

use super::UsbHostHandler;
use embassy_usb_driver::host::{EndpointDescriptor, USBHostDriverTrait};

pub struct HubHandler<H: USBHostDriverTrait> {
    interrupt_channel: H::ChannelIn,
}

impl<H: USBHostDriverTrait> UsbHostHandler for HubHandler<H: USBHostDriverTrait> {
    const fn static_spec() -> super::StaticHandlerSpec {
        super::StaticHandlerSpec {
            device: Some(super::DeviceFilter {
                base_class: Some(0x09), // Hub
                sub_class: Some(0x00),
                protocol: None, // 00 for FS, otherwise HS or higher
            }),
        }
    }

    async fn try_register(
        bus: &mut crate::host::UsbHost<H>,
        device_address: u8,
        configuration: &ConfigurationDescriptor,
    ) -> Result<Self, ()> {
        // TODO: in order to configure a driver the behind the root port we'll need to allow for device_address in channel configs
        //  alternatively we can have every hub contain multiple ports each implementing USBHostDriverTrait,
        //  this still requires changin the trait to set_recipient but effectively removes the requirement for a seperate device tree.
        //  ideally the architecture works for both buffer-dma, and scatter-gather dma; seems like most hw has a fixed channel pool.
        //  however most do support TDM just using set_recipient; it would be cool to at least allow for TDM
        //  maybe TDM can be done using an escape-hatch with just the resume info; e.g. UsbHandlerResumeInfo (can be generic),
        //  which contains just the info in order to resume a suspended handler;
        //  this allows for the driver to be temporarily dropped, thus allowing another driver to be registered in it's place
        //  this even allows for certain pipes to remain claimed (by moving them to the resumeinfo)
        // UPDATE: `UsbResumableHandler` convers the TDM, the main goal now is to allow channels to be addressable

        // TODO: retrieve interface endpointdescriptor; find interrupt ep

        // Dedicated control pipe for
        let control_pipe = bus.claim_endpoint(
            &EndpointDescriptor {
                len: 8,
                descriptor_type: 0,
                endpoint_address: 0x80,
                attributes: 0b00,
                max_packet_size: 8,
                interval: 0,
            },
            device_address,
        );

        let interrupt_ch = bus.claim_endpoint(&None, device_address)?;

        // bus.set_configuration())
        Ok(HubHandler {})
    }
}
