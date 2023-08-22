#![cfg(feature = "msos-descriptor")]

//! Microsoft OS Descriptors
//!
//! <https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/microsoft-os-2-0-descriptors-specification>

use core::mem::size_of;

use super::{capability_type, BosWriter};
use crate::types::InterfaceNumber;

/// A serialized Microsoft OS 2.0 Descriptor set.
///
/// Create with [`DeviceDescriptorSetBuilder`].
pub struct MsOsDescriptorSet<'d> {
    descriptor: &'d [u8],
    vendor_code: u8,
}

impl<'d> MsOsDescriptorSet<'d> {
    /// Gets the raw bytes of the MS OS descriptor
    pub fn descriptor(&self) -> &[u8] {
        self.descriptor
    }

    /// Gets the vendor code used by the host to retrieve the MS OS descriptor
    pub fn vendor_code(&self) -> u8 {
        self.vendor_code
    }

    /// Returns `true` if no MS OS descriptor data is available
    pub fn is_empty(&self) -> bool {
        self.descriptor.is_empty()
    }

    /// Returns the length of the descriptor field
    pub fn len(&self) -> usize {
        self.descriptor.len()
    }
}

/// Writes a Microsoft OS 2.0 Descriptor set into a buffer.
pub struct MsOsDescriptorWriter<'d> {
    buf: &'d mut [u8],

    position: usize,
    config_mark: Option<usize>,
    function_mark: Option<usize>,
    vendor_code: u8,
}

impl<'d> MsOsDescriptorWriter<'d> {
    pub(crate) fn new(buf: &'d mut [u8]) -> Self {
        MsOsDescriptorWriter {
            buf,
            position: 0,
            config_mark: None,
            function_mark: None,
            vendor_code: 0,
        }
    }

    pub(crate) fn build(mut self, bos: &mut BosWriter) -> MsOsDescriptorSet<'d> {
        self.end();

        if self.is_empty() {
            MsOsDescriptorSet {
                descriptor: &[],
                vendor_code: 0,
            }
        } else {
            self.write_bos(bos);
            MsOsDescriptorSet {
                descriptor: &self.buf[..self.position],
                vendor_code: self.vendor_code,
            }
        }
    }

    /// Returns `true` if the MS OS descriptor header has not yet been written
    pub fn is_empty(&self) -> bool {
        self.position == 0
    }

    /// Returns `true` if a configuration subset header has been started
    pub fn is_in_config_subset(&self) -> bool {
        self.config_mark.is_some()
    }

    /// Returns `true` if a function subset header has been started and not yet ended
    pub fn is_in_function_subset(&self) -> bool {
        self.function_mark.is_some()
    }

    /// Write the MS OS descriptor set header.
    ///
    /// - `windows_version` is an NTDDI version constant that describes a windows version. See the [`windows_version`]
    /// module.
    /// - `vendor_code` is the vendor request code used to read the MS OS descriptor set.
    pub fn header(&mut self, windows_version: u32, vendor_code: u8) {
        assert!(self.is_empty(), "You can only call MsOsDescriptorWriter::header once");
        self.write(DescriptorSetHeader::new(windows_version));
        self.vendor_code = vendor_code;
    }

    /// Add a device level feature descriptor.
    ///
    /// Note that some feature descriptors may only be used at the device level in non-composite devices.
    /// Those features must be written before the first call to [`Self::configuration`].
    pub fn device_feature<T: DeviceLevelDescriptor>(&mut self, desc: T) {
        assert!(
            !self.is_empty(),
            "device features may only be added after the header is written"
        );
        assert!(
            self.config_mark.is_none(),
            "device features must be added before the first configuration subset"
        );
        self.write(desc);
    }

    /// Add a configuration subset.
    pub fn configuration(&mut self, config: u8) {
        assert!(
            !self.is_empty(),
            "MsOsDescriptorWriter: configuration must be called after header"
        );
        Self::end_subset::<ConfigurationSubsetHeader>(self.buf, self.position, &mut self.config_mark);
        self.config_mark = Some(self.position);
        self.write(ConfigurationSubsetHeader::new(config));
    }

    /// Add a function subset.
    pub fn function(&mut self, first_interface: InterfaceNumber) {
        assert!(
            self.config_mark.is_some(),
            "MsOsDescriptorWriter: function subset requires a configuration subset"
        );
        self.end_function();
        self.function_mark = Some(self.position);
        self.write(FunctionSubsetHeader::new(first_interface));
    }

    /// Add a function level feature descriptor.
    ///
    /// Note that some features may only be used at the function level. Those features must be written after a call
    /// to [`Self::function`].
    pub fn function_feature<T: FunctionLevelDescriptor>(&mut self, desc: T) {
        assert!(
            self.function_mark.is_some(),
            "function features may only be added to a function subset"
        );
        self.write(desc);
    }

    /// Ends the current function subset (if any)
    pub fn end_function(&mut self) {
        Self::end_subset::<FunctionSubsetHeader>(self.buf, self.position, &mut self.function_mark);
    }

    fn write<T: Descriptor>(&mut self, desc: T) {
        desc.write_to(&mut self.buf[self.position..]);
        self.position += desc.size();
    }

    fn end_subset<T: DescriptorSet>(buf: &mut [u8], position: usize, mark: &mut Option<usize>) {
        if let Some(mark) = mark.take() {
            let len = position - mark;
            let p = mark + T::LENGTH_OFFSET;
            buf[p..(p + 2)].copy_from_slice(&(len as u16).to_le_bytes());
        }
    }

    fn end(&mut self) {
        if self.position > 0 {
            Self::end_subset::<FunctionSubsetHeader>(self.buf, self.position, &mut self.function_mark);
            Self::end_subset::<ConfigurationSubsetHeader>(self.buf, self.position, &mut self.config_mark);
            Self::end_subset::<DescriptorSetHeader>(self.buf, self.position, &mut Some(0));
        }
    }

    fn write_bos(&mut self, bos: &mut BosWriter) {
        let windows_version = &self.buf[4..8];
        let len = (self.position as u16).to_le_bytes();
        bos.capability(
            capability_type::PLATFORM,
            &[
                0, // reserved
                // platform capability UUID, Microsoft OS 2.0 platform compatibility
                0xdf,
                0x60,
                0xdd,
                0xd8,
                0x89,
                0x45,
                0xc7,
                0x4c,
                0x9c,
                0xd2,
                0x65,
                0x9d,
                0x9e,
                0x64,
                0x8a,
                0x9f,
                // Minimum compatible Windows version
                windows_version[0],
                windows_version[1],
                windows_version[2],
                windows_version[3],
                // Descriptor set length
                len[0],
                len[1],
                self.vendor_code,
                0x0, // Device does not support alternate enumeration
            ],
        );
    }
}

/// Microsoft Windows version codes
///
/// Windows 8.1 is the minimum version allowed for MS OS 2.0 descriptors.
pub mod windows_version {
    /// Windows 8.1 (aka `NTDDI_WINBLUE`)
    pub const WIN8_1: u32 = 0x06030000;
    /// Windows 10
    pub const WIN10: u32 = 0x0A000000;
}

mod sealed {
    use core::mem::size_of;

    /// A trait for descriptors
    pub trait Descriptor: Sized {
        const TYPE: super::DescriptorType;

        /// The size of the descriptor's header.
        fn size(&self) -> usize {
            size_of::<Self>()
        }

        fn write_to(&self, buf: &mut [u8]);
    }

    pub trait DescriptorSet: Descriptor {
        const LENGTH_OFFSET: usize;
    }
}

use sealed::*;

/// Copies the data of `t` into `buf`.
///
/// # Safety
/// The type `T` must be able to be safely cast to `&[u8]`. (e.g. it is a `#[repr(packed)]` struct)
unsafe fn transmute_write_to<T: Sized>(t: &T, buf: &mut [u8]) {
    let bytes = core::slice::from_raw_parts((t as *const T) as *const u8, size_of::<T>());
    assert!(buf.len() >= bytes.len(), "MS OS descriptor buffer full");
    (&mut buf[..bytes.len()]).copy_from_slice(bytes);
}

/// Table 9. Microsoft OS 2.0 descriptor wDescriptorType values.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DescriptorType {
    /// MS OS descriptor set header
    SetHeaderDescriptor = 0,
    /// Configuration subset header
    SubsetHeaderConfiguration = 1,
    /// Function subset header
    SubsetHeaderFunction = 2,
    /// Compatible device ID feature descriptor
    FeatureCompatibleId = 3,
    /// Registry property feature descriptor
    FeatureRegProperty = 4,
    /// Minimum USB resume time feature descriptor
    FeatureMinResumeTime = 5,
    /// Vendor revision feature descriptor
    FeatureModelId = 6,
    /// CCGP device descriptor feature descriptor
    FeatureCcgpDevice = 7,
    /// Vendor revision feature descriptor
    FeatureVendorRevision = 8,
}

/// Table 5. Descriptor set information structure.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct DescriptorSetInformation {
    dwWindowsVersion: u32,
    wMSOSDescriptorSetTotalLength: u16,
    bMS_VendorCode: u8,
    bAltEnumCode: u8,
}

/// Table 4. Microsoft OS 2.0 platform capability descriptor header.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct PlatformDescriptor {
    bLength: u8,
    bDescriptorType: u8,
    bDevCapabilityType: u8,
    bReserved: u8,
    platformCapabilityUUID: [u8; 16],
    descriptor_set_information: DescriptorSetInformation,
}

/// Table 10. Microsoft OS 2.0 descriptor set header.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct DescriptorSetHeader {
    wLength: u16,
    wDescriptorType: u16,
    dwWindowsVersion: u32,
    wTotalLength: u16,
}

impl DescriptorSetHeader {
    /// Creates a MS OS descriptor set header.
    ///
    /// `windows_version` is the minimum Windows version the descriptor set can apply to.
    pub fn new(windows_version: u32) -> Self {
        DescriptorSetHeader {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            dwWindowsVersion: windows_version.to_le(),
            wTotalLength: 0,
        }
    }
}

impl Descriptor for DescriptorSetHeader {
    const TYPE: DescriptorType = DescriptorType::SetHeaderDescriptor;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl DescriptorSet for DescriptorSetHeader {
    const LENGTH_OFFSET: usize = 8;
}

/// Table 11. Configuration subset header.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct ConfigurationSubsetHeader {
    wLength: u16,
    wDescriptorType: u16,
    bConfigurationValue: u8,
    bReserved: u8,
    wTotalLength: u16,
}

impl ConfigurationSubsetHeader {
    /// Creates a configuration subset header
    pub fn new(config: u8) -> Self {
        ConfigurationSubsetHeader {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            bConfigurationValue: config,
            bReserved: 0,
            wTotalLength: 0,
        }
    }
}

impl Descriptor for ConfigurationSubsetHeader {
    const TYPE: DescriptorType = DescriptorType::SubsetHeaderConfiguration;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl DescriptorSet for ConfigurationSubsetHeader {
    const LENGTH_OFFSET: usize = 6;
}

/// Table 12. Function subset header.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct FunctionSubsetHeader {
    wLength: u16,
    wDescriptorType: u16,
    bFirstInterface: InterfaceNumber,
    bReserved: u8,
    wSubsetLength: u16,
}

impl FunctionSubsetHeader {
    /// Creates a function subset header
    pub fn new(first_interface: InterfaceNumber) -> Self {
        FunctionSubsetHeader {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            bFirstInterface: first_interface,
            bReserved: 0,
            wSubsetLength: 0,
        }
    }
}

impl Descriptor for FunctionSubsetHeader {
    const TYPE: DescriptorType = DescriptorType::SubsetHeaderFunction;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl DescriptorSet for FunctionSubsetHeader {
    const LENGTH_OFFSET: usize = 6;
}

// Feature Descriptors

/// A marker trait for feature descriptors that are valid at the device level.
pub trait DeviceLevelDescriptor: Descriptor {}

/// A marker trait for feature descriptors that are valid at the function level.
pub trait FunctionLevelDescriptor: Descriptor {}

/// Table 13. Microsoft OS 2.0 compatible ID descriptor.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct CompatibleIdFeatureDescriptor {
    wLength: u16,
    wDescriptorType: u16,
    compatibleId: [u8; 8],
    subCompatibleId: [u8; 8],
}

impl DeviceLevelDescriptor for CompatibleIdFeatureDescriptor {}
impl FunctionLevelDescriptor for CompatibleIdFeatureDescriptor {}

impl Descriptor for CompatibleIdFeatureDescriptor {
    const TYPE: DescriptorType = DescriptorType::FeatureCompatibleId;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl CompatibleIdFeatureDescriptor {
    /// Creates a compatible ID feature descriptor
    ///
    /// The ids must be 8 ASCII bytes or fewer.
    pub fn new(compatible_id: &str, sub_compatible_id: &str) -> Self {
        assert!(compatible_id.len() <= 8 && sub_compatible_id.len() <= 8);
        let mut cid = [0u8; 8];
        (&mut cid[..compatible_id.len()]).copy_from_slice(compatible_id.as_bytes());
        let mut scid = [0u8; 8];
        (&mut scid[..sub_compatible_id.len()]).copy_from_slice(sub_compatible_id.as_bytes());
        Self::new_raw(cid, scid)
    }

    fn new_raw(compatible_id: [u8; 8], sub_compatible_id: [u8; 8]) -> Self {
        Self {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            compatibleId: compatible_id,
            subCompatibleId: sub_compatible_id,
        }
    }
}

/// Table 14. Microsoft OS 2.0 registry property descriptor
#[allow(non_snake_case)]
pub struct RegistryPropertyFeatureDescriptor<'a> {
    name: &'a str,
    data: PropertyData<'a>,
}

/// Data values that can be encoded into a registry property descriptor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PropertyData<'a> {
    /// A registry property containing a string.
    Sz(&'a str),
    /// A registry property containing a string that expands environment variables.
    ExpandSz(&'a str),
    /// A registry property containing binary data.
    Binary(&'a [u8]),
    /// A registry property containing a little-endian 32-bit integer.
    DwordLittleEndian(u32),
    /// A registry property containing a big-endian 32-bit integer.
    DwordBigEndian(u32),
    /// A registry property containing a string that contains a symbolic link.
    Link(&'a str),
    /// A registry property containing multiple strings.
    RegMultiSz(&'a [&'a str]),
}

fn write_bytes(val: &[u8], buf: &mut [u8]) -> usize {
    assert!(buf.len() >= val.len());
    buf[..val.len()].copy_from_slice(val);
    val.len()
}

fn write_utf16(val: &str, buf: &mut [u8]) -> usize {
    let mut pos = 0;
    for c in val.encode_utf16() {
        pos += write_bytes(&c.to_le_bytes(), &mut buf[pos..]);
    }
    pos + write_bytes(&0u16.to_le_bytes(), &mut buf[pos..])
}

impl<'a> PropertyData<'a> {
    /// Gets the `PropertyDataType` for this property value
    pub fn kind(&self) -> PropertyDataType {
        match self {
            PropertyData::Sz(_) => PropertyDataType::Sz,
            PropertyData::ExpandSz(_) => PropertyDataType::ExpandSz,
            PropertyData::Binary(_) => PropertyDataType::Binary,
            PropertyData::DwordLittleEndian(_) => PropertyDataType::DwordLittleEndian,
            PropertyData::DwordBigEndian(_) => PropertyDataType::DwordBigEndian,
            PropertyData::Link(_) => PropertyDataType::Link,
            PropertyData::RegMultiSz(_) => PropertyDataType::RegMultiSz,
        }
    }

    /// Gets the size (in bytes) of this property value when encoded.
    pub fn size(&self) -> usize {
        match self {
            PropertyData::Sz(val) | PropertyData::ExpandSz(val) | PropertyData::Link(val) => {
                core::mem::size_of::<u16>() * (val.encode_utf16().count() + 1)
            }
            PropertyData::Binary(val) => val.len(),
            PropertyData::DwordLittleEndian(val) | PropertyData::DwordBigEndian(val) => core::mem::size_of_val(val),
            PropertyData::RegMultiSz(val) => {
                core::mem::size_of::<u16>() * (val.iter().map(|x| x.encode_utf16().count() + 1).sum::<usize>() + 1)
            }
        }
    }

    /// Encodes the data for this property value and writes it to `buf`.
    pub fn write(&self, buf: &mut [u8]) -> usize {
        match self {
            PropertyData::Sz(val) | PropertyData::ExpandSz(val) | PropertyData::Link(val) => write_utf16(val, buf),
            PropertyData::Binary(val) => write_bytes(val, buf),
            PropertyData::DwordLittleEndian(val) => write_bytes(&val.to_le_bytes(), buf),
            PropertyData::DwordBigEndian(val) => write_bytes(&val.to_be_bytes(), buf),
            PropertyData::RegMultiSz(val) => {
                let mut pos = 0;
                for s in *val {
                    pos += write_utf16(s, &mut buf[pos..]);
                }
                pos + write_bytes(&0u16.to_le_bytes(), &mut buf[pos..])
            }
        }
    }
}

/// Table 15. wPropertyDataType values for the Microsoft OS 2.0 registry property descriptor.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PropertyDataType {
    /// A registry property containing a string.
    Sz = 1,
    /// A registry property containing a string that expands environment variables.
    ExpandSz = 2,
    /// A registry property containing binary data.
    Binary = 3,
    /// A registry property containing a little-endian 32-bit integer.
    DwordLittleEndian = 4,
    /// A registry property containing a big-endian 32-bit integer.
    DwordBigEndian = 5,
    /// A registry property containing a string that contains a symbolic link.
    Link = 6,
    /// A registry property containing multiple strings.
    RegMultiSz = 7,
}

impl<'a> DeviceLevelDescriptor for RegistryPropertyFeatureDescriptor<'a> {}
impl<'a> FunctionLevelDescriptor for RegistryPropertyFeatureDescriptor<'a> {}

impl<'a> Descriptor for RegistryPropertyFeatureDescriptor<'a> {
    const TYPE: DescriptorType = DescriptorType::FeatureRegProperty;

    fn size(&self) -> usize {
        10 + self.name_size() + self.data.size()
    }

    fn write_to(&self, buf: &mut [u8]) {
        assert!(buf.len() >= self.size(), "MS OS descriptor buffer full");

        let mut pos = 0;
        pos += write_bytes(&(self.size() as u16).to_le_bytes(), &mut buf[pos..]);
        pos += write_bytes(&(Self::TYPE as u16).to_le_bytes(), &mut buf[pos..]);
        pos += write_bytes(&(self.data.kind() as u16).to_le_bytes(), &mut buf[pos..]);
        pos += write_bytes(&(self.name_size() as u16).to_le_bytes(), &mut buf[pos..]);
        pos += write_utf16(self.name, &mut buf[pos..]);
        pos += write_bytes(&(self.data.size() as u16).to_le_bytes(), &mut buf[pos..]);
        self.data.write(&mut buf[pos..]);
    }
}

impl<'a> RegistryPropertyFeatureDescriptor<'a> {
    /// A registry property.
    pub fn new(name: &'a str, data: PropertyData<'a>) -> Self {
        Self { name, data }
    }

    fn name_size(&self) -> usize {
        core::mem::size_of::<u16>() * (self.name.encode_utf16().count() + 1)
    }
}

/// Table 16. Microsoft OS 2.0 minimum USB recovery time descriptor.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct MinimumRecoveryTimeDescriptor {
    wLength: u16,
    wDescriptorType: u16,
    bResumeRecoveryTime: u8,
    bResumeSignalingTime: u8,
}

impl DeviceLevelDescriptor for MinimumRecoveryTimeDescriptor {}

impl Descriptor for MinimumRecoveryTimeDescriptor {
    const TYPE: DescriptorType = DescriptorType::FeatureMinResumeTime;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl MinimumRecoveryTimeDescriptor {
    /// Times are in milliseconds.
    ///
    /// `resume_recovery_time` must be >= 0 and <= 10.
    /// `resume_signaling_time` must be >= 1 and <= 20.
    pub fn new(resume_recovery_time: u8, resume_signaling_time: u8) -> Self {
        assert!(resume_recovery_time <= 10);
        assert!(resume_signaling_time >= 1 && resume_signaling_time <= 20);
        Self {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            bResumeRecoveryTime: resume_recovery_time,
            bResumeSignalingTime: resume_signaling_time,
        }
    }
}

/// Table 17. Microsoft OS 2.0 model ID descriptor.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct ModelIdDescriptor {
    wLength: u16,
    wDescriptorType: u16,
    modelId: [u8; 16],
}

impl DeviceLevelDescriptor for ModelIdDescriptor {}

impl Descriptor for ModelIdDescriptor {
    const TYPE: DescriptorType = DescriptorType::FeatureModelId;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl ModelIdDescriptor {
    /// Creates a new model ID descriptor
    ///
    /// `model_id` should be a uuid that uniquely identifies a physical device.
    pub fn new(model_id: u128) -> Self {
        Self {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            modelId: model_id.to_le_bytes(),
        }
    }
}

/// Table 18. Microsoft OS 2.0 CCGP device descriptor.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct CcgpDeviceDescriptor {
    wLength: u16,
    wDescriptorType: u16,
}

impl DeviceLevelDescriptor for CcgpDeviceDescriptor {}

impl Descriptor for CcgpDeviceDescriptor {
    const TYPE: DescriptorType = DescriptorType::FeatureCcgpDevice;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl CcgpDeviceDescriptor {
    /// Creates a new CCGP device descriptor
    pub fn new() -> Self {
        Self {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
        }
    }
}

/// Table 19. Microsoft OS 2.0 vendor revision descriptor.
#[allow(non_snake_case)]
#[repr(C, packed(1))]
pub struct VendorRevisionDescriptor {
    wLength: u16,
    wDescriptorType: u16,
    /// Revision number associated with the descriptor set. Modify it every time you add/modify a registry property or
    /// other MS OS descriptor. Shell set to greater than or equal to 1.
    VendorRevision: u16,
}

impl DeviceLevelDescriptor for VendorRevisionDescriptor {}
impl FunctionLevelDescriptor for VendorRevisionDescriptor {}

impl Descriptor for VendorRevisionDescriptor {
    const TYPE: DescriptorType = DescriptorType::FeatureVendorRevision;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

impl VendorRevisionDescriptor {
    /// Creates a new vendor revision descriptor
    pub fn new(revision: u16) -> Self {
        assert!(revision >= 1);
        Self {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            VendorRevision: revision.to_le(),
        }
    }
}
