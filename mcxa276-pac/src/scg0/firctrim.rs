#[doc = "Register `FIRCTRIM` reader"]
pub type R = crate::R<FirctrimSpec>;
#[doc = "Register `FIRCTRIM` writer"]
pub type W = crate::W<FirctrimSpec>;
#[doc = "Field `TRIMFINE` reader - Trim Fine"]
pub type TrimfineR = crate::FieldReader;
#[doc = "Field `TRIMFINE` writer - Trim Fine"]
pub type TrimfineW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `TRIMCOAR` reader - Trim Coarse"]
pub type TrimcoarR = crate::FieldReader;
#[doc = "Field `TRIMCOAR` writer - Trim Coarse"]
pub type TrimcoarW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `TRIMTEMP` reader - Trim Temperature"]
pub type TrimtempR = crate::FieldReader;
#[doc = "Field `TRIMTEMP` writer - Trim Temperature"]
pub type TrimtempW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `TRIMSTART` reader - Trim Start"]
pub type TrimstartR = crate::FieldReader;
#[doc = "Field `TRIMSTART` writer - Trim Start"]
pub type TrimstartW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
impl R {
    #[doc = "Bits 0:7 - Trim Fine"]
    #[inline(always)]
    pub fn trimfine(&self) -> TrimfineR {
        TrimfineR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:13 - Trim Coarse"]
    #[inline(always)]
    pub fn trimcoar(&self) -> TrimcoarR {
        TrimcoarR::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bits 16:19 - Trim Temperature"]
    #[inline(always)]
    pub fn trimtemp(&self) -> TrimtempR {
        TrimtempR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 24:29 - Trim Start"]
    #[inline(always)]
    pub fn trimstart(&self) -> TrimstartR {
        TrimstartR::new(((self.bits >> 24) & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Trim Fine"]
    #[inline(always)]
    pub fn trimfine(&mut self) -> TrimfineW<FirctrimSpec> {
        TrimfineW::new(self, 0)
    }
    #[doc = "Bits 8:13 - Trim Coarse"]
    #[inline(always)]
    pub fn trimcoar(&mut self) -> TrimcoarW<FirctrimSpec> {
        TrimcoarW::new(self, 8)
    }
    #[doc = "Bits 16:19 - Trim Temperature"]
    #[inline(always)]
    pub fn trimtemp(&mut self) -> TrimtempW<FirctrimSpec> {
        TrimtempW::new(self, 16)
    }
    #[doc = "Bits 24:29 - Trim Start"]
    #[inline(always)]
    pub fn trimstart(&mut self) -> TrimstartW<FirctrimSpec> {
        TrimstartW::new(self, 24)
    }
}
#[doc = "FIRC Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`firctrim::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`firctrim::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FirctrimSpec;
impl crate::RegisterSpec for FirctrimSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`firctrim::R`](R) reader structure"]
impl crate::Readable for FirctrimSpec {}
#[doc = "`write(|w| ..)` method takes [`firctrim::W`](W) writer structure"]
impl crate::Writable for FirctrimSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FIRCTRIM to value 0"]
impl crate::Resettable for FirctrimSpec {}
