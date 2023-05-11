/// the payload of [`Evt`] for a command status event
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CsEvt {
    pub status: u8,
    pub num_cmd: u8,
    pub cmd_code: u16,
}
