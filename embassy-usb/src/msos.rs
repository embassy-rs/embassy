#![cfg(feature = "msos-descriptor")]

//! Microsoft OS Descriptors
//!
//! <https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/microsoft-os-2-0-descriptors-specification>

use core::mem::size_of;
use core::ops::Range;

pub use widestring::{u16cstr, u16str, U16CStr, U16Str};

use super::{capability_type, BosWriter};

fn write_u16<T: Into<u16>>(buf: &mut [u8], range: Range<usize>, data: T) {
    (&mut buf[range]).copy_from_slice(data.into().to_le_bytes().as_slice())
}

/// A serialized Microsoft OS 2.0 Descriptor set.
///
/// Create with [`DeviceDescriptorSetBuilder`].
pub struct MsOsDescriptorSet<'d> {
    descriptor: &'d [u8],
    vendor_code: u8,
}

impl<'d> MsOsDescriptorSet<'d> {
    pub fn descriptor(&self) -> &[u8] {
        self.descriptor
    }

    pub fn vendor_code(&self) -> u8 {
        self.vendor_code
    }

    pub fn is_empty(&self) -> bool {
        self.descriptor.is_empty()
    }
}

/// Writes a Microsoft OS 2.0 Descriptor set into a buffer.
pub struct MsOsDescriptorWriter<'d> {
    pub buf: &'d mut [u8],

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

    pub fn is_empty(&self) -> bool {
        self.position == 0
    }

    pub fn is_in_config_subset(&self) -> bool {
        self.config_mark.is_some()
    }

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
    pub fn function(&mut self, first_interface: u8) {
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
                // platform capability UUID, Microsoft OS 2.0 platform compabitility
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

pub mod windows_version {
    pub const WIN2K: u32 = 0x05000000;
    pub const WIN2KSP1: u32 = 0x05000100;
    pub const WIN2KSP2: u32 = 0x05000200;
    pub const WIN2KSP3: u32 = 0x05000300;
    pub const WIN2KSP4: u32 = 0x05000400;

    pub const WINXP: u32 = 0x05010000;
    pub const WINXPSP1: u32 = 0x05010100;
    pub const WINXPSP2: u32 = 0x05010200;
    pub const WINXPSP3: u32 = 0x05010300;
    pub const WINXPSP4: u32 = 0x05010400;

    pub const VISTA: u32 = 0x06000000;
    pub const VISTASP1: u32 = 0x06000100;
    pub const VISTASP2: u32 = 0x06000200;
    pub const VISTASP3: u32 = 0x06000300;
    pub const VISTASP4: u32 = 0x06000400;

    pub const WIN7: u32 = 0x06010000;
    pub const WIN8: u32 = 0x06020000;
    /// AKA `NTDDI_WINBLUE`
    pub const WIN8_1: u32 = 0x06030000;
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
    assert!(buf.len() >= bytes.len(), "MSOS descriptor buffer full");
    (&mut buf[..bytes.len()]).copy_from_slice(bytes);
}

/// Table 9. Microsoft OS 2.0 descriptor wDescriptorType values.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DescriptorType {
    SetHeaderDescriptor = 0,
    SubsetHeaderConfiguration = 1,
    SubsetHeaderFunction = 2,
    FeatureCompatibleId = 3,
    FeatureRegProperty = 4,
    FeatureMinResumeTime = 5,
    FeatureModelId = 6,
    FeatureCcgpDevice = 7,
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
    bFirstInterface: u8,
    bReserved: u8,
    wSubsetLength: u16,
}

impl FunctionSubsetHeader {
    pub fn new(first_interface: u8) -> Self {
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
    /// Creates a compatible ID descriptor that signals WINUSB driver compatiblilty.
    pub fn new_winusb() -> Self {
        Self::new_raw([b'W', b'I', b'N', b'U', b'S', b'B', 0, 0], [0u8; 8])
    }

    /// The ids must be 8 ASCII bytes or fewer.
    pub fn new(compatible_id: &str, sub_compatible_id: &str) -> Self {
        assert!(compatible_id.len() <= 8 && sub_compatible_id.len() <= 8);
        let mut cid = [0u8; 8];
        (&mut cid[..compatible_id.len()]).copy_from_slice(compatible_id.as_bytes());
        let mut scid = [0u8; 8];
        (&mut scid[..sub_compatible_id.len()]).copy_from_slice(sub_compatible_id.as_bytes());
        Self::new_raw(cid, scid)
    }

    pub fn new_raw(compatible_id: [u8; 8], sub_compatible_id: [u8; 8]) -> Self {
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
    wLength: u16,
    wDescriptorType: u16,
    wPropertyDataType: u16,
    PropertyName: &'a [u8],
    PropertyData: &'a [u8],
}

impl<'a> DeviceLevelDescriptor for RegistryPropertyFeatureDescriptor<'a> {}
impl<'a> FunctionLevelDescriptor for RegistryPropertyFeatureDescriptor<'a> {}

impl<'a> Descriptor for RegistryPropertyFeatureDescriptor<'a> {
    const TYPE: DescriptorType = DescriptorType::FeatureRegProperty;
    fn size(&self) -> usize {
        10 + self.PropertyName.len() + self.PropertyData.len()
    }
    fn write_to(&self, buf: &mut [u8]) {
        assert!(buf.len() >= self.size(), "MSOS descriptor buffer full");
        write_u16(buf, 0..2, self.wLength);
        write_u16(buf, 2..4, self.wDescriptorType);
        write_u16(buf, 4..6, self.wPropertyDataType);
        write_u16(buf, 6..8, (self.PropertyName.len() as u16).to_le());
        let pne = 8 + self.PropertyName.len();
        (&mut buf[8..pne]).copy_from_slice(self.PropertyName);
        let pds = pne + 2;
        let pde = pds + self.PropertyData.len();
        write_u16(buf, pne..pds, (self.PropertyData.len() as u16).to_le());
        (&mut buf[pds..pde]).copy_from_slice(self.PropertyData);
    }
}

impl<'a> RegistryPropertyFeatureDescriptor<'a> {
    pub const DEVICE_INTERFACE_GUIDS_NAME: &U16CStr = {
        // Can't use defmt::panic in constant expressions (inside u16cstr!)
        macro_rules! panic {
            ($($x:tt)*) => {
                {
                    ::core::panic!($($x)*);
                }
            };
        }

        u16cstr!("DeviceInterfaceGUIDs")
    };

    /// A registry property.
    ///
    /// `name` should be a NUL-terminated 16-bit Unicode string.
    pub fn new_raw<'n: 'a, 'd: 'a>(name: &'a [u8], data: &'d [u8], data_type: PropertyDataType) -> Self {
        assert!(name.len() < usize::from(u16::MAX) && data.len() < usize::from(u16::MAX));
        Self {
            wLength: ((10 + name.len() + data.len()) as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            wPropertyDataType: (data_type as u16).to_le(),
            PropertyName: name,
            PropertyData: data,
        }
    }

    fn u16str_bytes(s: &U16CStr) -> &[u8] {
        unsafe { core::slice::from_raw_parts(s.as_ptr() as *const u8, (s.len() + 1) * 2) }
    }

    /// A registry property containing a NUL-terminated 16-bit Unicode string.
    pub fn new_string<'n: 'a, 'd: 'a>(name: &'n U16CStr, data: &'d U16CStr) -> Self {
        Self::new_raw(Self::u16str_bytes(name), Self::u16str_bytes(data), PropertyDataType::Sz)
    }

    /// A registry property containing a NUL-terminated 16-bit Unicode string that expands environment variables.
    pub fn new_string_expand<'n: 'a, 'd: 'a>(name: &'n U16CStr, data: &'d U16CStr) -> Self {
        Self::new_raw(
            Self::u16str_bytes(name),
            Self::u16str_bytes(data),
            PropertyDataType::ExpandSz,
        )
    }

    /// A registry property containing a NUL-terminated 16-bit Unicode string that contains a symbolic link.
    pub fn new_link<'n: 'a, 'd: 'a>(name: &'n U16CStr, data: &'d U16CStr) -> Self {
        Self::new_raw(
            Self::u16str_bytes(name),
            Self::u16str_bytes(data),
            PropertyDataType::Link,
        )
    }

    /// A registry property containing multiple NUL-terminated 16-bit Unicode strings.
    pub fn new_multi_string<'n: 'a, 'd: 'a>(name: &'n U16CStr, data: &'d [u16]) -> Self {
        assert!(
            data.len() >= 2 && data[data.len() - 1] == 0 && data[data.len() - 2] == 0,
            "multi-strings must end in double nul terminators"
        );
        Self::new_raw(
            Self::u16str_bytes(name),
            unsafe { core::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2) },
            PropertyDataType::RegMultiSz,
        )
    }

    /// A registry property containing binary data.
    pub fn new_binary<'n: 'a, 'd: 'a>(name: &'n U16CStr, data: &'d [u8]) -> Self {
        Self::new_raw(Self::u16str_bytes(name), data, PropertyDataType::Binary)
    }

    /// A registry property containing a Little-Endian 32-bit integer.
    ///
    /// The function assumes that `data` is already little-endian, it does not convert it.
    pub fn new_dword_le<'n: 'a, 'd: 'a>(name: &'n U16CStr, data: &'d i32) -> Self {
        Self::new_raw(
            Self::u16str_bytes(name),
            unsafe { core::slice::from_raw_parts(data as *const i32 as *const u8, size_of::<i32>()) },
            PropertyDataType::DwordLittleEndian,
        )
    }

    /// A registry property containing a big-endian 32-bit integer.
    ///
    /// The function assumes that `data` is already big-endian, it does not convert it.
    pub fn new_dword_be<'n: 'a, 'd: 'a>(name: &'n U16CStr, data: &'d i32) -> Self {
        Self::new_raw(
            Self::u16str_bytes(name),
            unsafe { core::slice::from_raw_parts(data as *const i32 as *const u8, size_of::<i32>()) },
            PropertyDataType::DwordBigEndian,
        )
    }
}

/// Table 15. wPropertyDataType values for the Microsoft OS 2.0 registry property descriptor.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PropertyDataType {
    Sz = 1,
    ExpandSz = 2,
    Binary = 3,
    DwordLittleEndian = 4,
    DwordBigEndian = 5,
    Link = 6,
    RegMultiSz = 7,
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
    pub fn new(model_id: u128) -> Self {
        Self::new_bytes(model_id.to_le_bytes())
    }

    pub fn new_bytes(model_id: [u8; 16]) -> Self {
        Self {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            modelId: model_id,
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
    /// other MSOS descriptor. Shell set to greater than or equal to 1.
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
    pub fn new(revision: u16) -> Self {
        assert!(revision >= 1);
        Self {
            wLength: (size_of::<Self>() as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            VendorRevision: revision.to_le(),
        }
    }
}
