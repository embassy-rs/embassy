#[doc = "Register `SMAPCTRL0` reader"]
pub type R = crate::R<Smapctrl0Spec>;
#[doc = "Enable Primary Dynamic Address\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ena {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
}
impl From<Ena> for bool {
    #[inline(always)]
    fn from(variant: Ena) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ENA` reader - Enable Primary Dynamic Address"]
pub type EnaR = crate::BitReader<Ena>;
impl EnaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ena {
        match self.bits {
            false => Ena::Disable,
            true => Ena::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ena::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ena::Enable
    }
}
#[doc = "Field `DA` reader - Dynamic Address"]
pub type DaR = crate::FieldReader;
#[doc = "Cause\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cause {
    #[doc = "0: No information (this value occurs when not configured to write DA)"]
    None = 0,
    #[doc = "1: Set using ENTDAA"]
    Entdaa = 1,
    #[doc = "2: Set using SETDASA, SETAASA, or SETNEWDA"]
    Setdasa = 2,
    #[doc = "3: Cleared using RSTDAA"]
    Rstdaa = 3,
    #[doc = "4: Auto MAP change happened last"]
    Automap = 4,
}
impl From<Cause> for u8 {
    #[inline(always)]
    fn from(variant: Cause) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cause {
    type Ux = u8;
}
impl crate::IsEnum for Cause {}
#[doc = "Field `CAUSE` reader - Cause"]
pub type CauseR = crate::FieldReader<Cause>;
impl CauseR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cause> {
        match self.bits {
            0 => Some(Cause::None),
            1 => Some(Cause::Entdaa),
            2 => Some(Cause::Setdasa),
            3 => Some(Cause::Rstdaa),
            4 => Some(Cause::Automap),
            _ => None,
        }
    }
    #[doc = "No information (this value occurs when not configured to write DA)"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Cause::None
    }
    #[doc = "Set using ENTDAA"]
    #[inline(always)]
    pub fn is_entdaa(&self) -> bool {
        *self == Cause::Entdaa
    }
    #[doc = "Set using SETDASA, SETAASA, or SETNEWDA"]
    #[inline(always)]
    pub fn is_setdasa(&self) -> bool {
        *self == Cause::Setdasa
    }
    #[doc = "Cleared using RSTDAA"]
    #[inline(always)]
    pub fn is_rstdaa(&self) -> bool {
        *self == Cause::Rstdaa
    }
    #[doc = "Auto MAP change happened last"]
    #[inline(always)]
    pub fn is_automap(&self) -> bool {
        *self == Cause::Automap
    }
}
impl R {
    #[doc = "Bit 0 - Enable Primary Dynamic Address"]
    #[inline(always)]
    pub fn ena(&self) -> EnaR {
        EnaR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:7 - Dynamic Address"]
    #[inline(always)]
    pub fn da(&self) -> DaR {
        DaR::new(((self.bits >> 1) & 0x7f) as u8)
    }
    #[doc = "Bits 8:10 - Cause"]
    #[inline(always)]
    pub fn cause(&self) -> CauseR {
        CauseR::new(((self.bits >> 8) & 7) as u8)
    }
}
#[doc = "Map Feature Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`smapctrl0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Smapctrl0Spec;
impl crate::RegisterSpec for Smapctrl0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`smapctrl0::R`](R) reader structure"]
impl crate::Readable for Smapctrl0Spec {}
#[doc = "`reset()` method sets SMAPCTRL0 to value 0"]
impl crate::Resettable for Smapctrl0Spec {}
