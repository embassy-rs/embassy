use super::super::enums::PeripheralQualifier;
use crate::class::msc::subclass::scsi::enums::PeripheralDeviceType;
use crate::gen_packet;

gen_packet! {
    /// Inquiry response can contain many extensions. We support only the minimum required 36 bytes.
    pub struct InquiryResponse<36> {
        #[offset = 0, size = 5]
        peripheral_qualifier: PeripheralQualifier,
        #[offset = 5, size = 3]
        peripheral_device_type: PeripheralDeviceType,
    }
}

fn test() {
    let packet = InquiryResponse::new();
}
