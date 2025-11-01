#[doc = "Register `CR` reader"]
pub type R = crate::R<CrSpec>;
#[doc = "Register `CR` writer"]
pub type W = crate::W<CrSpec>;
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swr {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: Perform a software reset"]
    SwReset = 1,
}
impl From<Swr> for bool {
    #[inline(always)]
    fn from(variant: Swr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWR` reader - Software Reset"]
pub type SwrR = crate::BitReader<Swr>;
impl SwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swr {
        match self.bits {
            false => Swr::NoEffect,
            true => Swr::SwReset,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Swr::NoEffect
    }
    #[doc = "Perform a software reset"]
    #[inline(always)]
    pub fn is_sw_reset(&self) -> bool {
        *self == Swr::SwReset
    }
}
#[doc = "Field `SWR` writer - Software Reset"]
pub type SwrW<'a, REG> = crate::BitWriter<'a, REG, Swr>;
impl<'a, REG> SwrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Swr::NoEffect)
    }
    #[doc = "Perform a software reset"]
    #[inline(always)]
    pub fn sw_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Swr::SwReset)
    }
}
#[doc = "Digital Tamper Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Den {
    #[doc = "0: Disables TDET clock and prescaler"]
    Disable = 0,
    #[doc = "1: Enables TDET clock and prescaler"]
    Enable = 1,
}
impl From<Den> for bool {
    #[inline(always)]
    fn from(variant: Den) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DEN` reader - Digital Tamper Enable"]
pub type DenR = crate::BitReader<Den>;
impl DenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Den {
        match self.bits {
            false => Den::Disable,
            true => Den::Enable,
        }
    }
    #[doc = "Disables TDET clock and prescaler"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Den::Disable
    }
    #[doc = "Enables TDET clock and prescaler"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Den::Enable
    }
}
#[doc = "Field `DEN` writer - Digital Tamper Enable"]
pub type DenW<'a, REG> = crate::BitWriter<'a, REG, Den>;
impl<'a, REG> DenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables TDET clock and prescaler"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Den::Disable)
    }
    #[doc = "Enables TDET clock and prescaler"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Den::Enable)
    }
}
#[doc = "Tamper Force System Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tfsr {
    #[doc = "0: Do not force chip reset"]
    NoReset = 0,
    #[doc = "1: Force chip reset"]
    Reset = 1,
}
impl From<Tfsr> for bool {
    #[inline(always)]
    fn from(variant: Tfsr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TFSR` reader - Tamper Force System Reset"]
pub type TfsrR = crate::BitReader<Tfsr>;
impl TfsrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tfsr {
        match self.bits {
            false => Tfsr::NoReset,
            true => Tfsr::Reset,
        }
    }
    #[doc = "Do not force chip reset"]
    #[inline(always)]
    pub fn is_no_reset(&self) -> bool {
        *self == Tfsr::NoReset
    }
    #[doc = "Force chip reset"]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == Tfsr::Reset
    }
}
#[doc = "Field `TFSR` writer - Tamper Force System Reset"]
pub type TfsrW<'a, REG> = crate::BitWriter<'a, REG, Tfsr>;
impl<'a, REG> TfsrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Do not force chip reset"]
    #[inline(always)]
    pub fn no_reset(self) -> &'a mut crate::W<REG> {
        self.variant(Tfsr::NoReset)
    }
    #[doc = "Force chip reset"]
    #[inline(always)]
    pub fn reset(self) -> &'a mut crate::W<REG> {
        self.variant(Tfsr::Reset)
    }
}
#[doc = "Update Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Um {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: Allows the clearing of interrupts"]
    ClearInts = 1,
}
impl From<Um> for bool {
    #[inline(always)]
    fn from(variant: Um) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UM` reader - Update Mode"]
pub type UmR = crate::BitReader<Um>;
impl UmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Um {
        match self.bits {
            false => Um::NoEffect,
            true => Um::ClearInts,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Um::NoEffect
    }
    #[doc = "Allows the clearing of interrupts"]
    #[inline(always)]
    pub fn is_clear_ints(&self) -> bool {
        *self == Um::ClearInts
    }
}
#[doc = "Field `UM` writer - Update Mode"]
pub type UmW<'a, REG> = crate::BitWriter<'a, REG, Um>;
impl<'a, REG> UmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Um::NoEffect)
    }
    #[doc = "Allows the clearing of interrupts"]
    #[inline(always)]
    pub fn clear_ints(self) -> &'a mut crate::W<REG> {
        self.variant(Um::ClearInts)
    }
}
#[doc = "Disable Prescaler On Tamper\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distam {
    #[doc = "0: No effect"]
    NoEffect = 0,
    #[doc = "1: Automatically disables the prescaler after tamper detection"]
    AutoDis = 1,
}
impl From<Distam> for bool {
    #[inline(always)]
    fn from(variant: Distam) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DISTAM` reader - Disable Prescaler On Tamper"]
pub type DistamR = crate::BitReader<Distam>;
impl DistamR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Distam {
        match self.bits {
            false => Distam::NoEffect,
            true => Distam::AutoDis,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Distam::NoEffect
    }
    #[doc = "Automatically disables the prescaler after tamper detection"]
    #[inline(always)]
    pub fn is_auto_dis(&self) -> bool {
        *self == Distam::AutoDis
    }
}
#[doc = "Field `DISTAM` writer - Disable Prescaler On Tamper"]
pub type DistamW<'a, REG> = crate::BitWriter<'a, REG, Distam>;
impl<'a, REG> DistamW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Distam::NoEffect)
    }
    #[doc = "Automatically disables the prescaler after tamper detection"]
    #[inline(always)]
    pub fn auto_dis(self) -> &'a mut crate::W<REG> {
        self.variant(Distam::AutoDis)
    }
}
#[doc = "Field `DPR` reader - Digital Tamper Prescaler"]
pub type DprR = crate::FieldReader<u16>;
#[doc = "Field `DPR` writer - Digital Tamper Prescaler"]
pub type DprW<'a, REG> = crate::FieldWriter<'a, REG, 15, u16>;
impl R {
    #[doc = "Bit 0 - Software Reset"]
    #[inline(always)]
    pub fn swr(&self) -> SwrR {
        SwrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Digital Tamper Enable"]
    #[inline(always)]
    pub fn den(&self) -> DenR {
        DenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Tamper Force System Reset"]
    #[inline(always)]
    pub fn tfsr(&self) -> TfsrR {
        TfsrR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Update Mode"]
    #[inline(always)]
    pub fn um(&self) -> UmR {
        UmR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 8 - Disable Prescaler On Tamper"]
    #[inline(always)]
    pub fn distam(&self) -> DistamR {
        DistamR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 17:31 - Digital Tamper Prescaler"]
    #[inline(always)]
    pub fn dpr(&self) -> DprR {
        DprR::new(((self.bits >> 17) & 0x7fff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - Software Reset"]
    #[inline(always)]
    pub fn swr(&mut self) -> SwrW<CrSpec> {
        SwrW::new(self, 0)
    }
    #[doc = "Bit 1 - Digital Tamper Enable"]
    #[inline(always)]
    pub fn den(&mut self) -> DenW<CrSpec> {
        DenW::new(self, 1)
    }
    #[doc = "Bit 2 - Tamper Force System Reset"]
    #[inline(always)]
    pub fn tfsr(&mut self) -> TfsrW<CrSpec> {
        TfsrW::new(self, 2)
    }
    #[doc = "Bit 3 - Update Mode"]
    #[inline(always)]
    pub fn um(&mut self) -> UmW<CrSpec> {
        UmW::new(self, 3)
    }
    #[doc = "Bit 8 - Disable Prescaler On Tamper"]
    #[inline(always)]
    pub fn distam(&mut self) -> DistamW<CrSpec> {
        DistamW::new(self, 8)
    }
    #[doc = "Bits 17:31 - Digital Tamper Prescaler"]
    #[inline(always)]
    pub fn dpr(&mut self) -> DprW<CrSpec> {
        DprW::new(self, 17)
    }
}
#[doc = "Control\n\nYou can [`read`](crate::Reg::read) this register and get [`cr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CrSpec;
impl crate::RegisterSpec for CrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cr::R`](R) reader structure"]
impl crate::Readable for CrSpec {}
#[doc = "`write(|w| ..)` method takes [`cr::W`](W) writer structure"]
impl crate::Writable for CrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CR to value 0"]
impl crate::Resettable for CrSpec {}
