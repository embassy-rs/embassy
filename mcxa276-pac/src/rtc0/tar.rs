#[doc = "Register `TAR` reader"]
pub type R = crate::R<TarSpec>;
#[doc = "Register `TAR` writer"]
pub type W = crate::W<TarSpec>;
#[doc = "Field `TAR` reader - Time Alarm Register"]
pub type TarR = crate::FieldReader<u32>;
#[doc = "Field `TAR` writer - Time Alarm Register"]
pub type TarW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Time Alarm Register"]
    #[inline(always)]
    pub fn tar(&self) -> TarR {
        TarR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Time Alarm Register"]
    #[inline(always)]
    pub fn tar(&mut self) -> TarW<TarSpec> {
        TarW::new(self, 0)
    }
}
#[doc = "RTC Time Alarm\n\nYou can [`read`](crate::Reg::read) this register and get [`tar::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tar::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TarSpec;
impl crate::RegisterSpec for TarSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tar::R`](R) reader structure"]
impl crate::Readable for TarSpec {}
#[doc = "`write(|w| ..)` method takes [`tar::W`](W) writer structure"]
impl crate::Writable for TarSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TAR to value 0"]
impl crate::Resettable for TarSpec {}
