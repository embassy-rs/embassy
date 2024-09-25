use crate::can::frame::Header;

use super::message_ram::{DataLength, FrameFormat, HeaderElement, IdType, RxFifoElementHeader, TxBufferElementHeader};

fn make_id(id: u32, extended: bool) -> embedded_can::Id {
    if extended {
        embedded_can::Id::from(unsafe { embedded_can::ExtendedId::new_unchecked(id & 0x1FFFFFFF) })
    } else {
        // A standard identifier is stored into ID[28:18].
        embedded_can::Id::from(unsafe { embedded_can::StandardId::new_unchecked(((id >> 18) & 0x000007FF) as u16) })
    }
}

#[allow(dead_code)]
pub(crate) struct TxElementData {
    pub header: Header,
    pub data: [u8; 64],
    pub marker: u8,
    pub tx_event: bool,
}

impl TxElementData {
    //#[inline]
    //pub(crate) fn extract(element: &HeaderElement<TxBufferElementHeader>) -> Self {
    //    todo!()
    //}

    #[inline]
    pub fn put(&self, element: &mut HeaderElement<TxBufferElementHeader>) {
        let (id, id_type) = match self.header.id() {
            // A standard identifier has to be written to ID[28:18].
            embedded_can::Id::Standard(id) => ((id.as_raw() as u32) << 18, IdType::StandardId),
            embedded_can::Id::Extended(id) => (id.as_raw() as u32, IdType::ExtendedId),
        };

        let frame_format = if self.header.fdcan() {
            FrameFormat::Fdcan
        } else {
            FrameFormat::Classic
        };

        element.header.write(|w| {
            w.esi().bit(self.header.esi());
            w.xtd().set_id_type(id_type);
            w.rtr().bit(self.header.rtr());
            w.id().bits(id);
            w.mm().bits(self.marker);
            w.efc().bit(self.tx_event);
            w.fdf().bit(self.header.fdcan());
            w.brs().bit(self.header.bit_rate_switching());
            w.set_len(DataLength::new(self.header.len(), frame_format));
            w
        });

        // TODO validate endianness
        let u32_data: &[u32; 16] = unsafe { core::mem::transmute(&self.data) };
        element.data.iter_mut().zip(u32_data.iter()).for_each(|(reg, data)| {
            unsafe { reg.write(*data) };
        });
    }
}

#[allow(dead_code)]
pub(crate) struct RxElementData {
    pub header: Header,
    pub data: [u8; 64],
    /// If `dlc` is above what can be represented in the configured element,
    /// data can be clipped.
    /// This indicates what the real data word size (4 byte words) was.
    pub data_words: u8,
    pub timestamp: u16,
    /// If None, the frame was accepted without a filter as enabled by
    /// `GFC.ANFS` or `GFC.ANFE`.
    /// If Some, contains the index of the filter that was matched.
    pub filter_index: Option<u8>,
}

impl RxElementData {
    #[inline]
    pub fn extract(element: &HeaderElement<RxFifoElementHeader>) -> Self {
        let reg = element.header.read();

        let dlc = reg.to_data_length().len();

        let id = make_id(reg.id().bits(), reg.xtd().bits());
        let timestamp = reg.txts().bits;

        let rtr = reg.rtr().bits;
        let can_fd = reg.fdf().bits;
        let brs = reg.brs().bits;
        let esi = reg.esi().bits;

        let filter_index = if reg.anmf().bits { Some(reg.fidx().bits) } else { None };

        let mut data_u32 = [0u32; 16];
        data_u32
            .iter_mut()
            .zip(element.data.iter())
            .for_each(|(val, reg)| *val = reg.read());
        // TODO validate endianness
        let data: [u8; 64] = unsafe { core::mem::transmute(data_u32) };

        let data_words = element.data.len() as u8;

        Self {
            header: Header::new(id, dlc, rtr).set_can_fd(can_fd, brs).set_esi(esi),
            data_words,
            data,
            timestamp,
            filter_index,
        }
    }
}
