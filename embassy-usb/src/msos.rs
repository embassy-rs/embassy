//! Microsoft OS Descriptors
//!
//! <https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/microsoft-os-2-0-descriptors-specification>

#![allow(dead_code)]

use core::mem::size_of;
use core::ops::Range;

pub use widestring::{u16cstr, U16CStr};

use crate::descriptor::{capability_type, BosWriter};
use crate::types::InterfaceNumber;

fn write_u16<T: Into<u16>>(buf: &mut [u8], range: Range<usize>, data: T) {
    (&mut buf[range]).copy_from_slice(data.into().to_le_bytes().as_slice())
}

/// A serialized Microsoft OS 2.0 Descriptor set.
///
/// Create with [`DeviceDescriptorSetBuilder`].
pub struct MsOsDescriptorSet<'a> {
    descriptor: &'a [u8],
    windows_version: u32,
    vendor_code: u8,
}

impl<'a> MsOsDescriptorSet<'a> {
    pub fn descriptor(&self) -> &[u8] {
        self.descriptor
    }

    pub fn vendor_code(&self) -> u8 {
        self.vendor_code
    }

    pub fn write_bos_capability(&self, bos: &mut BosWriter) {
        let windows_version = self.windows_version.to_le_bytes();
        let len = self.descriptor.len().to_le_bytes();
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
        )
    }
}

/// A helper struct to implement the different descriptor set builders.
struct DescriptorSetBuilder<'a> {
    used: usize,
    buf: &'a mut [u8],
}

impl<'a> DescriptorSetBuilder<'a> {
    pub fn descriptor<T>(&mut self, desc: T)
    where
        T: Descriptor + 'a,
    {
        let size = desc.size();
        let start = self.used;
        let end = start + size;
        desc.write_to(&mut self.buf[start..end]);
        self.used += size;
    }

    pub fn subset(&mut self, build_subset: impl FnOnce(&mut DescriptorSetBuilder<'_>)) {
        self.used += {
            let mut subset = DescriptorSetBuilder {
                used: 0,
                buf: self.remaining(),
            };
            build_subset(&mut subset);
            subset.used
        };
    }

    pub fn remaining(&mut self) -> &mut [u8] {
        &mut self.buf[self.used..]
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

/// Helps build a Microsoft OS 2.0 Descriptor set.
///
/// # Example
/// ```rust
/// # use embassy_usb::types::InterfaceNumber;
/// # use embassy_usb::msos::*;
/// # let cdc_interface = unsafe { core::mem::transmute::<u8, InterfaceNumber>(0) };
/// # let dfu_interface = unsafe { core::mem::transmute::<u8, InterfaceNumber>(1) };
/// let mut buf = [0u8; 256];
/// let mut builder = DeviceDescriptorSetBuilder::new(&mut buf[..], windows_version::WIN8_1);
/// builder.feature(MinimumRecoveryTimeDescriptor::new(5, 10));
/// builder.feature(ModelIdDescriptor::new(0xdeadbeef1234u128));
/// builder.configuration(1, |conf| {
///     conf.function(cdc_interface, |func| {
///         func.winusb_device();
///         func.feature(VendorRevisionDescriptor::new(1));
///     });
///     conf.function(dfu_interface, |func| {
///         func.winusb_device();
///         func.feature(VendorRevisionDescriptor::new(1));
///     });
/// });
/// ```
pub struct DeviceDescriptorSetBuilder<'a> {
    builder: DescriptorSetBuilder<'a>,
    windows_version: u32,
    vendor_code: u8,
}

impl<'a> DeviceDescriptorSetBuilder<'a> {
    /// Create a device descriptor set builder.
    ///
    /// - `windows_version` is an NTDDI version constant that describes a windows version. See the [`windows_version`]
    /// module.
    /// - `vendor_code` is the vendor request code used to read the MS OS descriptor set.
    pub fn new<'b: 'a>(buf: &'b mut [u8], windows_version: u32, vendor_code: u8) -> Self {
        let mut builder = DescriptorSetBuilder { used: 0, buf };
        builder.descriptor(DescriptorSetHeader {
            wLength: (size_of::<DescriptorSetHeader>() as u16).to_le(),
            wDescriptorType: (DescriptorSetHeader::TYPE as u16).to_le(),
            dwWindowsVersion: windows_version.to_le(),
            wTotalLength: 0,
        });
        Self {
            builder,
            windows_version,
            vendor_code,
        }
    }

    /// Add a device-level feature descriptor.
    ///
    /// Note that some feature descriptors may only be used at the device level in non-composite devices.
    pub fn feature<T>(&mut self, desc: T)
    where
        T: Descriptor + DeviceLevelDescriptor + 'a,
    {
        self.builder.descriptor(desc)
    }

    /// Add a configuration subset.
    pub fn configuration(&mut self, configuration: u8, build_conf: impl FnOnce(&mut ConfigurationSubsetBuilder<'_>)) {
        let mut cb = ConfigurationSubsetBuilder::new(self.builder.remaining(), configuration);
        build_conf(&mut cb);
        self.builder.used += cb.finalize();
    }

    /// Finishes writing the data.
    pub fn finalize(self) -> MsOsDescriptorSet<'a> {
        let used = self.builder.used;
        let buf = self.builder.buf;
        // Update length in header with final length
        let total_len = &mut buf[8..10];
        total_len.copy_from_slice((used as u16).to_le_bytes().as_slice());

        MsOsDescriptorSet {
            descriptor: &buf[..used],
            windows_version: self.windows_version,
            vendor_code: self.vendor_code,
        }
    }
}

pub struct ConfigurationSubsetBuilder<'a> {
    builder: DescriptorSetBuilder<'a>,
}

impl<'a> ConfigurationSubsetBuilder<'a> {
    pub fn new<'b: 'a>(buf: &'b mut [u8], configuration: u8) -> Self {
        let mut builder = DescriptorSetBuilder { used: 0, buf };
        builder.descriptor(ConfigurationSubsetHeader {
            wLength: (size_of::<ConfigurationSubsetHeader>() as u16).to_le(),
            wDescriptorType: (ConfigurationSubsetHeader::TYPE as u16).to_le(),
            bConfigurationValue: configuration,
            bReserved: 0,
            wTotalLength: 0,
        });
        Self { builder }
    }

    /// Add a function subset.
    pub fn function(&mut self, interface: InterfaceNumber, build_func: impl FnOnce(&mut FunctionSubsetBuilder<'_>)) {
        let mut fb = FunctionSubsetBuilder::new(self.builder.remaining(), interface);
        build_func(&mut fb);
        self.builder.used += fb.finalize();
    }

    /// Finishes writing the data. Returns the total number of bytes used by the descriptor set.
    pub fn finalize(self) -> usize {
        let used = self.builder.used;
        let buf = self.builder.buf;
        // Update length in header with final length
        let total_len = &mut buf[6..8];
        total_len.copy_from_slice((used as u16).to_le_bytes().as_slice());
        used
    }
}

pub struct FunctionSubsetBuilder<'a> {
    builder: DescriptorSetBuilder<'a>,
}

impl<'a> FunctionSubsetBuilder<'a> {
    pub fn new<'b: 'a>(buf: &'b mut [u8], interface: InterfaceNumber) -> Self {
        let mut builder = DescriptorSetBuilder { used: 0, buf };
        builder.descriptor(FunctionSubsetHeader {
            wLength: (size_of::<FunctionSubsetHeader>() as u16).to_le(),
            wDescriptorType: (FunctionSubsetHeader::TYPE as u16).to_le(),
            bFirstInterface: interface.0,
            bReserved: 0,
            wSubsetLength: 0,
        });
        Self { builder }
    }

    /// Add a function-level descriptor.
    ///
    /// Note that many descriptors can only be used at function-level in a composite device.
    pub fn feature<T>(&mut self, desc: T)
    where
        T: Descriptor + FunctionLevelDescriptor + 'a,
    {
        self.builder.descriptor(desc)
    }

    /// Adds the feature descriptors to configure this function to use the WinUSB driver.
    ///
    /// Adds a compatible id descriptor "WINUSB" and a registry descriptor that sets the DeviceInterfaceGUID to the
    /// USB_DEVICE GUID.
    pub fn winusb_device(&mut self) {
        self.feature(CompatibleIdFeatureDescriptor::new_winusb());
        self.feature(RegistryPropertyFeatureDescriptor::new_usb_deviceinterfaceguid());
    }

    /// Finishes writing the data. Returns the total number of bytes used by the descriptor set.
    pub fn finalize(self) -> usize {
        let used = self.builder.used;
        let buf = self.builder.buf;
        // Update length in header with final length
        let total_len = &mut buf[6..8];
        total_len.copy_from_slice((used as u16).to_le_bytes().as_slice());
        used
    }
}

/// A trait for descriptors
pub trait Descriptor: Sized {
    const TYPE: DescriptorType;

    /// The size of the descriptor's header.
    fn size(&self) -> usize {
        size_of::<Self>()
    }

    fn write_to(&self, buf: &mut [u8]);
}

/// Copies the data of `t` into `buf`.
///
/// # Safety
/// The type `T` must be able to be safely cast to `&[u8]`. (e.g. it is a `#[repr(packed)]` struct)
unsafe fn transmute_write_to<T: Sized>(t: &T, buf: &mut [u8]) {
    let bytes = core::slice::from_raw_parts((t as *const T) as *const u8, size_of::<T>());
    assert!(buf.len() >= bytes.len());
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

impl Descriptor for DescriptorSetHeader {
    const TYPE: DescriptorType = DescriptorType::SetHeaderDescriptor;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
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

impl Descriptor for ConfigurationSubsetHeader {
    const TYPE: DescriptorType = DescriptorType::SubsetHeaderConfiguration;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
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

impl Descriptor for FunctionSubsetHeader {
    const TYPE: DescriptorType = DescriptorType::SubsetHeaderFunction;
    fn write_to(&self, buf: &mut [u8]) {
        unsafe { transmute_write_to(self, buf) }
    }
}

// Feature Descriptors

/// A marker trait for feature descriptors that are valid at the device level.
pub trait DeviceLevelDescriptor {}

/// A marker trait for feature descriptors that are valid at the function level.
pub trait FunctionLevelDescriptor {
    /// `true` when the feature descriptor may only be used at the function level in composite devices.
    const COMPOSITE_ONLY: bool = false;
}

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
impl FunctionLevelDescriptor for CompatibleIdFeatureDescriptor {
    const COMPOSITE_ONLY: bool = true;
}

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
    wPropertyNameLength: u16,
    PropertyName: &'a [u8],
    wPropertyDataLength: u16,
    PropertyData: &'a [u8],
}

impl<'a> DeviceLevelDescriptor for RegistryPropertyFeatureDescriptor<'a> {}
impl<'a> FunctionLevelDescriptor for RegistryPropertyFeatureDescriptor<'a> {
    const COMPOSITE_ONLY: bool = true;
}

impl<'a> Descriptor for RegistryPropertyFeatureDescriptor<'a> {
    const TYPE: DescriptorType = DescriptorType::FeatureRegProperty;
    fn size(&self) -> usize {
        10 + self.PropertyName.len() + self.PropertyData.len()
    }
    fn write_to(&self, buf: &mut [u8]) {
        assert!(buf.len() >= self.size());
        assert!(self.wPropertyNameLength as usize == self.PropertyName.len());
        assert!(self.wPropertyDataLength as usize == self.PropertyData.len());
        write_u16(buf, 0..2, self.wLength);
        write_u16(buf, 2..4, self.wDescriptorType);
        write_u16(buf, 4..6, self.wPropertyDataType);
        write_u16(buf, 6..8, self.wPropertyNameLength);
        let pne = 8 + self.PropertyName.len();
        (&mut buf[8..pne]).copy_from_slice(self.PropertyName);
        let pds = pne + 2;
        let pde = pds + self.PropertyData.len();
        write_u16(buf, pne..pds, self.wPropertyDataLength);
        (&mut buf[pds..pde]).copy_from_slice(self.PropertyData);
    }
}

impl<'a> RegistryPropertyFeatureDescriptor<'a> {
    /// A registry property.
    ///
    /// `name` should be a NUL-terminated 16-bit Unicode string.
    pub fn new_raw<'n: 'a, 'd: 'a>(name: &'a [u8], data: &'d [u8], data_type: PropertyDataType) -> Self {
        Self {
            wLength: ((10 + name.len() + data.len()) as u16).to_le(),
            wDescriptorType: (Self::TYPE as u16).to_le(),
            wPropertyDataType: (data_type as u16).to_le(),
            wPropertyNameLength: (name.len() as u16).to_le(),
            PropertyName: name,
            wPropertyDataLength: (data.len() as u16).to_le(),
            PropertyData: data,
        }
    }

    fn u16str_bytes(s: &U16CStr) -> &[u8] {
        unsafe { core::slice::from_raw_parts(s.as_ptr() as *const u8, (s.len() + 1) * 2) }
    }

    /// A registry property that sets the DeviceInterfaceGUIDs to the device interface class for USB devices which are
    /// attached to a USB hub.
    pub fn new_usb_deviceinterfaceguid() -> Self {
        // Can't use defmt::panic in constant expressions (inside u16cstr!)
        macro_rules! panic {
            ($($x:tt)*) => {
                {
                    ::core::panic!($($x)*);
                }
            };
        }

        Self::new_multi_string(
            u16cstr!("DeviceInterfaceGUIDs"),
            u16cstr!("{A5DCBF10-6530-11D2-901F-00C04FB951ED}").as_slice_with_nul(),
        )
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
