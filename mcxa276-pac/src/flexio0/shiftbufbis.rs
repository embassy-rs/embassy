#[doc = "Register `SHIFTBUFBIS[%s]` reader"]
pub type R = crate::R<ShiftbufbisSpec>;
#[doc = "Register `SHIFTBUFBIS[%s]` writer"]
pub type W = crate::W<ShiftbufbisSpec>;
#[doc = "Field `SHIFTBUFBIS` reader - Shift Buffer"]
pub type ShiftbufbisR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFBIS` writer - Shift Buffer"]
pub type ShiftbufbisW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufbis(&self) -> ShiftbufbisR {
        ShiftbufbisR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufbis(&mut self) -> ShiftbufbisW<ShiftbufbisSpec> {
        ShiftbufbisW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Bit Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufbis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufbis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufbisSpec;
impl crate::RegisterSpec for ShiftbufbisSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufbis::R`](R) reader structure"]
impl crate::Readable for ShiftbufbisSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufbis::W`](W) writer structure"]
impl crate::Writable for ShiftbufbisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFBIS[%s] to value 0"]
impl crate::Resettable for ShiftbufbisSpec {}
