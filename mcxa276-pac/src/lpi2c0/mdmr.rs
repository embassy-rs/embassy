#[doc = "Register `MDMR` reader"]
pub type R = crate::R<MdmrSpec>;
#[doc = "Register `MDMR` writer"]
pub type W = crate::W<MdmrSpec>;
#[doc = "Field `MATCH0` reader - Match 0 Value"]
pub type Match0R = crate::FieldReader;
#[doc = "Field `MATCH0` writer - Match 0 Value"]
pub type Match0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `MATCH1` reader - Match 1 Value"]
pub type Match1R = crate::FieldReader;
#[doc = "Field `MATCH1` writer - Match 1 Value"]
pub type Match1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Match 0 Value"]
    #[inline(always)]
    pub fn match0(&self) -> Match0R {
        Match0R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Match 1 Value"]
    #[inline(always)]
    pub fn match1(&self) -> Match1R {
        Match1R::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Match 0 Value"]
    #[inline(always)]
    pub fn match0(&mut self) -> Match0W<MdmrSpec> {
        Match0W::new(self, 0)
    }
    #[doc = "Bits 16:23 - Match 1 Value"]
    #[inline(always)]
    pub fn match1(&mut self) -> Match1W<MdmrSpec> {
        Match1W::new(self, 16)
    }
}
#[doc = "Controller Data Match\n\nYou can [`read`](crate::Reg::read) this register and get [`mdmr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mdmr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MdmrSpec;
impl crate::RegisterSpec for MdmrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mdmr::R`](R) reader structure"]
impl crate::Readable for MdmrSpec {}
#[doc = "`write(|w| ..)` method takes [`mdmr::W`](W) writer structure"]
impl crate::Writable for MdmrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MDMR to value 0"]
impl crate::Resettable for MdmrSpec {}
