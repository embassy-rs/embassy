#[doc = "Register `SHIFTBUFOES[%s]` reader"]
pub type R = crate::R<ShiftbufoesSpec>;
#[doc = "Register `SHIFTBUFOES[%s]` writer"]
pub type W = crate::W<ShiftbufoesSpec>;
#[doc = "Field `SHIFTBUFOES` reader - Shift Buffer"]
pub type ShiftbufoesR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFOES` writer - Shift Buffer"]
pub type ShiftbufoesW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufoes(&self) -> ShiftbufoesR {
        ShiftbufoesR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufoes(&mut self) -> ShiftbufoesW<ShiftbufoesSpec> {
        ShiftbufoesW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Odd Even Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufoes::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufoes::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufoesSpec;
impl crate::RegisterSpec for ShiftbufoesSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufoes::R`](R) reader structure"]
impl crate::Readable for ShiftbufoesSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufoes::W`](W) writer structure"]
impl crate::Writable for ShiftbufoesSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFOES[%s] to value 0"]
impl crate::Resettable for ShiftbufoesSpec {}
