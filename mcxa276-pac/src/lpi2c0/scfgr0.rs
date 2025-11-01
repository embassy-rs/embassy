#[doc = "Register `SCFGR0` reader"]
pub type R = crate::R<Scfgr0Spec>;
#[doc = "Register `SCFGR0` writer"]
pub type W = crate::W<Scfgr0Spec>;
#[doc = "Read Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdreq {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rdreq> for bool {
    #[inline(always)]
    fn from(variant: Rdreq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDREQ` reader - Read Request"]
pub type RdreqR = crate::BitReader<Rdreq>;
impl RdreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdreq {
        match self.bits {
            false => Rdreq::Disabled,
            true => Rdreq::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdreq::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rdreq::Enabled
    }
}
#[doc = "Field `RDREQ` writer - Read Request"]
pub type RdreqW<'a, REG> = crate::BitWriter<'a, REG, Rdreq>;
impl<'a, REG> RdreqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdreq::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdreq::Enabled)
    }
}
#[doc = "Read Acknowledge Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdack {
    #[doc = "0: Read Request not acknowledged"]
    NotAcknowledged = 0,
    #[doc = "1: Read Request acknowledged"]
    Acknowledged = 1,
}
impl From<Rdack> for bool {
    #[inline(always)]
    fn from(variant: Rdack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDACK` reader - Read Acknowledge Flag"]
pub type RdackR = crate::BitReader<Rdack>;
impl RdackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdack {
        match self.bits {
            false => Rdack::NotAcknowledged,
            true => Rdack::Acknowledged,
        }
    }
    #[doc = "Read Request not acknowledged"]
    #[inline(always)]
    pub fn is_not_acknowledged(&self) -> bool {
        *self == Rdack::NotAcknowledged
    }
    #[doc = "Read Request acknowledged"]
    #[inline(always)]
    pub fn is_acknowledged(&self) -> bool {
        *self == Rdack::Acknowledged
    }
}
impl R {
    #[doc = "Bit 0 - Read Request"]
    #[inline(always)]
    pub fn rdreq(&self) -> RdreqR {
        RdreqR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Read Acknowledge Flag"]
    #[inline(always)]
    pub fn rdack(&self) -> RdackR {
        RdackR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Read Request"]
    #[inline(always)]
    pub fn rdreq(&mut self) -> RdreqW<Scfgr0Spec> {
        RdreqW::new(self, 0)
    }
}
#[doc = "Target Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`scfgr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scfgr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scfgr0Spec;
impl crate::RegisterSpec for Scfgr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scfgr0::R`](R) reader structure"]
impl crate::Readable for Scfgr0Spec {}
#[doc = "`write(|w| ..)` method takes [`scfgr0::W`](W) writer structure"]
impl crate::Writable for Scfgr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCFGR0 to value 0"]
impl crate::Resettable for Scfgr0Spec {}
