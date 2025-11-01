#[doc = "Register `SHIFTBUFEOS[%s]` reader"]
pub type R = crate::R<ShiftbufeosSpec>;
#[doc = "Register `SHIFTBUFEOS[%s]` writer"]
pub type W = crate::W<ShiftbufeosSpec>;
#[doc = "Field `SHIFTBUFEOS` reader - Shift Buffer"]
pub type ShiftbufeosR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFEOS` writer - Shift Buffer"]
pub type ShiftbufeosW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufeos(&self) -> ShiftbufeosR {
        ShiftbufeosR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufeos(&mut self) -> ShiftbufeosW<ShiftbufeosSpec> {
        ShiftbufeosW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Even Odd Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufeos::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufeos::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufeosSpec;
impl crate::RegisterSpec for ShiftbufeosSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufeos::R`](R) reader structure"]
impl crate::Readable for ShiftbufeosSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufeos::W`](W) writer structure"]
impl crate::Writable for ShiftbufeosSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFEOS[%s] to value 0"]
impl crate::Resettable for ShiftbufeosSpec {}
