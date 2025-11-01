#[doc = "Register `SHIFTSTATE` reader"]
pub type R = crate::R<ShiftstateSpec>;
#[doc = "Register `SHIFTSTATE` writer"]
pub type W = crate::W<ShiftstateSpec>;
#[doc = "Field `STATE` reader - Current State Pointer"]
pub type StateR = crate::FieldReader;
#[doc = "Field `STATE` writer - Current State Pointer"]
pub type StateW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:2 - Current State Pointer"]
    #[inline(always)]
    pub fn state(&self) -> StateR {
        StateR::new((self.bits & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Current State Pointer"]
    #[inline(always)]
    pub fn state(&mut self) -> StateW<ShiftstateSpec> {
        StateW::new(self, 0)
    }
}
#[doc = "Shifter State\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftstate::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftstate::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftstateSpec;
impl crate::RegisterSpec for ShiftstateSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftstate::R`](R) reader structure"]
impl crate::Readable for ShiftstateSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftstate::W`](W) writer structure"]
impl crate::Writable for ShiftstateSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTSTATE to value 0"]
impl crate::Resettable for ShiftstateSpec {}
