use crate::class::msc::subclass::scsi::enums::MediumType;
use crate::{packed_enum, packed_struct};

packed_struct! {
    pub struct ModeParameterHeader6<4> {
        /// When using the MODE SENSE command, the MODE DATA LENGTH field indicates the length in bytes of the following data that is available to be
        /// transferred. The mode data length does not include the number of bytes in the MODE DATA LENGTH field.
        ///
        /// When using the MODE SELECT command, this field is reserved.
        #[offset = 0, size = 8]
        mode_data_length: u8,
        #[offset = 1*8+0, size = 8]
        medium_type: MediumType,
        /// A DPOFUA bit set to zero indicates that the device server does not support the DPO and FUA bits.
        ///
        /// When used with the MODE SENSE command, a DPOFUA bit set to one indicates that the device server supports the DPO and FUA bits
        #[offset = 2*8+4, size = 1]
        dpofua: bool,
        /// A WP bit set to one indicates that the medium is write-protected. The medium may be write protected when the software write protect
        /// (SWP) bit in the Control mode page (see 5.3.12) is set to one or if another vendor specific mechanism causes the medium to be write protected.
        ///
        /// A WP bit set to zero indicates that the medium is not write-protected.
        #[offset = 2*8+7, size = 1]
        write_protect: bool,
        /// The BLOCK DESCRIPTOR LENGTH field contains the length in bytes of all the block descriptors. It is equal to the number of block descriptors
        /// times eight if the LONGLBA bit is set to zero or times sixteen if the LONGLBA bit is set to one, and does not include mode pages or vendor
        /// specific parameters (e.g., page code set to zero), if any, that may follow the last block descriptor. A block descriptor length of zero indicates that no
        /// block descriptors are included in the mode parameter list. This condition shall not be considered an error.
        #[offset = 3*8+0, size = 8]
        block_descriptor_length: u8,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PageCode<u8> {
        CachingModePage = 0x08,
    }
}

packed_struct! {
    pub struct CachingModePage<3> {
        #[offset = 0, size = 6]
        page_code: PageCode,
        #[offset = 1*8+0, size = 8]
        page_length: u8,
        #[offset = 2*8+0, size = 1]
        read_cache_disable: bool,
        #[offset = 2*8+2, size = 1]
        write_cache_enable: bool,
    }
}
