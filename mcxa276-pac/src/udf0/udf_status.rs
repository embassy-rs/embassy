#[doc = "Register `udf_status` reader"]
pub type R = crate::R<UdfStatusSpec>;
#[doc = "Status bits\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OStatus {
    #[doc = "1: 5'b00001 = Reset"]
    Reset = 1,
    #[doc = "2: 5'b00010 = Init"]
    Init = 2,
    #[doc = "4: 5'b00100 = Warmup"]
    Warmup = 4,
    #[doc = "8: 5'b01000 = Ready"]
    Ready = 8,
    #[doc = "16: 5'b10000 = Error"]
    Error = 16,
}
impl From<OStatus> for u8 {
    #[inline(always)]
    fn from(variant: OStatus) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OStatus {
    type Ux = u8;
}
impl crate::IsEnum for OStatus {}
#[doc = "Field `o_status` reader - Status bits"]
pub type OStatusR = crate::FieldReader<OStatus>;
impl OStatusR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<OStatus> {
        match self.bits {
            1 => Some(OStatus::Reset),
            2 => Some(OStatus::Init),
            4 => Some(OStatus::Warmup),
            8 => Some(OStatus::Ready),
            16 => Some(OStatus::Error),
            _ => None,
        }
    }
    #[doc = "5'b00001 = Reset"]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == OStatus::Reset
    }
    #[doc = "5'b00010 = Init"]
    #[inline(always)]
    pub fn is_init(&self) -> bool {
        *self == OStatus::Init
    }
    #[doc = "5'b00100 = Warmup"]
    #[inline(always)]
    pub fn is_warmup(&self) -> bool {
        *self == OStatus::Warmup
    }
    #[doc = "5'b01000 = Ready"]
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        *self == OStatus::Ready
    }
    #[doc = "5'b10000 = Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == OStatus::Error
    }
}
#[doc = "Field `rsv` reader - RFU"]
pub type RsvR = crate::FieldReader<u32>;
#[doc = "Field `o_wait` reader - Indicates UDF is processing data"]
pub type OWaitR = crate::BitReader;
impl R {
    #[doc = "Bits 0:4 - Status bits"]
    #[inline(always)]
    pub fn o_status(&self) -> OStatusR {
        OStatusR::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 5:30 - RFU"]
    #[inline(always)]
    pub fn rsv(&self) -> RsvR {
        RsvR::new((self.bits >> 5) & 0x03ff_ffff)
    }
    #[doc = "Bit 31 - Indicates UDF is processing data"]
    #[inline(always)]
    pub fn o_wait(&self) -> OWaitR {
        OWaitR::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Status register\n\nYou can [`read`](crate::Reg::read) this register and get [`udf_status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UdfStatusSpec;
impl crate::RegisterSpec for UdfStatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`udf_status::R`](R) reader structure"]
impl crate::Readable for UdfStatusSpec {}
#[doc = "`reset()` method sets udf_status to value 0x01"]
impl crate::Resettable for UdfStatusSpec {
    const RESET_VALUE: u32 = 0x01;
}
