#[doc = "Register `LINIT` reader"]
pub type R = crate::R<LinitSpec>;
#[doc = "Register `LINIT` writer"]
pub type W = crate::W<LinitSpec>;
#[doc = "Field `INIT` reader - INIT"]
pub type InitR = crate::FieldReader<u16>;
#[doc = "Field `INIT` writer - INIT"]
pub type InitW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - INIT"]
    #[inline(always)]
    pub fn init(&self) -> InitR {
        InitR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - INIT"]
    #[inline(always)]
    pub fn init(&mut self) -> InitW<LinitSpec> {
        InitW::new(self, 0)
    }
}
#[doc = "Lower Initialization Register\n\nYou can [`read`](crate::Reg::read) this register and get [`linit::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`linit::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LinitSpec;
impl crate::RegisterSpec for LinitSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`linit::R`](R) reader structure"]
impl crate::Readable for LinitSpec {}
#[doc = "`write(|w| ..)` method takes [`linit::W`](W) writer structure"]
impl crate::Writable for LinitSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LINIT to value 0"]
impl crate::Resettable for LinitSpec {}
