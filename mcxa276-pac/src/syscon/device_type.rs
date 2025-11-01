#[doc = "Register `DEVICE_TYPE` reader"]
pub type R = crate::R<DeviceTypeSpec>;
#[doc = "Field `DEVICE_TYPE_NUM` reader - Indicates the device part number"]
pub type DeviceTypeNumR = crate::FieldReader<u16>;
#[doc = "Indicates the device type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceTypeSec {
    #[doc = "0: Non Secure"]
    NonSec = 0,
    #[doc = "1: Secure"]
    Sec = 1,
}
impl From<DeviceTypeSec> for bool {
    #[inline(always)]
    fn from(variant: DeviceTypeSec) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DEVICE_TYPE_SEC` reader - Indicates the device type"]
pub type DeviceTypeSecR = crate::BitReader<DeviceTypeSec>;
impl DeviceTypeSecR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DeviceTypeSec {
        match self.bits {
            false => DeviceTypeSec::NonSec,
            true => DeviceTypeSec::Sec,
        }
    }
    #[doc = "Non Secure"]
    #[inline(always)]
    pub fn is_non_sec(&self) -> bool {
        *self == DeviceTypeSec::NonSec
    }
    #[doc = "Secure"]
    #[inline(always)]
    pub fn is_sec(&self) -> bool {
        *self == DeviceTypeSec::Sec
    }
}
#[doc = "Indicates the device's package type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceTypePkg {
    #[doc = "0: HLQFP"]
    Hlqfp = 0,
    #[doc = "1: HTQFP"]
    Htqfp = 1,
    #[doc = "2: BGA"]
    Bga = 2,
    #[doc = "3: HDQFP"]
    Hdqfp = 3,
    #[doc = "4: QFN"]
    Qfn = 4,
    #[doc = "5: CSP"]
    Csp = 5,
    #[doc = "6: LQFP"]
    Lqfp = 6,
}
impl From<DeviceTypePkg> for u8 {
    #[inline(always)]
    fn from(variant: DeviceTypePkg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for DeviceTypePkg {
    type Ux = u8;
}
impl crate::IsEnum for DeviceTypePkg {}
#[doc = "Field `DEVICE_TYPE_PKG` reader - Indicates the device's package type"]
pub type DeviceTypePkgR = crate::FieldReader<DeviceTypePkg>;
impl DeviceTypePkgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<DeviceTypePkg> {
        match self.bits {
            0 => Some(DeviceTypePkg::Hlqfp),
            1 => Some(DeviceTypePkg::Htqfp),
            2 => Some(DeviceTypePkg::Bga),
            3 => Some(DeviceTypePkg::Hdqfp),
            4 => Some(DeviceTypePkg::Qfn),
            5 => Some(DeviceTypePkg::Csp),
            6 => Some(DeviceTypePkg::Lqfp),
            _ => None,
        }
    }
    #[doc = "HLQFP"]
    #[inline(always)]
    pub fn is_hlqfp(&self) -> bool {
        *self == DeviceTypePkg::Hlqfp
    }
    #[doc = "HTQFP"]
    #[inline(always)]
    pub fn is_htqfp(&self) -> bool {
        *self == DeviceTypePkg::Htqfp
    }
    #[doc = "BGA"]
    #[inline(always)]
    pub fn is_bga(&self) -> bool {
        *self == DeviceTypePkg::Bga
    }
    #[doc = "HDQFP"]
    #[inline(always)]
    pub fn is_hdqfp(&self) -> bool {
        *self == DeviceTypePkg::Hdqfp
    }
    #[doc = "QFN"]
    #[inline(always)]
    pub fn is_qfn(&self) -> bool {
        *self == DeviceTypePkg::Qfn
    }
    #[doc = "CSP"]
    #[inline(always)]
    pub fn is_csp(&self) -> bool {
        *self == DeviceTypePkg::Csp
    }
    #[doc = "LQFP"]
    #[inline(always)]
    pub fn is_lqfp(&self) -> bool {
        *self == DeviceTypePkg::Lqfp
    }
}
#[doc = "Field `DEVICE_TYPE_PIN` reader - Indicates the device's pin number"]
pub type DeviceTypePinR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:15 - Indicates the device part number"]
    #[inline(always)]
    pub fn device_type_num(&self) -> DeviceTypeNumR {
        DeviceTypeNumR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bit 16 - Indicates the device type"]
    #[inline(always)]
    pub fn device_type_sec(&self) -> DeviceTypeSecR {
        DeviceTypeSecR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bits 20:23 - Indicates the device's package type"]
    #[inline(always)]
    pub fn device_type_pkg(&self) -> DeviceTypePkgR {
        DeviceTypePkgR::new(((self.bits >> 20) & 0x0f) as u8)
    }
    #[doc = "Bits 24:31 - Indicates the device's pin number"]
    #[inline(always)]
    pub fn device_type_pin(&self) -> DeviceTypePinR {
        DeviceTypePinR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
#[doc = "Device Type\n\nYou can [`read`](crate::Reg::read) this register and get [`device_type::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DeviceTypeSpec;
impl crate::RegisterSpec for DeviceTypeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`device_type::R`](R) reader structure"]
impl crate::Readable for DeviceTypeSpec {}
#[doc = "`reset()` method sets DEVICE_TYPE to value 0x2000"]
impl crate::Resettable for DeviceTypeSpec {
    const RESET_VALUE: u32 = 0x2000;
}
