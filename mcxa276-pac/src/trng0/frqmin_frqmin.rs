#[doc = "Register `FRQMIN` reader"]
pub type R = crate::R<FrqminFrqminSpec>;
#[doc = "Register `FRQMIN` writer"]
pub type W = crate::W<FrqminFrqminSpec>;
#[doc = "Field `FRQ_MIN` reader - Frequency Count Minimum Limit"]
pub type FrqMinR = crate::FieldReader<u32>;
#[doc = "Field `FRQ_MIN` writer - Frequency Count Minimum Limit"]
pub type FrqMinW<'a, REG> = crate::FieldWriter<'a, REG, 22, u32>;
impl R {
    #[doc = "Bits 0:21 - Frequency Count Minimum Limit"]
    #[inline(always)]
    pub fn frq_min(&self) -> FrqMinR {
        FrqMinR::new(self.bits & 0x003f_ffff)
    }
}
impl W {
    #[doc = "Bits 0:21 - Frequency Count Minimum Limit"]
    #[inline(always)]
    pub fn frq_min(&mut self) -> FrqMinW<FrqminFrqminSpec> {
        FrqMinW::new(self, 0)
    }
}
#[doc = "Frequency Count Minimum Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`frqmin_frqmin::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frqmin_frqmin::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrqminFrqminSpec;
impl crate::RegisterSpec for FrqminFrqminSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`frqmin_frqmin::R`](R) reader structure"]
impl crate::Readable for FrqminFrqminSpec {}
#[doc = "`write(|w| ..)` method takes [`frqmin_frqmin::W`](W) writer structure"]
impl crate::Writable for FrqminFrqminSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FRQMIN to value 0x0640"]
impl crate::Resettable for FrqminFrqminSpec {
    const RESET_VALUE: u32 = 0x0640;
}
