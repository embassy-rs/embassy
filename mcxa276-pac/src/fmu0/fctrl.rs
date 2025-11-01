#[doc = "Register `FCTRL` reader"]
pub type R = crate::R<FctrlSpec>;
#[doc = "Register `FCTRL` writer"]
pub type W = crate::W<FctrlSpec>;
#[doc = "Field `RWSC` reader - Read Wait-State Control"]
pub type RwscR = crate::FieldReader;
#[doc = "Field `RWSC` writer - Read Wait-State Control"]
pub type RwscW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Low speed active mode\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lsactive {
    #[doc = "0: Full speed active mode requested"]
    Lsactive0 = 0,
    #[doc = "1: Low speed active mode requested"]
    Lsactive1 = 1,
}
impl From<Lsactive> for bool {
    #[inline(always)]
    fn from(variant: Lsactive) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LSACTIVE` reader - Low speed active mode"]
pub type LsactiveR = crate::BitReader<Lsactive>;
impl LsactiveR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lsactive {
        match self.bits {
            false => Lsactive::Lsactive0,
            true => Lsactive::Lsactive1,
        }
    }
    #[doc = "Full speed active mode requested"]
    #[inline(always)]
    pub fn is_lsactive0(&self) -> bool {
        *self == Lsactive::Lsactive0
    }
    #[doc = "Low speed active mode requested"]
    #[inline(always)]
    pub fn is_lsactive1(&self) -> bool {
        *self == Lsactive::Lsactive1
    }
}
#[doc = "Field `LSACTIVE` writer - Low speed active mode"]
pub type LsactiveW<'a, REG> = crate::BitWriter<'a, REG, Lsactive>;
impl<'a, REG> LsactiveW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Full speed active mode requested"]
    #[inline(always)]
    pub fn lsactive0(self) -> &'a mut crate::W<REG> {
        self.variant(Lsactive::Lsactive0)
    }
    #[doc = "Low speed active mode requested"]
    #[inline(always)]
    pub fn lsactive1(self) -> &'a mut crate::W<REG> {
        self.variant(Lsactive::Lsactive1)
    }
}
#[doc = "Force Double Bit Fault Detect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fdfd {
    #[doc = "0: FSTAT\\[DFDIF\\] sets only if a double bit fault is detected during a valid flash read access from the platform flash controller"]
    Fdfd0 = 0,
    #[doc = "1: FSTAT\\[DFDIF\\] sets during any valid flash read access from the platform flash controller. An interrupt request is generated if the DFDIE bit is set."]
    Fdfd1 = 1,
}
impl From<Fdfd> for bool {
    #[inline(always)]
    fn from(variant: Fdfd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FDFD` reader - Force Double Bit Fault Detect"]
pub type FdfdR = crate::BitReader<Fdfd>;
impl FdfdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fdfd {
        match self.bits {
            false => Fdfd::Fdfd0,
            true => Fdfd::Fdfd1,
        }
    }
    #[doc = "FSTAT\\[DFDIF\\] sets only if a double bit fault is detected during a valid flash read access from the platform flash controller"]
    #[inline(always)]
    pub fn is_fdfd0(&self) -> bool {
        *self == Fdfd::Fdfd0
    }
    #[doc = "FSTAT\\[DFDIF\\] sets during any valid flash read access from the platform flash controller. An interrupt request is generated if the DFDIE bit is set."]
    #[inline(always)]
    pub fn is_fdfd1(&self) -> bool {
        *self == Fdfd::Fdfd1
    }
}
#[doc = "Field `FDFD` writer - Force Double Bit Fault Detect"]
pub type FdfdW<'a, REG> = crate::BitWriter<'a, REG, Fdfd>;
impl<'a, REG> FdfdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FSTAT\\[DFDIF\\] sets only if a double bit fault is detected during a valid flash read access from the platform flash controller"]
    #[inline(always)]
    pub fn fdfd0(self) -> &'a mut crate::W<REG> {
        self.variant(Fdfd::Fdfd0)
    }
    #[doc = "FSTAT\\[DFDIF\\] sets during any valid flash read access from the platform flash controller. An interrupt request is generated if the DFDIE bit is set."]
    #[inline(always)]
    pub fn fdfd1(self) -> &'a mut crate::W<REG> {
        self.variant(Fdfd::Fdfd1)
    }
}
#[doc = "Abort Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Abtreq {
    #[doc = "0: No request to abort a command write sequence"]
    Abtreq0 = 0,
    #[doc = "1: Request to abort a command write sequence"]
    Abtreq1 = 1,
}
impl From<Abtreq> for bool {
    #[inline(always)]
    fn from(variant: Abtreq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ABTREQ` reader - Abort Request"]
pub type AbtreqR = crate::BitReader<Abtreq>;
impl AbtreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Abtreq {
        match self.bits {
            false => Abtreq::Abtreq0,
            true => Abtreq::Abtreq1,
        }
    }
    #[doc = "No request to abort a command write sequence"]
    #[inline(always)]
    pub fn is_abtreq0(&self) -> bool {
        *self == Abtreq::Abtreq0
    }
    #[doc = "Request to abort a command write sequence"]
    #[inline(always)]
    pub fn is_abtreq1(&self) -> bool {
        *self == Abtreq::Abtreq1
    }
}
#[doc = "Field `ABTREQ` writer - Abort Request"]
pub type AbtreqW<'a, REG> = crate::BitWriter<'a, REG, Abtreq>;
impl<'a, REG> AbtreqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No request to abort a command write sequence"]
    #[inline(always)]
    pub fn abtreq0(self) -> &'a mut crate::W<REG> {
        self.variant(Abtreq::Abtreq0)
    }
    #[doc = "Request to abort a command write sequence"]
    #[inline(always)]
    pub fn abtreq1(self) -> &'a mut crate::W<REG> {
        self.variant(Abtreq::Abtreq1)
    }
}
impl R {
    #[doc = "Bits 0:3 - Read Wait-State Control"]
    #[inline(always)]
    pub fn rwsc(&self) -> RwscR {
        RwscR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 8 - Low speed active mode"]
    #[inline(always)]
    pub fn lsactive(&self) -> LsactiveR {
        LsactiveR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 16 - Force Double Bit Fault Detect"]
    #[inline(always)]
    pub fn fdfd(&self) -> FdfdR {
        FdfdR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 24 - Abort Request"]
    #[inline(always)]
    pub fn abtreq(&self) -> AbtreqR {
        AbtreqR::new(((self.bits >> 24) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:3 - Read Wait-State Control"]
    #[inline(always)]
    pub fn rwsc(&mut self) -> RwscW<FctrlSpec> {
        RwscW::new(self, 0)
    }
    #[doc = "Bit 8 - Low speed active mode"]
    #[inline(always)]
    pub fn lsactive(&mut self) -> LsactiveW<FctrlSpec> {
        LsactiveW::new(self, 8)
    }
    #[doc = "Bit 16 - Force Double Bit Fault Detect"]
    #[inline(always)]
    pub fn fdfd(&mut self) -> FdfdW<FctrlSpec> {
        FdfdW::new(self, 16)
    }
    #[doc = "Bit 24 - Abort Request"]
    #[inline(always)]
    pub fn abtreq(&mut self) -> AbtreqW<FctrlSpec> {
        AbtreqW::new(self, 24)
    }
}
#[doc = "Flash Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FctrlSpec;
impl crate::RegisterSpec for FctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fctrl::R`](R) reader structure"]
impl crate::Readable for FctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`fctrl::W`](W) writer structure"]
impl crate::Writable for FctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FCTRL to value 0x0100"]
impl crate::Resettable for FctrlSpec {
    const RESET_VALUE: u32 = 0x0100;
}
