//! Implementations of well-known USB classes.
//!
//! Each class is gated by a `class-<name>` Cargo feature. The device and host
//! submodules are additionally gated by the `device` and `host` roles, so a
//! class can be compiled for either side or both.

#[cfg(feature = "class-bth")]
pub mod bth;
#[cfg(feature = "class-ccid")]
pub mod ccid;
#[cfg(feature = "class-cdc-acm")]
pub mod cdc_acm;
#[cfg(feature = "class-cdc-ecm")]
pub mod cdc_ecm;
#[cfg(feature = "class-cdc-ncm")]
pub mod cdc_ncm {
    //! CDC NCM (Ethernet over USB) class.
    #[cfg(feature = "device")]
    pub mod device;
}
#[cfg(feature = "class-cmsis-dap-v2")]
pub mod cmsis_dap_v2 {
    //! CMSIS-DAP v2 class.
    #[cfg(feature = "device")]
    pub mod device;
}
#[cfg(feature = "class-dfu")]
pub mod dfu {
    //! Device Firmware Upgrade (DFU) class.
    #[cfg(feature = "device")]
    pub mod device;
}
#[cfg(feature = "class-gip")]
pub mod gip {
    //! Xbox Gaming Input Protocol (GIP) class.
    #[cfg(feature = "host")]
    pub mod host;
}
#[cfg(feature = "class-gud")]
pub mod gud {
    //! GUD (Generic USB Display) class.
    #[cfg(feature = "device")]
    pub mod device;
}
#[cfg(feature = "class-hid")]
pub mod hid;
#[cfg(feature = "class-hub")]
pub mod hub {
    //! USB hub class.
    #[cfg(feature = "host")]
    pub mod host;
}
#[cfg(feature = "class-midi")]
pub mod midi;
#[cfg(feature = "class-msc")]
pub mod msc;
#[cfg(feature = "class-mtp")]
pub mod mtp;
#[cfg(feature = "class-printer")]
pub mod printer;
#[cfg(feature = "class-uac")]
pub mod uac;
#[cfg(feature = "class-usbtmc")]
pub mod usbtmc;
#[cfg(feature = "class-uvc")]
pub mod uvc {
    //! USB Video Class (UVC).
    #[cfg(feature = "device")]
    pub mod device;
}
#[cfg(feature = "class-vcp")]
pub mod vcp {
    //! Vendor-specific serial (VCP) adapters.
    #[cfg(feature = "host")]
    pub mod host;
}
#[cfg(feature = "class-vendor")]
pub mod vendor;
#[cfg(feature = "class-web-usb")]
pub mod web_usb {
    //! WebUSB class.
    #[cfg(feature = "device")]
    pub mod device;
}

// Classes that can be built for both roles need at least one role enabled.
// Device-only and host-only class features already imply their role.
#[cfg(all(feature = "class-ccid", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-ccid` requires at least one of `device` or `host`");
#[cfg(all(feature = "class-cdc-acm", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-cdc-acm` requires at least one of `device` or `host`");
#[cfg(all(feature = "class-cdc-ecm", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-cdc-ecm` requires at least one of `device` or `host`");
#[cfg(all(feature = "class-hid", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-hid` requires at least one of `device` or `host`");
#[cfg(all(feature = "class-midi", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-midi` requires at least one of `device` or `host`");
#[cfg(all(feature = "class-msc", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-msc` requires at least one of `device` or `host`");
#[cfg(all(feature = "class-printer", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-printer` requires at least one of `device` or `host`");
#[cfg(all(feature = "class-uac", not(any(feature = "device", feature = "host"))))]
compile_error!("feature `class-uac` requires at least one of `device` or `host`");

#[cfg(feature = "device")]
#[allow(dead_code)]
pub(crate) async fn read_packet<E: crate::driver::EndpointOut>(
    ep: &mut E,
    buf: &mut [u8],
) -> Result<usize, crate::driver::EndpointError> {
    let cap = buf.len().min(ep.info().max_packet_size as usize);
    ep.read_transfer(&mut buf[..cap]).await
}

#[cfg(feature = "device")]
#[allow(dead_code)]
pub(crate) async fn write_packet<E: crate::driver::EndpointIn>(
    ep: &mut E,
    buf: &[u8],
) -> Result<(), crate::driver::EndpointError> {
    if buf.len() > ep.info().max_packet_size as usize {
        return Err(crate::driver::EndpointError::BufferOverflow);
    }
    ep.write_transfer(buf, false).await?;
    Ok(())
}

/// Writes one isochronous packet for the next service interval.
///
/// Reserves a slot via [`IsoEndpointIn::ready`](crate::driver::IsoEndpointIn::ready) and queues
/// `buf` synchronously via [`IsoInSlot::write`](crate::driver::IsoInSlot::write). Returns how
/// many service intervals were skipped before this slot, if any.
#[cfg(feature = "device")]
#[allow(dead_code)]
pub(crate) async fn write_iso_packet<E: crate::driver::IsoEndpointIn>(
    ep: &mut E,
    buf: &[u8],
) -> Result<Option<core::num::NonZeroU32>, crate::driver::IsoWriteError> {
    use crate::driver::IsoInSlot;

    if buf.len() > ep.info().max_packet_size as usize {
        return Err(crate::driver::IsoWriteError::BufferOverflow);
    }
    let slot = ep.ready().await?;
    let skipped = slot.skipped();
    slot.write(buf)?;
    Ok(skipped)
}

#[cfg(all(test, feature = "device"))]
mod tests {
    use core::num::NonZeroU32;

    use embassy_futures::block_on;

    use super::write_iso_packet;
    use crate::driver::{
        Endpoint, EndpointAddress, EndpointDisabledError, EndpointInfo, EndpointInterval, EndpointType, IsoEndpointIn,
        IsoInSlot, IsoWriteError,
    };

    struct IsoEndpoint {
        disabled: bool,
        too_late: bool,
        skipped: Option<NonZeroU32>,
        written: Option<usize>,
    }

    struct MockSlot<'a> {
        ep: &'a mut IsoEndpoint,
        skipped: Option<NonZeroU32>,
    }

    impl IsoInSlot for MockSlot<'_> {
        fn skipped(&self) -> Option<NonZeroU32> {
            self.skipped
        }

        fn write(self, data: &[u8]) -> Result<(), IsoWriteError> {
            if data.len() > usize::from(self.ep.info().max_packet_size) {
                return Err(IsoWriteError::BufferOverflow);
            }
            if self.ep.disabled {
                return Err(IsoWriteError::Disabled);
            }
            if self.ep.too_late {
                return Err(IsoWriteError::TooLate);
            }
            self.ep.written = Some(data.len());
            Ok(())
        }
    }

    impl Endpoint for IsoEndpoint {
        fn info(&self) -> EndpointInfo {
            EndpointInfo::new(
                EndpointAddress::from(0x81),
                EndpointType::Isochronous,
                8,
                EndpointInterval::from_ms(1),
            )
        }
        async fn wait_enabled(&mut self) {}
        fn is_enabled(&self) -> bool {
            !self.disabled
        }
        async fn set_stalled(&mut self, _: bool) {}
        fn is_stalled(&self) -> bool {
            false
        }
    }

    impl IsoEndpointIn for IsoEndpoint {
        type Slot<'a>
            = MockSlot<'a>
        where
            Self: 'a;

        fn queue_depth(&self) -> u8 {
            1
        }

        async fn ready(&mut self) -> Result<Self::Slot<'_>, EndpointDisabledError> {
            if self.disabled {
                return Err(EndpointDisabledError);
            }
            let skipped = self.skipped;
            Ok(MockSlot { ep: self, skipped })
        }
    }

    #[test]
    fn iso_packet_queues_one_packet_and_reports_skipped_intervals() {
        let mut ep = IsoEndpoint {
            disabled: false,
            too_late: false,
            skipped: NonZeroU32::new(2),
            written: None,
        };
        assert_eq!(block_on(write_iso_packet(&mut ep, &[0; 8])), Ok(NonZeroU32::new(2)));
        assert_eq!(ep.written, Some(8));
        // An empty packet is still a packet.
        ep.skipped = None;
        assert_eq!(block_on(write_iso_packet(&mut ep, &[])), Ok(None));
        assert_eq!(ep.written, Some(0));
    }

    #[test]
    fn iso_packet_larger_than_the_endpoint_is_refused_before_ready() {
        let mut ep = IsoEndpoint {
            disabled: false,
            too_late: false,
            skipped: None,
            written: None,
        };
        assert_eq!(
            block_on(write_iso_packet(&mut ep, &[0; 9])),
            Err(IsoWriteError::BufferOverflow)
        );
        assert_eq!(ep.written, None);
    }

    #[test]
    fn iso_packet_on_a_disabled_endpoint_returns_disabled() {
        let mut ep = IsoEndpoint {
            disabled: true,
            too_late: false,
            skipped: None,
            written: None,
        };
        assert_eq!(block_on(write_iso_packet(&mut ep, &[1])), Err(IsoWriteError::Disabled));
    }

    #[test]
    fn iso_packet_too_late_propagates_without_disabling() {
        let mut ep = IsoEndpoint {
            disabled: false,
            too_late: true,
            skipped: None,
            written: None,
        };
        assert_eq!(block_on(write_iso_packet(&mut ep, &[1])), Err(IsoWriteError::TooLate));
        assert_eq!(ep.written, None);
    }
}
