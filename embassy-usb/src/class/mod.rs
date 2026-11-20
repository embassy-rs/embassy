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
