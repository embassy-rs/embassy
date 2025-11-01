#[doc = "Register `DEVICE_ID0` reader"]
pub type R = crate::R<DeviceId0Spec>;
#[doc = "Indicates the device's ram size\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RamSize {
    #[doc = "0: 8KB."]
    Size8kb = 0,
    #[doc = "1: 16KB."]
    Size16kb = 1,
    #[doc = "2: 32KB."]
    Size32kb = 2,
    #[doc = "3: 64KB."]
    Size64kb = 3,
    #[doc = "4: 96KB."]
    Size96kb = 4,
    #[doc = "5: 128KB."]
    Size128kb = 5,
    #[doc = "6: 160KB."]
    Size160kb = 6,
    #[doc = "7: 192KB."]
    Size192kb = 7,
    #[doc = "8: 256KB."]
    Size256kb = 8,
    #[doc = "9: 288KB."]
    Size288kb = 9,
    #[doc = "10: 352KB."]
    Size352kb = 10,
    #[doc = "11: 512KB."]
    Size512kb = 11,
}
impl From<RamSize> for u8 {
    #[inline(always)]
    fn from(variant: RamSize) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for RamSize {
    type Ux = u8;
}
impl crate::IsEnum for RamSize {}
#[doc = "Field `RAM_SIZE` reader - Indicates the device's ram size"]
pub type RamSizeR = crate::FieldReader<RamSize>;
impl RamSizeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<RamSize> {
        match self.bits {
            0 => Some(RamSize::Size8kb),
            1 => Some(RamSize::Size16kb),
            2 => Some(RamSize::Size32kb),
            3 => Some(RamSize::Size64kb),
            4 => Some(RamSize::Size96kb),
            5 => Some(RamSize::Size128kb),
            6 => Some(RamSize::Size160kb),
            7 => Some(RamSize::Size192kb),
            8 => Some(RamSize::Size256kb),
            9 => Some(RamSize::Size288kb),
            10 => Some(RamSize::Size352kb),
            11 => Some(RamSize::Size512kb),
            _ => None,
        }
    }
    #[doc = "8KB."]
    #[inline(always)]
    pub fn is_size_8kb(&self) -> bool {
        *self == RamSize::Size8kb
    }
    #[doc = "16KB."]
    #[inline(always)]
    pub fn is_size_16kb(&self) -> bool {
        *self == RamSize::Size16kb
    }
    #[doc = "32KB."]
    #[inline(always)]
    pub fn is_size_32kb(&self) -> bool {
        *self == RamSize::Size32kb
    }
    #[doc = "64KB."]
    #[inline(always)]
    pub fn is_size_64kb(&self) -> bool {
        *self == RamSize::Size64kb
    }
    #[doc = "96KB."]
    #[inline(always)]
    pub fn is_size_96kb(&self) -> bool {
        *self == RamSize::Size96kb
    }
    #[doc = "128KB."]
    #[inline(always)]
    pub fn is_size_128kb(&self) -> bool {
        *self == RamSize::Size128kb
    }
    #[doc = "160KB."]
    #[inline(always)]
    pub fn is_size_160kb(&self) -> bool {
        *self == RamSize::Size160kb
    }
    #[doc = "192KB."]
    #[inline(always)]
    pub fn is_size_192kb(&self) -> bool {
        *self == RamSize::Size192kb
    }
    #[doc = "256KB."]
    #[inline(always)]
    pub fn is_size_256kb(&self) -> bool {
        *self == RamSize::Size256kb
    }
    #[doc = "288KB."]
    #[inline(always)]
    pub fn is_size_288kb(&self) -> bool {
        *self == RamSize::Size288kb
    }
    #[doc = "352KB."]
    #[inline(always)]
    pub fn is_size_352kb(&self) -> bool {
        *self == RamSize::Size352kb
    }
    #[doc = "512KB."]
    #[inline(always)]
    pub fn is_size_512kb(&self) -> bool {
        *self == RamSize::Size512kb
    }
}
#[doc = "Indicates the device's flash size\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FlashSize {
    #[doc = "0: 32KB."]
    Size32kb = 0,
    #[doc = "1: 64KB."]
    Size64kb = 1,
    #[doc = "2: 128KB."]
    Size128kb = 2,
    #[doc = "3: 256KB."]
    Size256kb = 3,
    #[doc = "4: 512KB."]
    Size512kb = 4,
    #[doc = "5: 768KB."]
    Size768kb = 5,
    #[doc = "6: 1MB."]
    Size1mb = 6,
    #[doc = "7: 1.5MB."]
    Size1p5mb = 7,
    #[doc = "8: 2MB."]
    Size2mb = 8,
}
impl From<FlashSize> for u8 {
    #[inline(always)]
    fn from(variant: FlashSize) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for FlashSize {
    type Ux = u8;
}
impl crate::IsEnum for FlashSize {}
#[doc = "Field `FLASH_SIZE` reader - Indicates the device's flash size"]
pub type FlashSizeR = crate::FieldReader<FlashSize>;
impl FlashSizeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<FlashSize> {
        match self.bits {
            0 => Some(FlashSize::Size32kb),
            1 => Some(FlashSize::Size64kb),
            2 => Some(FlashSize::Size128kb),
            3 => Some(FlashSize::Size256kb),
            4 => Some(FlashSize::Size512kb),
            5 => Some(FlashSize::Size768kb),
            6 => Some(FlashSize::Size1mb),
            7 => Some(FlashSize::Size1p5mb),
            8 => Some(FlashSize::Size2mb),
            _ => None,
        }
    }
    #[doc = "32KB."]
    #[inline(always)]
    pub fn is_size_32kb(&self) -> bool {
        *self == FlashSize::Size32kb
    }
    #[doc = "64KB."]
    #[inline(always)]
    pub fn is_size_64kb(&self) -> bool {
        *self == FlashSize::Size64kb
    }
    #[doc = "128KB."]
    #[inline(always)]
    pub fn is_size_128kb(&self) -> bool {
        *self == FlashSize::Size128kb
    }
    #[doc = "256KB."]
    #[inline(always)]
    pub fn is_size_256kb(&self) -> bool {
        *self == FlashSize::Size256kb
    }
    #[doc = "512KB."]
    #[inline(always)]
    pub fn is_size_512kb(&self) -> bool {
        *self == FlashSize::Size512kb
    }
    #[doc = "768KB."]
    #[inline(always)]
    pub fn is_size_768kb(&self) -> bool {
        *self == FlashSize::Size768kb
    }
    #[doc = "1MB."]
    #[inline(always)]
    pub fn is_size_1mb(&self) -> bool {
        *self == FlashSize::Size1mb
    }
    #[doc = "1.5MB."]
    #[inline(always)]
    pub fn is_size_1p5mb(&self) -> bool {
        *self == FlashSize::Size1p5mb
    }
    #[doc = "2MB."]
    #[inline(always)]
    pub fn is_size_2mb(&self) -> bool {
        *self == FlashSize::Size2mb
    }
}
#[doc = "Field `ROM_REV_MINOR` reader - Indicates the device's ROM revision"]
pub type RomRevMinorR = crate::FieldReader;
#[doc = "no description available\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Security {
    #[doc = "5: Secure version."]
    NonSec = 5,
    #[doc = "10: Non secure version."]
    Security10 = 10,
}
impl From<Security> for u8 {
    #[inline(always)]
    fn from(variant: Security) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Security {
    type Ux = u8;
}
impl crate::IsEnum for Security {}
#[doc = "Field `SECURITY` reader - no description available"]
pub type SecurityR = crate::FieldReader<Security>;
impl SecurityR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Security> {
        match self.bits {
            5 => Some(Security::NonSec),
            10 => Some(Security::Security10),
            _ => None,
        }
    }
    #[doc = "Secure version."]
    #[inline(always)]
    pub fn is_non_sec(&self) -> bool {
        *self == Security::NonSec
    }
    #[doc = "Non secure version."]
    #[inline(always)]
    pub fn is_security_10(&self) -> bool {
        *self == Security::Security10
    }
}
impl R {
    #[doc = "Bits 0:3 - Indicates the device's ram size"]
    #[inline(always)]
    pub fn ram_size(&self) -> RamSizeR {
        RamSizeR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Indicates the device's flash size"]
    #[inline(always)]
    pub fn flash_size(&self) -> FlashSizeR {
        FlashSizeR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 20:23 - Indicates the device's ROM revision"]
    #[inline(always)]
    pub fn rom_rev_minor(&self) -> RomRevMinorR {
        RomRevMinorR::new(((self.bits >> 20) & 0x0f) as u8)
    }
    #[doc = "Bits 24:27 - no description available"]
    #[inline(always)]
    pub fn security(&self) -> SecurityR {
        SecurityR::new(((self.bits >> 24) & 0x0f) as u8)
    }
}
#[doc = "Device ID\n\nYou can [`read`](crate::Reg::read) this register and get [`device_id0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DeviceId0Spec;
impl crate::RegisterSpec for DeviceId0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`device_id0::R`](R) reader structure"]
impl crate::Readable for DeviceId0Spec {}
#[doc = "`reset()` method sets DEVICE_ID0 to value 0"]
impl crate::Resettable for DeviceId0Spec {}
