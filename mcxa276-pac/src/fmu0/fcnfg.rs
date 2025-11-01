#[doc = "Register `FCNFG` reader"]
pub type R = crate::R<FcnfgSpec>;
#[doc = "Register `FCNFG` writer"]
pub type W = crate::W<FcnfgSpec>;
#[doc = "Command Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ccie {
    #[doc = "0: Command complete interrupt disabled"]
    Ccie0 = 0,
    #[doc = "1: Command complete interrupt enabled"]
    Ccie1 = 1,
}
impl From<Ccie> for bool {
    #[inline(always)]
    fn from(variant: Ccie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CCIE` reader - Command Complete Interrupt Enable"]
pub type CcieR = crate::BitReader<Ccie>;
impl CcieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ccie {
        match self.bits {
            false => Ccie::Ccie0,
            true => Ccie::Ccie1,
        }
    }
    #[doc = "Command complete interrupt disabled"]
    #[inline(always)]
    pub fn is_ccie0(&self) -> bool {
        *self == Ccie::Ccie0
    }
    #[doc = "Command complete interrupt enabled"]
    #[inline(always)]
    pub fn is_ccie1(&self) -> bool {
        *self == Ccie::Ccie1
    }
}
#[doc = "Field `CCIE` writer - Command Complete Interrupt Enable"]
pub type CcieW<'a, REG> = crate::BitWriter<'a, REG, Ccie>;
impl<'a, REG> CcieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Command complete interrupt disabled"]
    #[inline(always)]
    pub fn ccie0(self) -> &'a mut crate::W<REG> {
        self.variant(Ccie::Ccie0)
    }
    #[doc = "Command complete interrupt enabled"]
    #[inline(always)]
    pub fn ccie1(self) -> &'a mut crate::W<REG> {
        self.variant(Ccie::Ccie1)
    }
}
#[doc = "Mass Erase Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ersreq {
    #[doc = "0: No request or request complete"]
    Ersreq0 = 0,
    #[doc = "1: Request to run the Mass Erase operation"]
    Ersreq1 = 1,
}
impl From<Ersreq> for bool {
    #[inline(always)]
    fn from(variant: Ersreq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERSREQ` reader - Mass Erase Request"]
pub type ErsreqR = crate::BitReader<Ersreq>;
impl ErsreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ersreq {
        match self.bits {
            false => Ersreq::Ersreq0,
            true => Ersreq::Ersreq1,
        }
    }
    #[doc = "No request or request complete"]
    #[inline(always)]
    pub fn is_ersreq0(&self) -> bool {
        *self == Ersreq::Ersreq0
    }
    #[doc = "Request to run the Mass Erase operation"]
    #[inline(always)]
    pub fn is_ersreq1(&self) -> bool {
        *self == Ersreq::Ersreq1
    }
}
#[doc = "Double Bit Fault Detect Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dfdie {
    #[doc = "0: Double bit fault detect interrupt disabled"]
    Dfdie0 = 0,
    #[doc = "1: Double bit fault detect interrupt enabled"]
    Dfdie1 = 1,
}
impl From<Dfdie> for bool {
    #[inline(always)]
    fn from(variant: Dfdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DFDIE` reader - Double Bit Fault Detect Interrupt Enable"]
pub type DfdieR = crate::BitReader<Dfdie>;
impl DfdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dfdie {
        match self.bits {
            false => Dfdie::Dfdie0,
            true => Dfdie::Dfdie1,
        }
    }
    #[doc = "Double bit fault detect interrupt disabled"]
    #[inline(always)]
    pub fn is_dfdie0(&self) -> bool {
        *self == Dfdie::Dfdie0
    }
    #[doc = "Double bit fault detect interrupt enabled"]
    #[inline(always)]
    pub fn is_dfdie1(&self) -> bool {
        *self == Dfdie::Dfdie1
    }
}
#[doc = "Field `DFDIE` writer - Double Bit Fault Detect Interrupt Enable"]
pub type DfdieW<'a, REG> = crate::BitWriter<'a, REG, Dfdie>;
impl<'a, REG> DfdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Double bit fault detect interrupt disabled"]
    #[inline(always)]
    pub fn dfdie0(self) -> &'a mut crate::W<REG> {
        self.variant(Dfdie::Dfdie0)
    }
    #[doc = "Double bit fault detect interrupt enabled"]
    #[inline(always)]
    pub fn dfdie1(self) -> &'a mut crate::W<REG> {
        self.variant(Dfdie::Dfdie1)
    }
}
#[doc = "Erase IFR Sector Enable - Block 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ersien0 {
    #[doc = "0: Block 0 IFR Sector X is protected from erase by ERSSCR command"]
    Ersien00 = 0,
    #[doc = "1: Block 0 IFR Sector X is not protected from erase by ERSSCR command"]
    Ersien01 = 1,
}
impl From<Ersien0> for u8 {
    #[inline(always)]
    fn from(variant: Ersien0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ersien0 {
    type Ux = u8;
}
impl crate::IsEnum for Ersien0 {}
#[doc = "Field `ERSIEN0` reader - Erase IFR Sector Enable - Block 0"]
pub type Ersien0R = crate::FieldReader<Ersien0>;
impl Ersien0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ersien0> {
        match self.bits {
            0 => Some(Ersien0::Ersien00),
            1 => Some(Ersien0::Ersien01),
            _ => None,
        }
    }
    #[doc = "Block 0 IFR Sector X is protected from erase by ERSSCR command"]
    #[inline(always)]
    pub fn is_ersien00(&self) -> bool {
        *self == Ersien0::Ersien00
    }
    #[doc = "Block 0 IFR Sector X is not protected from erase by ERSSCR command"]
    #[inline(always)]
    pub fn is_ersien01(&self) -> bool {
        *self == Ersien0::Ersien01
    }
}
#[doc = "Erase IFR Sector Enable - Block 1 (for dual block configs)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ersien1 {
    #[doc = "0: Block 1 IFR Sector X is protected from erase by ERSSCR command"]
    Ersien10 = 0,
    #[doc = "1: Block 1 IFR Sector X is not protected from erase by ERSSCR command"]
    Ersien11 = 1,
}
impl From<Ersien1> for u8 {
    #[inline(always)]
    fn from(variant: Ersien1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ersien1 {
    type Ux = u8;
}
impl crate::IsEnum for Ersien1 {}
#[doc = "Field `ERSIEN1` reader - Erase IFR Sector Enable - Block 1 (for dual block configs)"]
pub type Ersien1R = crate::FieldReader<Ersien1>;
impl Ersien1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ersien1> {
        match self.bits {
            0 => Some(Ersien1::Ersien10),
            1 => Some(Ersien1::Ersien11),
            _ => None,
        }
    }
    #[doc = "Block 1 IFR Sector X is protected from erase by ERSSCR command"]
    #[inline(always)]
    pub fn is_ersien10(&self) -> bool {
        *self == Ersien1::Ersien10
    }
    #[doc = "Block 1 IFR Sector X is not protected from erase by ERSSCR command"]
    #[inline(always)]
    pub fn is_ersien11(&self) -> bool {
        *self == Ersien1::Ersien11
    }
}
impl R {
    #[doc = "Bit 7 - Command Complete Interrupt Enable"]
    #[inline(always)]
    pub fn ccie(&self) -> CcieR {
        CcieR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Mass Erase Request"]
    #[inline(always)]
    pub fn ersreq(&self) -> ErsreqR {
        ErsreqR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 16 - Double Bit Fault Detect Interrupt Enable"]
    #[inline(always)]
    pub fn dfdie(&self) -> DfdieR {
        DfdieR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bits 24:27 - Erase IFR Sector Enable - Block 0"]
    #[inline(always)]
    pub fn ersien0(&self) -> Ersien0R {
        Ersien0R::new(((self.bits >> 24) & 0x0f) as u8)
    }
    #[doc = "Bits 28:31 - Erase IFR Sector Enable - Block 1 (for dual block configs)"]
    #[inline(always)]
    pub fn ersien1(&self) -> Ersien1R {
        Ersien1R::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 7 - Command Complete Interrupt Enable"]
    #[inline(always)]
    pub fn ccie(&mut self) -> CcieW<FcnfgSpec> {
        CcieW::new(self, 7)
    }
    #[doc = "Bit 16 - Double Bit Fault Detect Interrupt Enable"]
    #[inline(always)]
    pub fn dfdie(&mut self) -> DfdieW<FcnfgSpec> {
        DfdieW::new(self, 16)
    }
}
#[doc = "Flash Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fcnfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fcnfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FcnfgSpec;
impl crate::RegisterSpec for FcnfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fcnfg::R`](R) reader structure"]
impl crate::Readable for FcnfgSpec {}
#[doc = "`write(|w| ..)` method takes [`fcnfg::W`](W) writer structure"]
impl crate::Writable for FcnfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FCNFG to value 0"]
impl crate::Resettable for FcnfgSpec {}
