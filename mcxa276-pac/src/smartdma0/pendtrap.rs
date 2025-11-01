#[doc = "Register `PENDTRAP` reader"]
pub type R = crate::R<PendtrapSpec>;
#[doc = "Register `PENDTRAP` writer"]
pub type W = crate::W<PendtrapSpec>;
#[doc = "Field `STATUS` reader - Status Flag or Pending Trap Request"]
pub type StatusR = crate::FieldReader;
#[doc = "Field `STATUS` writer - Status Flag or Pending Trap Request"]
pub type StatusW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `POL` reader - Polarity"]
pub type PolR = crate::FieldReader;
#[doc = "Field `POL` writer - Polarity"]
pub type PolW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `EN` reader - Enable Pending Trap"]
pub type EnR = crate::FieldReader;
#[doc = "Field `EN` writer - Enable Pending Trap"]
pub type EnW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Status Flag or Pending Trap Request"]
    #[inline(always)]
    pub fn status(&self) -> StatusR {
        StatusR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Polarity"]
    #[inline(always)]
    pub fn pol(&self) -> PolR {
        PolR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Enable Pending Trap"]
    #[inline(always)]
    pub fn en(&self) -> EnR {
        EnR::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Status Flag or Pending Trap Request"]
    #[inline(always)]
    pub fn status(&mut self) -> StatusW<PendtrapSpec> {
        StatusW::new(self, 0)
    }
    #[doc = "Bits 8:15 - Polarity"]
    #[inline(always)]
    pub fn pol(&mut self) -> PolW<PendtrapSpec> {
        PolW::new(self, 8)
    }
    #[doc = "Bits 16:23 - Enable Pending Trap"]
    #[inline(always)]
    pub fn en(&mut self) -> EnW<PendtrapSpec> {
        EnW::new(self, 16)
    }
}
#[doc = "Pending Trap Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pendtrap::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pendtrap::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PendtrapSpec;
impl crate::RegisterSpec for PendtrapSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pendtrap::R`](R) reader structure"]
impl crate::Readable for PendtrapSpec {}
#[doc = "`write(|w| ..)` method takes [`pendtrap::W`](W) writer structure"]
impl crate::Writable for PendtrapSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PENDTRAP to value 0"]
impl crate::Resettable for PendtrapSpec {}
