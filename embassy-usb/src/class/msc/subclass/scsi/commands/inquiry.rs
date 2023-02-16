// use super::control::Control;

// #[bitfield(bytes = 6)]
// pub struct InquiryCommand {
//     /// Always 0x12
//     pub op_code: B8,
//     #[skip]
//     __: B7,
//     /// If set, return vital data related to the page_code field
//     pub enable_vital_product_data: B1,
//     /// What kind of vital data to return
//     pub page_code: B8,
//     /// Amount of bytes allocation for data-in transfer
//     pub allocation_length: B16,
//     /// Control byte
//     pub control: Control,
// }

use super::Control;
use crate::class::msc::subclass::scsi::enums::{
    PeripheralDeviceType, PeripheralQualifier, ResponseDataFormat, SpcVersion, TargetPortGroupSupport,
};
use crate::packed::BE;
use crate::{packed_enum, packed_struct};

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum VitalProductDataPage<u8> {
        SupportedVitalProductDataPages = 0x00,
        UnitSerialNumber = 0x80,
    }
}

packed_struct! {
    pub struct InquiryCommand<6> {
        #[offset = 0, size = 8]
        op_code: u8,
        #[offset = 1*8, size = 1]
        enable_vital_product_data: bool,
        #[offset = 2*8, size = 8]
        page_code: VitalProductDataPage,
        #[offset = 3*8, size = 16]
        allocation_length: BE<u16>,
        #[offset = 5*8, size = 8]
        control: Control<[u8; Control::SIZE]>,
    }
}

impl InquiryCommand<[u8; InquiryCommand::SIZE]> {
    pub const OPCODE: u8 = 0x12;
}

packed_struct! {
    /// Inquiry response can contain many extensions. We support only the minimum required 36 bytes.
    pub struct InquiryResponse<36> {
        #[offset = 0*8+0, size = 5]
        peripheral_device_type: PeripheralDeviceType,
        #[offset = 0*8+5, size = 3]
        peripheral_qualifier: PeripheralQualifier,
        /// A removable medium (RMB) bit set to zero indicates that the medium is not removable.
        /// A RMB bit set to one indicates that the medium is removable.
        #[offset = 1*8+7, size = 1]
        removable_medium: bool,
        /// Indicates the implemented version of the SPC standard
        #[offset = 2*8+0, size = 8]
        version: SpcVersion,
        /// The RESPONSE DATA FORMAT field indicates the format of the standard INQUIRY data and shall be set as shown in table 139.
        /// A RESPONSE DATA FORMAT field set to 2h indicates that the standard INQUIRY data is in the format defined in this standard.
        /// Response data format values less than 2h are obsolete. Response data format values greater than 2h are reserved.
        #[offset = 3*8+0, size = 4]
        response_data_format: ResponseDataFormat,
        /// A hierarchical support (HISUP) bit set to zero indicates the SCSI target device does not use the hierarchical addressing model to assign LUNs to logical units.
        /// A HISUP bit set to one indicates the SCSI target device uses the hierarchical addressing model to assign LUNs to logical units.
        #[offset = 3*8+4, size = 1]
        hierarchical_support: bool,
        /// The Normal ACA Supported (NORMACA) bit set to one indicates that the device server supports a NACA bit set to one in the CDB CONTROL byte and supports the ACA task attribute (see SAM-4).
        /// A NORMACA bit set to zero indicates that the device server does not support a NACA bit set to one and does not support the ACA task attribute.
        #[offset = 3*8+5, size = 1]
        normal_aca: bool,
        /// The ADDITIONAL LENGTH field indicates the length in bytes of the remaining standard INQUIRY data.
        /// The relationship between the ADDITIONAL LENGTH field and the CDB ALLOCATION LENGTH field is defined in 4.3.5.6.
        /// Set to total length in bytes minus 4
        #[offset = 4*8+0, size = 8]
        additional_length: u8,
        /// A PROTECT bit set to zero indicates that the logical unit does not support protection information. A PROTECT bit set to one indicates that the logical unit supports:
        /// - a) type 1 protection, type 2 protection, or type 3 protection (see SBC-3); or
        /// - b) logical block protection (see SSC-4).
        ///
        /// More information about the type of protection the logical unit supports is available in the SPT field (see 7.8.7).
        #[offset = 5*8+0, size = 1]
        protect: bool,
        /// A Third-Party Copy (3PC) bit set to one indicates that the SCSI target device contains a copy manager that is addressable through this logical unit.
        /// A 3 PC bit set to zero indicates that no copy manager is addressable through this logical unit.
        #[offset = 5*8+3, size = 1]
        third_party_copy: bool,
        /// The contents of the target port group support ( TPGS ) field (see table 143) indicate the support for asymmetric logical unit access (see 5.11).
        #[offset = 5*8+4, size = 2]
        target_port_group_support: TargetPortGroupSupport,
        /// An Access Controls Coordinator (ACC) bit set to one indicates that the SCSI target device contains an access controls coordinator (see 3.1.4) that is addressable through this logical unit.
        /// An ACC bit set to zero indicates that no access controls coordinator is addressable through this logical unit.
        /// If the SCSI target device contains an access controls coordinator that is addressable through any logical unit other than the ACCESS CONTROLS well known logical unit (see 8.3), then the ACC bit shall be set to one for LUN 0.
        #[offset = 5*8+6, size = 1]
        access_controls_coordinator: bool,
        /// An SCC Supported (SCCS) bit set to one indicates that the SCSI target device contains an embedded storage array controller component that is addressable through this logical unit.
        /// See SCC-2 for details about storage array controller devices. An SCCS bit set to zero indicates that no embedded storage array controller component is addressable through this logical unit.
        #[offset = 5*8+7, size = 1]
        scc_supported: bool,
        /// A Multi Port (MULTIP) bit set to one indicates that this is a multi-port (two or more ports) SCSI target device
        /// and conforms to the SCSI multi-port device requirements found in the applicable standards
        /// (e.g., SAM-4, a SCSI transport protocol standard and possibly provisions of a command standard).
        /// A MULTIP bit set to zero indicates that this SCSI target device has a single port and does not implement the multi-port requirements.
        #[offset = 6*8+4, size = 1]
        multi_port: bool,
        /// An Enclosure Services (ENCSERV) bit set to one indicates that the SCSI target device contains an embedded enclosure services component
        /// that is addressable through this logical unit. See SES-3 for details about enclosure services.
        /// An ENCSERV bit set to zero indicates that no embedded enclosure services component is addressable through this logical unit.
        #[offset = 6*8+6, size = 1]
        enclosure_services: bool,
        /// The T10 VENDOR IDENTIFICATION field contains eight bytes of left-aligned ASCII data (see 4.4.1) identifying the vendor of the logical unit.
        /// The T10 vendor identification shall be one assigned by INCITS.
        /// A list of assigned T10 vendor identifications is in Annex E and on the T10 web site (http://www.t10.org).
        #[offset = 8*8+0, size = 8*8]
        vendor_identification: [u8; 8],
        /// The PRODUCT IDENTIFICATION field contains sixteen bytes of left-aligned ASCII data (see 4.4.1) defined by the vendor.
        #[offset = 16*8+0, size = 16*8]
        product_identification: [u8; 16],
        /// The PRODUCT REVISION LEVEL field contains four bytes of left-aligned ASCII data defined by the vendor.
        #[offset = 32*8+0, size = 4*8]
        product_revision_level: [u8; 4],
    }
}

packed_struct! {
    pub struct SupportedVitalProductDataPages<4> {
        #[offset = 0*8+0, size = 5]
        peripheral_device_type: PeripheralDeviceType,
        #[offset = 0*8+5, size = 3]
        peripheral_qualifier: PeripheralQualifier,
        #[offset = 1*8, size = 8]
        page_code: VitalProductDataPage,
        #[offset = 3*8, size = 8]
        page_length: u8,
    }
}

packed_struct! {
    pub struct UnitSerialNumberPage<4> {
        #[offset = 0*8+0, size = 5]
        peripheral_device_type: PeripheralDeviceType,
        #[offset = 0*8+5, size = 3]
        peripheral_qualifier: PeripheralQualifier,
        #[offset = 1*8, size = 8]
        page_code: VitalProductDataPage,
        #[offset = 3*8, size = 8]
        page_length: u8,
    }
}
