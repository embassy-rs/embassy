use core::slice;

use volatile_register::RW;

use crate::can::frame::Header;

use super::message_ram::{DataLength, Event, FrameFormat, IdType, RxFifoElementHeader, TxBufferElementHeader};

fn make_id(id: u32, extended: bool) -> embedded_can::Id {
    if extended {
        embedded_can::Id::from(unsafe { embedded_can::ExtendedId::new_unchecked(id & 0x1FFFFFFF) })
    } else {
        // A standard identifier is stored into ID[28:18].
        embedded_can::Id::from(unsafe { embedded_can::StandardId::new_unchecked(((id >> 18) & 0x000007FF) as u16) })
    }
}

pub(crate) fn put_tx_header(reg: &mut TxBufferElementHeader, header: &Header) {
    let (id, id_type) = match header.id() {
        // A standard identifier has to be written to ID[28:18].
        embedded_can::Id::Standard(id) => ((id.as_raw() as u32) << 18, IdType::StandardId),
        embedded_can::Id::Extended(id) => (id.as_raw() as u32, IdType::ExtendedId),
    };

    // Use FDCAN only for DLC > 8. FDCAN users can revise this if required.
    let frame_format = if header.len() > 8 || header.fdcan() {
        FrameFormat::Fdcan
    } else {
        FrameFormat::Classic
    };
    let brs = (frame_format == FrameFormat::Fdcan) && header.bit_rate_switching();

    reg.write(|w| {
        unsafe { w.id().bits(id) }
            .rtr()
            .bit(header.len() == 0 && header.rtr())
            .xtd()
            .set_id_type(id_type)
            .set_len(DataLength::new(header.len(), frame_format))
            .set_event(Event::NoEvent)
            .fdf()
            .set_format(frame_format)
            .brs()
            .bit(brs)
        //esi.set_error_indicator(//TODO//)
    });
}

pub(crate) fn put_tx_data(mailbox_data: &mut [RW<u32>], buffer: &[u8]) {
    let mut lbuffer = [0_u32; 16];
    let len = buffer.len();
    let data = unsafe { slice::from_raw_parts_mut(lbuffer.as_mut_ptr() as *mut u8, len) };
    data[..len].copy_from_slice(&buffer[..len]);
    let data_len = ((len) + 3) / 4;
    for (register, byte) in mailbox_data.iter_mut().zip(lbuffer[..data_len].iter()) {
        unsafe { register.write(*byte) };
    }
}

pub(crate) fn data_from_fifo(buffer: &mut [u8], mailbox_data: &[RW<u32>], len: usize) {
    for (i, register) in mailbox_data.iter().enumerate() {
        let register_value = register.read();
        let register_bytes = unsafe { slice::from_raw_parts(&register_value as *const u32 as *const u8, 4) };
        let num_bytes = (len) - i * 4;
        if num_bytes <= 4 {
            buffer[i * 4..i * 4 + num_bytes].copy_from_slice(&register_bytes[..num_bytes]);
            break;
        }
        buffer[i * 4..(i + 1) * 4].copy_from_slice(register_bytes);
    }
}

pub(crate) fn data_from_tx_buffer(buffer: &mut [u8], mailbox_data: &[RW<u32>], len: usize) {
    for (i, register) in mailbox_data.iter().enumerate() {
        let register_value = register.read();
        let register_bytes = unsafe { slice::from_raw_parts(&register_value as *const u32 as *const u8, 4) };
        let num_bytes = (len) - i * 4;
        if num_bytes <= 4 {
            buffer[i * 4..i * 4 + num_bytes].copy_from_slice(&register_bytes[..num_bytes]);
            break;
        }
        buffer[i * 4..(i + 1) * 4].copy_from_slice(register_bytes);
    }
}

pub(crate) fn extract_frame(
    mailbox_header: &RxFifoElementHeader,
    mailbox_data: &[RW<u32>],
    buffer: &mut [u8],
) -> Option<(Header, u16)> {
    let header_reg = mailbox_header.read();

    let id = make_id(header_reg.id().bits(), header_reg.xtd().bits());
    let dlc = header_reg.to_data_length().len();
    let len = dlc as usize;
    let timestamp = header_reg.txts().bits;
    if len > buffer.len() {
        return None;
    }
    data_from_fifo(buffer, mailbox_data, len);
    let header = if header_reg.fdf().bits {
        Header::new_fd(id, dlc, header_reg.rtr().bits(), header_reg.brs().bits())
    } else {
        Header::new(id, dlc, header_reg.rtr().bits())
    };
    Some((header, timestamp))
}
