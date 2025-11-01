#[doc = "Register `PC` reader"]
pub type R = crate::R<PcSpec>;
#[doc = "Register `PC` writer"]
pub type W = crate::W<PcSpec>;
#[doc = "Field `PCVAL` reader - Prescale Counter Value"]
pub type PcvalR = crate::FieldReader<u32>;
#[doc = "Field `PCVAL` writer - Prescale Counter Value"]
pub type PcvalW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Prescale Counter Value"]
    #[inline(always)]
    pub fn pcval(&self) -> PcvalR {
        PcvalR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Prescale Counter Value"]
    #[inline(always)]
    pub fn pcval(&mut self) -> PcvalW<PcSpec> {
        PcvalW::new(self, 0)
    }
}
#[doc = "Prescale Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`pc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PcSpec;
impl crate::RegisterSpec for PcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pc::R`](R) reader structure"]
impl crate::Readable for PcSpec {}
#[doc = "`write(|w| ..)` method takes [`pc::W`](W) writer structure"]
impl crate::Writable for PcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PC to value 0"]
impl crate::Resettable for PcSpec {}
