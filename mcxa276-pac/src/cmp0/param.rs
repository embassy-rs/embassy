#[doc = "Register `PARAM` reader"]
pub type R = crate::R<ParamSpec>;
#[doc = "DAC Resolution\n\nValue on reset: 2"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DacRes {
    #[doc = "0: 4-bit DAC"]
    Reso4 = 0,
    #[doc = "1: 6-bit DAC"]
    Reso6 = 1,
    #[doc = "2: 8-bit DAC"]
    Reso8 = 2,
    #[doc = "3: 10-bit DAC"]
    Reso10 = 3,
    #[doc = "4: 12-bit DAC"]
    Reso12 = 4,
    #[doc = "5: 14-bit DAC"]
    Reso14 = 5,
    #[doc = "6: 16-bit DAC"]
    Reso16 = 6,
}
impl From<DacRes> for u8 {
    #[inline(always)]
    fn from(variant: DacRes) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for DacRes {
    type Ux = u8;
}
impl crate::IsEnum for DacRes {}
#[doc = "Field `DAC_RES` reader - DAC Resolution"]
pub type DacResR = crate::FieldReader<DacRes>;
impl DacResR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<DacRes> {
        match self.bits {
            0 => Some(DacRes::Reso4),
            1 => Some(DacRes::Reso6),
            2 => Some(DacRes::Reso8),
            3 => Some(DacRes::Reso10),
            4 => Some(DacRes::Reso12),
            5 => Some(DacRes::Reso14),
            6 => Some(DacRes::Reso16),
            _ => None,
        }
    }
    #[doc = "4-bit DAC"]
    #[inline(always)]
    pub fn is_reso_4(&self) -> bool {
        *self == DacRes::Reso4
    }
    #[doc = "6-bit DAC"]
    #[inline(always)]
    pub fn is_reso_6(&self) -> bool {
        *self == DacRes::Reso6
    }
    #[doc = "8-bit DAC"]
    #[inline(always)]
    pub fn is_reso_8(&self) -> bool {
        *self == DacRes::Reso8
    }
    #[doc = "10-bit DAC"]
    #[inline(always)]
    pub fn is_reso_10(&self) -> bool {
        *self == DacRes::Reso10
    }
    #[doc = "12-bit DAC"]
    #[inline(always)]
    pub fn is_reso_12(&self) -> bool {
        *self == DacRes::Reso12
    }
    #[doc = "14-bit DAC"]
    #[inline(always)]
    pub fn is_reso_14(&self) -> bool {
        *self == DacRes::Reso14
    }
    #[doc = "16-bit DAC"]
    #[inline(always)]
    pub fn is_reso_16(&self) -> bool {
        *self == DacRes::Reso16
    }
}
impl R {
    #[doc = "Bits 0:3 - DAC Resolution"]
    #[inline(always)]
    pub fn dac_res(&self) -> DacResR {
        DacResR::new((self.bits & 0x0f) as u8)
    }
}
#[doc = "Parameter\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ParamSpec;
impl crate::RegisterSpec for ParamSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`param::R`](R) reader structure"]
impl crate::Readable for ParamSpec {}
#[doc = "`reset()` method sets PARAM to value 0x02"]
impl crate::Resettable for ParamSpec {
    const RESET_VALUE: u32 = 0x02;
}
