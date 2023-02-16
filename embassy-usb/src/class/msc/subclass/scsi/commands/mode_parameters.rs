use crate::class::msc::subclass::scsi::enums::{MediumType, Mrie};
use crate::packed::BE;
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
        Caching = 0x08,
        InformationalExceptionsControl = 0x1C,
        AllPages = 0x3F,
    }
}

packed_struct! {
    pub struct ModePageHeader<2> {
        #[offset = 0, size = 6]
        page_code: PageCode,
        #[offset = 1*8+0, size = 8]
        page_length: u8,
    }
}

packed_struct! {
    pub struct CachingModePage<18> {
        /// RCD (READ Cache Disable) bit:
        /// - `false` - SCSI READ commands may access the cache or the media.
        /// - `true` - SCSI READ commands must access the media. Data cannot come from the cache
        #[offset = 0*8+0, size = 1]
        read_cache_disable: bool,
        /// MF (Multiplication Factor) bit:
        /// - `false` - The Minimum PREFETCH and Maximum PREFETCH fields are interpreted as a number of logical blocks.
        /// - `true` - Specifies that the target shall interpret the minimum and maximum PREFETCH fields to be specified in terms of a scalar number which,
        /// when multiplied by the number of logical blocks to be transferred for the current command, yields the number of logical blocks for each
        /// of the respective types of PREFETCH.
        #[offset = 0*8+1, size = 1]
        multiplication_factor: bool,
        /// WCE (Write Cache Enable) bit:
        /// - `false` - SCSI WRITE commands may not return status and completion message bytes until all data has been written to the media.
        /// - `true` - SCSI WRITE commands may return status and completion message bytes as soon as all data has been received from the host.
        #[offset = 0*8+2, size = 1]
        write_cache_enable: bool,
        /// SIZE (Size Enable) bit:
        /// - `false` - Initiator requests that the Number of Cache Segments is to be used to control caching segmentation.
        /// - `true` - Indicates that the Cache Segment Size is to be used to control caching segmentation.
        #[offset = 0*8+3, size = 1]
        size_enable: bool,
        /// DISC (Discontinuity) bit:
        /// - `false` - When set to zero, the DISC requests that prefetches be truncated at time discontinuities.
        /// - `true` - The Discontinuity (DISC) bit, when set to one, requests that the SCSI device continue the PREFETCH across time discontinuities, such as
        /// across cylinders or tracks up to the limits of the buffer, or segment, space available for PREFETCH.
        #[offset = 0*8+4, size = 1]
        discontinuity: bool,
        /// CAP (Caching Analysis Permitted) bit:
        /// - `false` - A zero indicates caching analysis is disabled. Caching analysis results are placed in the SCSI logging information table.
        /// See individual drive’s Product Manual, Volume 1, SCSI Bus Conditions and Miscellaneous Features Supported table.
        /// - `true` - The Caching Analysis Permitted (CAP) bit, when set to one, enables caching analysis.
        #[offset = 0*8+5, size = 1]
        caching_analysis_permitted: bool,
        /// ABPF (Abort Prefetch) bit:
        /// - `false` - When set to zero, with the DRA bit equal to zero, the termination of any active PREFETCH is dependent upon Caching Page bytes 4
        /// through 11 and is operation and/or vendor-specific.
        /// - `true` - The ABORT PREFETCH (ABPF) bit, when set to one, with the DRA bit equal to zero, requests that the SCSI device abort the PREFETCH
        /// upon selection. The ABPF set to one takes precedence over the Minimum PREFETCH bytes.
        #[offset = 0*8+6, size = 1]
        abort_prefetch: bool,
        /// IC (Initiator Control) bit:
        /// - `false` - When IC is set to ZERO, ARLA is enabled. Since Seagate drives covered by this manual do not organize the cache according to size of segment, but rather by number of segments, this bit is used to enable or disable ARLA (in non-Seagate equipment, this might be used to
        /// designate cache size).
        /// - `true` - When the Initiator Control (IC) enable bit is set to one, adaptive read look-ahead (ARLA) is disabled.
        #[offset = 0*8+7, size = 1]
        initiator_control: bool,
        /// WRITE RETENTION PRIORITY. The cache replacement algorithm does distinguish between retention in the cache of host-requested data and
        /// PREFETCH data. Therefore, this half byte is always 0.
        #[offset = 1*8+0, size = 4]
        write_retention_priority: u8,
        /// DEMAND READ RETENTION PRIORITY. The cache replacement algorithm does not distinguish between retention in the cache of
        /// host-requested data and PREFETCH data. Therefore, this half byte is always 0.
        #[offset = 1*8+4, size = 4]
        demand_read_retention_priority: u8,
        /// DISABLE PREFETCH TRANSFER LENGTH. PREFETCH is disabled for any SCSI READ command whose requested transfer length exceeds this value.
        #[offset = 2*8+0, size = 16]
        disable_prefetch_transfer_length: BE<u16>,
        /// The MINIMUM PREFETCH specifies the minimum number sectors to prefetch, regardless of the delay it may cause to other commands.
        #[offset = 4*8+0, size = 16]
        minimum_prefetch: BE<u16>,
        /// The MAXIMUM PREFETCH specifies the maximum number of logical blocks that may be prefetched. The PREFETCH operation may be aborted
        /// before the MAXIMUM PREFETCH value is reached, but only if the MINIMUM PREFETCH value has been satisfied.
        #[offset = 6*8+0, size = 16]
        maximum_prefetch: BE<u16>,
        /// The MAXIMUM PREFETCH Ceiling specifies an upper limit on the number of logical blocks computed as the maximum prefetch.
        /// If the MAXIMUM PREFETCH value is greater than the MAXIMUM PREFETCH CEILING, the value is truncated to the MAXIMUM PREFETCH CEILING value.
        #[offset = 8*8+0, size = 16]
        maximum_prefetch_ceiling: BE<u16>,
        /// NV_DIS bit:
        /// - `false` - An NV_DIS bit set to zero specifies that the device server may use a non-volatile cache and indicates that a non-volatile cache may be
        /// present and enabled.
        /// - `true` - An NV_DIS bit set to one specifies that the device server shall disable a non-volatile cache and indicates that a non-volatile cache is supported but disabled.
        #[offset = 10*8+0, size = 1]
        non_volatile_cache_disable: bool,
        /// The synchronize cache progress indication support (SYNC_PROG) field specifies device server progress indication reporting for the
        /// SYNCHRONIZE CACHE commands as defined in 3.52.
        #[offset = 10*8+1, size = 2]
        sync_prog: u8,
        /// DRA (Disable READ-Ahead) bit:
        /// - `false` - When the DRA bit equals zero, the target may continue to read logical blocks into the buffer beyond the addressed logical block(s).
        /// - `true` - The Disable READ-Ahead (DRA) bit, when set to one, requests that the target not read into the buffer any logical blocks beyond the
        /// addressed logical block(s).
        #[offset = 10*8+5, size = 1]
        disable_read_ahead: bool,
        /// LBCSS bit:
        /// - `false` - An LBCSS bit set to zero specifies that the CACHE SEGMENT SIZE field units shall be interpreted as bytes. The LBCSS shall not impact the
        /// units of other fields.
        /// - `true` - A logical block cache segment size (LBCSS) bit set to one specifies that the CACHE SEGMENT SIZE field units shall be interpreted as logical blocks.
        #[offset = 10*8+6, size = 1]
        logical_block_cache_segment_size: bool,
        /// FSW (FORCE SEQUENTIAL WRITE) bit:
        /// - `false` - When the FSW bit equals zero, the target is allowed to reorder the sequence of writing addressed logical blocks in order to achieve a
        /// faster command completion.
        /// - `true` - The Force Sequential Write (FSW) bit, when set to one, indicates that multiple block writes are to be transferred over the SCSI bus and
        /// written to the media in an ascending, sequential, logical block order.
        #[offset = 10*8+7, size = 1]
        fsw: bool,
        /// The NUMBER OF CACHE SEGMENTS byte gives the number of segments into which the host requests the drive divide the cache.
        #[offset = 11*8+0, size = 8]
        num_cache_segments: u8,
        /// The CACHE SEGMENT SIZE field indicates the requested segment size in bytes. This manual assumes that the Cache Segment Size field is valid
        /// only when the Size bit is one.
        #[offset = 12*8+0, size = 16]
        cache_segment_size: BE<u16>,
    }
}

packed_struct! {
    /// The Informational Exceptions Control mode page (see table 390) defines the methods used by the device server to control the
    /// reporting and the operations of specific informational exception conditions. This page shall only apply to informational
    /// exceptions that report an additional sense code of FAILURE PREDICTION THRESHOLD EXCEEDED or an additional sense code of
    /// WARNING to the application client. The mode page policy (see 5.4.14) for this mode page shall be shared, or per I_T nexus.
    ///
    /// Informational exception conditions occur as the result of vendor specific events within a logical unit. An informational exception
    /// condition may occur asynchronous to any commands issued by an application client.
    pub struct InformationalExceptionsControlModePage<10> {
        /// LOGERR (Log Error) bit:
        /// - `false` - If the log errors (LOGERR) bit is set to zero, the logging of informational exception conditions by a device server is vendor specific.
        /// - `true` - If the LOGERR bit is set to one, the device server shall log informational exception conditions.
        #[offset = 0*8+0, size = 1]
        log_error: bool,
        /// EBACKERR (enable background error) bit:
        /// - `false` - An enable background error (EBACKERR) bit set to zero indicates the target shall disable reporting of background self-test errors (SPC-5)
        /// and background scan errors (see SBC-4).
        /// - `true` - An EBACKERR bit set to one indicates reporting of background self-test errors and background scan errors shall be enabled. The method
        /// for reporting background self-test errors and background scan errors is determined by contents of the mrie field. Background self-test
        /// errors and background scan errors shall be reported as soon as the method specified in the mrie field occurs (i.e., the interval timer field
        /// and report count field do not apply for background self-test errors and background scan errors).
        #[offset = 0*8+1, size = 1]
        enable_background_error: bool,
        /// TEST bit:
        /// - `false` - A TEST bit set to zero shall instruct the device server not to generate any test device failure notifications.
        /// - `true` - If DEXCPT bit is set to zero and the TEST bit set to one, then the device server shall create a test device failure as specified by the MRIE
        /// field, INTERVAL TIMER field, and REPORT COUNT field (see table 390). The test device failure shall be reported with the additional sense
        /// code set to FAILURE PREDICTION THRESHOLD EXCEEDED (FALSE). If both the TEST bit and the DEXCPT bit are set to one, then the MODE
        /// SELECT command shall be terminated with CHECK CONDITION status, with the sense key set to ILLEGAL REQUEST, and the additional
        /// sense code set to INVALID FIELD IN PARAMETER LIST.
        #[offset = 0*8+2, size = 1]
        test: bool,
        /// DEXCPT (Disable Exception Control) bit:
        /// - `false` - A disable exception control (DEXCPT) bit set to zero indicates the failure prediction threshold exceeded reporting shall be enabled. The
        /// method for reporting the failure prediction threshold exceeded when the DEXCPT bit is set to zero is determined from the MRIE field.
        /// - `true` - A DEXCPT bit set to one indicates the device server shall disable reporting of the failure prediction threshold exceeded. The MRIE field is
        /// ignored when DEXCPT is set to one and EWASC is set to zero.
        #[offset = 0*8+3, size = 1]
        disable_exception_control: bool,
        /// EWASC (Enable Warning) bit:
        /// - `false` - If the enable warning (EWASC) bit is set to zero, the device server shall disable reporting of the warning. The MRIE field is ignored when
        /// DEXCPT is set to one and EWASC is set to zero.
        /// - `true` - If the EWASC bit is set to one, warning reporting shall be enabled. The method for reporting the warning when the EWASC bit is set to
        /// one is determined from the MRIE field.
        #[offset = 0*8+4, size = 1]
        enable_warning: bool,
        /// EBF (Enable Background Function) bit:
        /// - `false` - If the EBF bit is set to zero, the device server shall disable the functions. Background functions with separate enable control bits (e.g.,
        /// background medium scan defined in 4.3.7 are not controlled by this bit.
        /// - `true` - If background functions are supported and the Enable Background Function (EBF) bit is set to one, then the device server shall enable
        /// background functions.
        ///
        /// For the purposes of the EBF bit, background functions are defined as idle time functions that may impact performance that are performed by a
        /// device server operating without errors but do not impact the reliability of the logical unit (e.g., read scan).
        #[offset = 0*8+5, size = 1]
        enable_background_function: bool,
        /// PERF (Performance) bit:
        /// - `false` - If the performance (PERF) bit is set to zero, informational exception operations that are the cause of delays are acceptable.
        /// - `true` - If the PERF bit is set to one, the device server shall not cause delays while doing informational exception operations. A PERF bit set to
        /// one may cause the device server to disable some or all of the informational exceptions operations, thereby limiting the reporting of
        /// informational exception conditions.
        #[offset = 0*8+7, size = 1]
        performance: bool,
        /// The value in the method of reporting informational exceptions (MRIE) field (see table 390) defines the method that shall be used
        /// by the device server to report informational exception conditions. The priority of reporting multiple information exceptions is
        /// vendor specific.
        #[offset = 1*8+0, size = 4]
        mire: Mrie,
        /// The INTERVAL TIMER field specifies the period in 100 millisecond increments that the device server shall use for reporting that an informational
        /// exception condition has occurred (see table 392). After an informational exception condition has been reported, the interval timer shall be
        /// started. An INTERVAL TIMER field set to zero or FFFF_FFFFh specifies that the period for reporting an informational exception condition is
        /// vendor specific.
        #[offset = 2*8+0, size = 32]
        interval_timer: BE<u32>,
        /// The REPORT COUNT field specifies the maximum number of times the device server may report an informational exception condition to the
        /// application client. A REPORT COUNT field set to zero specifies that there is no limit on the number of times the device server may report an
        /// informational exception condition.
        #[offset = 6*8+0, size = 32]
        report_count: BE<u32>,
    }
}

pub enum ModeParameterWriterError {
    BufferTooSmall,
}

pub struct ModeParameter6Writer<'d> {
    buf: &'d mut [u8],
    offset: usize,
}

impl<'d> ModeParameter6Writer<'d> {
    pub fn new(buf: &'d mut [u8]) -> Result<Self, ModeParameterWriterError> {
        if buf.len() < ModeParameterHeader6::SIZE {
            Err(ModeParameterWriterError::BufferTooSmall)
        } else {
            Ok(Self {
                buf,
                offset: ModeParameterHeader6::SIZE,
            })
        }
    }

    pub fn write_page<'a>(
        &'a mut self,
        page_code: PageCode,
        size: usize,
    ) -> Result<&'a mut [u8], ModeParameterWriterError> {
        if self.buf.len() < self.offset + 2 + size {
            Err(ModeParameterWriterError::BufferTooSmall)
        } else {
            let mut page_header = ModePageHeader::from_bytes(&mut self.buf[self.offset..self.offset + 2]).unwrap();
            page_header.data.fill(0x00);
            page_header.set_page_code(page_code);
            page_header.set_page_length(size as u8);
            self.offset += 2;

            let page_data = &mut self.buf[self.offset..self.offset + size];
            self.offset += size;

            page_data.fill(0x00);
            Ok(page_data)
        }
    }

    pub fn page_size(&self) -> usize {
        self.offset - ModeParameterHeader6::SIZE
    }

    pub fn finalize(self) -> &'d [u8] {
        let mut header = ModeParameterHeader6::from_bytes(&mut self.buf[..ModeParameterHeader6::SIZE]).unwrap();
        header.data.fill(0x00);
        header.set_mode_data_length(self.offset as u8 - 1);
        &self.buf[..self.offset]
    }
}
