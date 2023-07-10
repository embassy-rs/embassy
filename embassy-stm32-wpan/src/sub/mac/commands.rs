use bit_field::BitField;

use super::opcodes::OpcodeM4ToM0;

pub trait MacCommand {
    type Response;
    const OPCODE: OpcodeM4ToM0;
    const SIZE: usize;

    fn copy_into_slice(&self, buf: &mut [u8]);
}

pub struct ResetRequest {
    /// MAC PIB attributes are set to their default values or not during reset
    pub set_default_pib: bool,
}

impl MacCommand for ResetRequest {
    type Response = ();
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeResetReq;
    const SIZE: usize = 4;

    fn copy_into_slice(&self, buf: &mut [u8]) {
        buf[0] = self.set_default_pib as u8;
    }
}

#[repr(C)]
pub struct SetRequest {
    pub pib_attribute_ptr: *const u8,
    pub pib_attribute: u8,
    pub stuffing: [u8; 3],
}

impl MacCommand for SetRequest {
    type Response = ();
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeSetReq;
    const SIZE: usize = 8;

    fn copy_into_slice(&self, buf: &mut [u8]) {
        let address = self.pib_attribute_ptr as usize;

        // 68 ff 2 20 6f

        let a = unsafe { core::slice::from_raw_parts(&self as *const _ as *const u8, Self::SIZE) };
        debug!("{:#04x}", a);

        unsafe { core::ptr::copy(self as *const _ as *const u8, buf as *mut _ as *mut u8, 8) };

        // buf[0] = self.pib_attribute_ptr as u8;
        // buf[1] = self.pib_attribute;
    }
}

pub struct AssociateRequest {
    pub channel_number: u8,
    pub channel_page: u8,
    pub coord_addr_mode: u8,
    pub capability_information: u8,
    pub coord_pan_id: [u8; 2],
    pub security_level: u8,
    pub key_id_mode: u8,
    pub key_source: [u8; 8],
    pub coord_address: MacAddress,
    pub key_index: u8,
}

impl MacCommand for AssociateRequest {
    const OPCODE: OpcodeM4ToM0 = OpcodeM4ToM0::MlmeAssociateReq;
    const SIZE: usize = 25;
    type Response = ();

    fn copy_into_slice(&self, buf: &mut [u8]) {
        let a = unsafe { core::slice::from_raw_parts(&self as *const _ as *const u8, core::mem::size_of::<Self>()) };

        buf[..a.len()].copy_from_slice(a);
    }
}

pub union MacAddress {
    pub short: [u8; 2],
    pub extended: [u8; 8],
}
