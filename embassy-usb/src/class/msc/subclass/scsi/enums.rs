use crate::packed_enum;

packed_enum! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
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
