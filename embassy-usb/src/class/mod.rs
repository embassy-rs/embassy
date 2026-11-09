//! Implementations of well-known USB classes.
pub mod ccid;
pub mod cdc_acm;
pub mod cdc_ncm {
    //! CDC NCM (Ethernet over USB) class.
    pub mod device;
}
pub mod cmsis_dap_v2 {
    //! CMSIS-DAP v2 class.
    pub mod device;
}
pub mod dfu {
    //! Device Firmware Upgrade (DFU) class.
    pub mod device;
}
pub mod gip {
    //! Xbox Gaming Input Protocol (GIP) class.
    pub mod host;
}
pub mod gud {
    //! GUD (Generic USB Display) class.
    pub mod device;
}
pub mod hid;
pub mod hub {
    //! USB hub class.
    pub mod host;
}
pub mod midi;
pub mod msc;
pub mod uac;
pub mod usbtmc;
pub mod uvc {
    //! USB Video Class (UVC).
    pub mod device;
}
pub mod vcp {
    //! Vendor-specific serial (VCP) adapters.
    pub mod host;
}
pub mod web_usb {
    //! WebUSB class.
    pub mod device;
}
