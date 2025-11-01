#[doc = "Register `EIMCR` reader"]
pub type R = crate::R<EimcrSpec>;
#[doc = "Register `EIMCR` writer"]
pub type W = crate::W<EimcrSpec>;
#[doc = "Global Error Injection Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geien {
    #[doc = "0: Disabled"]
    Disable = 0,
    #[doc = "1: Enabled"]
    Enable = 1,
}
impl From<Geien> for bool {
    #[inline(always)]
    fn from(variant: Geien) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GEIEN` reader - Global Error Injection Enable"]
pub type GeienR = crate::BitReader<Geien>;
impl GeienR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Geien {
        match self.bits {
            false => Geien::Disable,
            true => Geien::Enable,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Geien::Disable
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Geien::Enable
    }
}
#[doc = "Field `GEIEN` writer - Global Error Injection Enable"]
pub type GeienW<'a, REG> = crate::BitWriter<'a, REG, Geien>;
impl<'a, REG> GeienW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Geien::Disable)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Geien::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Global Error Injection Enable"]
    #[inline(always)]
    pub fn geien(&self) -> GeienR {
        GeienR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Global Error Injection Enable"]
    #[inline(always)]
    pub fn geien(&mut self) -> GeienW<EimcrSpec> {
        GeienW::new(self, 0)
    }
}
#[doc = "Error Injection Module Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`eimcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eimcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EimcrSpec;
impl crate::RegisterSpec for EimcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`eimcr::R`](R) reader structure"]
impl crate::Readable for EimcrSpec {}
#[doc = "`write(|w| ..)` method takes [`eimcr::W`](W) writer structure"]
impl crate::Writable for EimcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EIMCR to value 0"]
impl crate::Resettable for EimcrSpec {}
