#[doc = "Register `SAMR` reader"]
pub type R = crate::R<SamrSpec>;
#[doc = "Register `SAMR` writer"]
pub type W = crate::W<SamrSpec>;
#[doc = "Field `ADDR0` reader - Address 0 Value"]
pub type Addr0R = crate::FieldReader<u16>;
#[doc = "Field `ADDR0` writer - Address 0 Value"]
pub type Addr0W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `ADDR1` reader - Address 1 Value"]
pub type Addr1R = crate::FieldReader<u16>;
#[doc = "Field `ADDR1` writer - Address 1 Value"]
pub type Addr1W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 1:10 - Address 0 Value"]
    #[inline(always)]
    pub fn addr0(&self) -> Addr0R {
        Addr0R::new(((self.bits >> 1) & 0x03ff) as u16)
    }
    #[doc = "Bits 17:26 - Address 1 Value"]
    #[inline(always)]
    pub fn addr1(&self) -> Addr1R {
        Addr1R::new(((self.bits >> 17) & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 1:10 - Address 0 Value"]
    #[inline(always)]
    pub fn addr0(&mut self) -> Addr0W<SamrSpec> {
        Addr0W::new(self, 1)
    }
    #[doc = "Bits 17:26 - Address 1 Value"]
    #[inline(always)]
    pub fn addr1(&mut self) -> Addr1W<SamrSpec> {
        Addr1W::new(self, 17)
    }
}
#[doc = "Target Address Match\n\nYou can [`read`](crate::Reg::read) this register and get [`samr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`samr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SamrSpec;
impl crate::RegisterSpec for SamrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`samr::R`](R) reader structure"]
impl crate::Readable for SamrSpec {}
#[doc = "`write(|w| ..)` method takes [`samr::W`](W) writer structure"]
impl crate::Writable for SamrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SAMR to value 0"]
impl crate::Resettable for SamrSpec {}
