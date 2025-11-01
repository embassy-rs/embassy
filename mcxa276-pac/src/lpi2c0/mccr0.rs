#[doc = "Register `MCCR0` reader"]
pub type R = crate::R<Mccr0Spec>;
#[doc = "Register `MCCR0` writer"]
pub type W = crate::W<Mccr0Spec>;
#[doc = "Field `CLKLO` reader - Clock Low Period"]
pub type ClkloR = crate::FieldReader;
#[doc = "Field `CLKLO` writer - Clock Low Period"]
pub type ClkloW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `CLKHI` reader - Clock High Period"]
pub type ClkhiR = crate::FieldReader;
#[doc = "Field `CLKHI` writer - Clock High Period"]
pub type ClkhiW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `SETHOLD` reader - Setup Hold Delay"]
pub type SetholdR = crate::FieldReader;
#[doc = "Field `SETHOLD` writer - Setup Hold Delay"]
pub type SetholdW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `DATAVD` reader - Data Valid Delay"]
pub type DatavdR = crate::FieldReader;
#[doc = "Field `DATAVD` writer - Data Valid Delay"]
pub type DatavdW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
impl R {
    #[doc = "Bits 0:5 - Clock Low Period"]
    #[inline(always)]
    pub fn clklo(&self) -> ClkloR {
        ClkloR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 8:13 - Clock High Period"]
    #[inline(always)]
    pub fn clkhi(&self) -> ClkhiR {
        ClkhiR::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bits 16:21 - Setup Hold Delay"]
    #[inline(always)]
    pub fn sethold(&self) -> SetholdR {
        SetholdR::new(((self.bits >> 16) & 0x3f) as u8)
    }
    #[doc = "Bits 24:29 - Data Valid Delay"]
    #[inline(always)]
    pub fn datavd(&self) -> DatavdR {
        DatavdR::new(((self.bits >> 24) & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - Clock Low Period"]
    #[inline(always)]
    pub fn clklo(&mut self) -> ClkloW<Mccr0Spec> {
        ClkloW::new(self, 0)
    }
    #[doc = "Bits 8:13 - Clock High Period"]
    #[inline(always)]
    pub fn clkhi(&mut self) -> ClkhiW<Mccr0Spec> {
        ClkhiW::new(self, 8)
    }
    #[doc = "Bits 16:21 - Setup Hold Delay"]
    #[inline(always)]
    pub fn sethold(&mut self) -> SetholdW<Mccr0Spec> {
        SetholdW::new(self, 16)
    }
    #[doc = "Bits 24:29 - Data Valid Delay"]
    #[inline(always)]
    pub fn datavd(&mut self) -> DatavdW<Mccr0Spec> {
        DatavdW::new(self, 24)
    }
}
#[doc = "Controller Clock Configuration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`mccr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mccr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mccr0Spec;
impl crate::RegisterSpec for Mccr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mccr0::R`](R) reader structure"]
impl crate::Readable for Mccr0Spec {}
#[doc = "`write(|w| ..)` method takes [`mccr0::W`](W) writer structure"]
impl crate::Writable for Mccr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCCR0 to value 0"]
impl crate::Resettable for Mccr0Spec {}
