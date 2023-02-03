use crate::packed_enum;

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PeripheralQualifier<u8> {
        /// A peripheral device having the specified peripheral device type is connected to this logical unit. If the device server is unable to determine whether or not a peripheral device is connected, it also shall use this peripheral qualifier. This peripheral qualifier does not mean that the peripheral device connected to the logical unit is ready for access.
        Connected = 0b000,
        /// A peripheral device having the specified peripheral device type is not connected to this logical unit. However, the device server is capable of supporting the specified peripheral device type on this logical unit.
        NotConnected = 0b001,
        /// The device server is not capable of supporting a peripheral device on this logical unit. For this peripheral qualifier the peripheral device type shall be set to 1Fh. All other peripheral device type values are reserved for this peripheral qualifier.
        Incapable = 0b011,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PeripheralDeviceType<u8> {
        /// Direct access block device (e.g., magnetic disk)
        DirectAccessBlock = 0x00,
        /// Sequential-access device (e.g., magnetic tape)
        SequentialAccess = 0x01,
        /// Printer device
        Printer = 0x02,
        /// Processor device
        Processor = 0x03,
        /// Write-once device (e.g., some optical disks)
        WriteOnce = 0x04,
        /// CD/DVD device
        CdDvd = 0x05,
        /// Optical memory device (e.g., some optical disks)
        OpticalMemory = 0x07,
        /// Media changer device (e.g., jukeboxes)
        MediaChanger = 0x08,
        /// Storage array controller device (e.g., RAID)
        StorageArrayController = 0x0C,
        /// Enclosure services device
        EnclosureServices = 0x0D,
        /// Simplified direct-access device (e.g., magnetic disk)
        SimplifiedDirectAccess = 0x0E,
        /// Optical card reader/writer device
        OpticaCardReaderWriter = 0x0F,
        /// Bridge Controller Commands
        BridgeController = 0x10,
        /// Object-based Storage Device
        ObjectBasedStorage = 0x11,
        /// Automation/Drive Interface
        AutomationInterface = 0x12,
        /// Security manager device
        SecurityManager = 0x13,
        /// Well known logical unit
        WellKnownLogicalUnit = 0x1E,
        /// Unknown or no device type
        UnknownOrNone = 0x1F,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum ResponseDataFormat<u8> {
        /// A RESPONSE DATA FORMAT field set to 2h indicates that the standard INQUIRY data
        Standard = 0x2,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum SpcVersion<u8> {
        /// The device does not claim conformance to any standard.
        None = 0x00,
        /// The device complies to ANSI INCITS 301-1997 (SPC)
        Spc = 0x03,
        /// The device complies to ANSI INCITS 351-2001 (SPC-2)
        Spc2 = 0x04,
        /// The device complies to ANSI INCITS 408-2005 (SPC-3)
        Spc3 = 0x05,
        /// The device complies to ANSI INCITS 513-2015 (SPC-4)
        Spc4 = 0x06,
        /// The device complies to T10/BSR INCITS 503 (SPC-5)
        Spc5 = 0x07,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum TargetPortGroupSupport<u8> {
        /// The logical unit does not support asymmetric logical unit access or supports a form of asymmetric access that is vendor specific.
        /// Neither the REPORT TARGET GROUPS nor the SET TARGET PORT GROUPS commands is supported.
        Unsupported = 0b00,
        /// The logical unit supports only implicit asymmetric logical unit access (see 5.11.2.7).
        /// The logical unit is capable of changing target port asymmetric access states without a SET TARGET PORT GROUPS command.
        /// The REPORT TARGET PORT GROUPS command is supported and the SET TARGET PORT GROUPS command is not supported.
        Implicit = 0b01,
        /// The logical unit supports only explicit asymmetric logical unit access (see 5.11.2.8).
        /// The logical unit only changes target port asymmetric access states as requested with the SET TARGET PORT GROUPS command.
        /// Both the REPORT TARGET PORT GROUPS command and the SET TARGET PORT GROUPS command are supported.
        Explicit = 0b10,
        /// The logical unit supports both explicit and implicit asymmetric logical unit access.
        /// Both the REPORT TARGET PORT GROUPS command and the SET TARGET PORT GROUPS commands are supported.
        ImplicitAndExplicit = 0b11,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PageControl<u8> {
        /// Current values
        CurrentValues = 0b00,
        /// Changeable values
        ChangeableValues = 0b01,
        /// Default values
        DefaultValues = 0b10,
        /// Saved values
        SavedValues = 0b11,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum MediumType<u8> {
        Sbc = 0x00,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum ResponseCode<u8> {
        CurrentFixedSenseData = 0x70,
        DeferredFixedSenseData = 0x71,
        CurrentDescriptorSenseData = 0x72,
        DeferredDescriptorSenseData = 0x73,
    }
}

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum SenseKey<u8> {
        /// Indicates that there is no specific sense key information to be reported. This may occur for a successful command or for a command that receives CHECK CONDITION status because one of the FILEMARK , EOM , or ILI bits is set to one.
        NoSense = 0x0,
        /// Indicates that the command completed successfully, with some recovery action performed by the device server. Details may be determined by examining the additional sense bytes and the INFORMATION field. When multiple recovered errors occur during one command, the choice of which error to report (e.g., first, last, most severe) is vendor specific.
        RecoveredError = 0x1,
        /// Indicates that the logical unit is not accessible. Operator intervention may be required to correct this condition.
        NotReady = 0x2,
        /// Indicates that the command terminated with a non-recovered error condition that may have been caused by a flaw in the medium or an error in the recorded data. This sense key may also be returned if the device server is unable to distinguish between a flaw in the medium and a specific hardware failure (i.e., sense key 4h).
        MediumError = 0x3,
        /// Indicates that the device server detected a non-recoverable hardware failure (e.g., controller failure, device failure, or parity error) while performing the command or during a self test.
        HardwareError = 0x4,
        /// Indicates that:
        /// a) the command was addressed to an incorrect logical unit number (see SAM-4);
        /// b) the command had an invalid task attribute (see SAM-4);
        /// c) the command was addressed to a logical unit whose current configuration prohibits
        /// processing the command;
        /// d) there was an illegal parameter in the CDB; or
        /// e) there was an illegal parameter in the additional parameters supplied as data for some
        /// commands (e.g., PERSISTENT RESERVE OUT).
        /// If the device server detects an invalid parameter in the CDB, it shall terminate the command without
        /// altering the medium. If the device server detects an invalid parameter in the additional parameters
        /// supplied as data, the device server may have already altered the medium.
        IllegalRequest = 0x5,
        /// Indicates that a unit attention condition has been established (e.g., the removable medium may have been changed, a logical unit reset occurred). See SAM-4.
        UnitAttention = 0x6,
        /// Indicates that a command that reads or writes the medium was attempted on a block that is protected. The read or write operation is not performed.
        DataProtect = 0x7,
        /// Indicates that a write-once device or a sequential-access device encountered blank medium or format-defined end-of-data indication while reading or that a write-once device encountered a non-blank medium while writing.
        BlankCheck = 0x8,
        /// This sense key is available for reporting vendor specific conditions.
        VendorSpecific = 0x9,
        /// Indicates an EXTENDED COPY command was aborted due to an error condition on the source device, the destination device, or both (see 6.3.3).
        CopyAborted = 0xA,
        /// Indicates that the device server aborted the command. The application client may be able to recover by trying the command again.
        AbortedCommand = 0xB,
        /// Indicates that a buffered SCSI device has reached the end-of-partition and data may remain in the buffer that has not been written to the medium. One or more RECOVER BUFFERED DATA command(s) may be issued to read the unwritten data from the buffer. (See SSC-2.)
        VolumeOverflow = 0xD,
        /// Indicates that the source data did not match the data read from the medium.
        Miscompare = 0xE,
        /// Indicates there is completion sense data to be reported. This may occur for a successful command.
        Completed = 0xF,
    }
}

// There are many more variants (see asc-num.txt) but these are the ones the scsi code
// currently uses
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdditionalSenseCode {
    /// ASC 0x20, ASCQ: 0x0 - INVALID COMMAND OPERATION CODE
    InvalidCommandOperationCode,
    /// ASC 0x64, ASCQ: 0x1 - INVALID PACKET SIZE
    InvalidPacketSize,
    /// ASC 0x24, ASCQ: 0x0 - INVALID FIELD IN CDB
    InvalidFieldInCdb,
    /// ASC 0x0, ASCQ: 0x0 - NO ADDITIONAL SENSE INFORMATION
    NoAdditionalSenseInformation,
    /// ASC 0xC, ASCQ: 0x0 - WRITE ERROR
    WriteError,
    /// ASC 0x51, ASCQ: 0x0 - ERASE FAILURE
    EraseFailure,
    /// ASC 0x21, ASCQ: 0x0 - LOGICAL BLOCK ADDRESS OUT OF RANGE
    LogicalBlockAddressOutOfRange,
}

impl AdditionalSenseCode {
    /// Returns the ASC code for this variant
    pub fn asc(&self) -> u8 {
        match self {
            AdditionalSenseCode::InvalidCommandOperationCode => 32,
            AdditionalSenseCode::InvalidPacketSize => 100,
            AdditionalSenseCode::InvalidFieldInCdb => 36,
            AdditionalSenseCode::NoAdditionalSenseInformation => 0,
            AdditionalSenseCode::WriteError => 12,
            AdditionalSenseCode::EraseFailure => 81,
            AdditionalSenseCode::LogicalBlockAddressOutOfRange => 33,
        }
    }

    /// Returns the ASCQ code for this variant
    pub fn ascq(&self) -> u8 {
        match self {
            AdditionalSenseCode::InvalidCommandOperationCode => 0,
            AdditionalSenseCode::InvalidPacketSize => 1,
            AdditionalSenseCode::InvalidFieldInCdb => 0,
            AdditionalSenseCode::NoAdditionalSenseInformation => 0,
            AdditionalSenseCode::WriteError => 0,
            AdditionalSenseCode::EraseFailure => 0,
            AdditionalSenseCode::LogicalBlockAddressOutOfRange => 0,
        }
    }
}
