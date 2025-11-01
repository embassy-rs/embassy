#[doc = "Register `SHIFTBUF[%s]` reader"]
pub type R = crate::R<ShiftbufSpec>;
#[doc = "Register `SHIFTBUF[%s]` writer"]
pub type W = crate::W<ShiftbufSpec>;
#[doc = "Field `SHIFTBUF` reader - Shift Buffer"]
pub type ShiftbufR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUF` writer - Shift Buffer"]
pub type ShiftbufW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbuf(&self) -> ShiftbufR {
        ShiftbufR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbuf(&mut self) -> ShiftbufW<ShiftbufSpec> {
        ShiftbufW::new(self, 0)
    }
}
#[doc = "Shifter Buffer\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbuf::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbuf::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufSpec;
impl crate::RegisterSpec for ShiftbufSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbuf::R`](R) reader structure"]
impl crate::Readable for ShiftbufSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbuf::W`](W) writer structure"]
impl crate::Writable for ShiftbufSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUF[%s] to value 0"]
impl crate::Resettable for ShiftbufSpec {}
