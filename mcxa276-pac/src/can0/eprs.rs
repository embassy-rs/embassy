#[doc = "Register `EPRS` reader"]
pub type R = crate::R<EprsSpec>;
#[doc = "Register `EPRS` writer"]
pub type W = crate::W<EprsSpec>;
#[doc = "Field `ENPRESDIV` reader - Extended Nominal Prescaler Division Factor"]
pub type EnpresdivR = crate::FieldReader<u16>;
#[doc = "Field `ENPRESDIV` writer - Extended Nominal Prescaler Division Factor"]
pub type EnpresdivW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `EDPRESDIV` reader - Extended Data Phase Prescaler Division Factor"]
pub type EdpresdivR = crate::FieldReader<u16>;
#[doc = "Field `EDPRESDIV` writer - Extended Data Phase Prescaler Division Factor"]
pub type EdpresdivW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - Extended Nominal Prescaler Division Factor"]
    #[inline(always)]
    pub fn enpresdiv(&self) -> EnpresdivR {
        EnpresdivR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 16:25 - Extended Data Phase Prescaler Division Factor"]
    #[inline(always)]
    pub fn edpresdiv(&self) -> EdpresdivR {
        EdpresdivR::new(((self.bits >> 16) & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Extended Nominal Prescaler Division Factor"]
    #[inline(always)]
    pub fn enpresdiv(&mut self) -> EnpresdivW<EprsSpec> {
        EnpresdivW::new(self, 0)
    }
    #[doc = "Bits 16:25 - Extended Data Phase Prescaler Division Factor"]
    #[inline(always)]
    pub fn edpresdiv(&mut self) -> EdpresdivW<EprsSpec> {
        EdpresdivW::new(self, 16)
    }
}
#[doc = "Enhanced CAN Bit Timing Prescalers\n\nYou can [`read`](crate::Reg::read) this register and get [`eprs::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eprs::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EprsSpec;
impl crate::RegisterSpec for EprsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`eprs::R`](R) reader structure"]
impl crate::Readable for EprsSpec {}
#[doc = "`write(|w| ..)` method takes [`eprs::W`](W) writer structure"]
impl crate::Writable for EprsSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EPRS to value 0"]
impl crate::Resettable for EprsSpec {}
