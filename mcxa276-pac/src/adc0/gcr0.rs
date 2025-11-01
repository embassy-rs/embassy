#[doc = "Register `GCR0` reader"]
pub type R = crate::R<Gcr0Spec>;
#[doc = "Register `GCR0` writer"]
pub type W = crate::W<Gcr0Spec>;
#[doc = "Field `GCALR` reader - Gain Calculation Result"]
pub type GcalrR = crate::FieldReader<u32>;
#[doc = "Field `GCALR` writer - Gain Calculation Result"]
pub type GcalrW<'a, REG> = crate::FieldWriter<'a, REG, 17, u32>;
#[doc = "Gain Calculation Ready\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdy {
    #[doc = "0: The GCALR value is invalid."]
    NotValid = 0,
    #[doc = "1: The GCALR value is valid."]
    Valid = 1,
}
impl From<Rdy> for bool {
    #[inline(always)]
    fn from(variant: Rdy) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDY` reader - Gain Calculation Ready"]
pub type RdyR = crate::BitReader<Rdy>;
impl RdyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdy {
        match self.bits {
            false => Rdy::NotValid,
            true => Rdy::Valid,
        }
    }
    #[doc = "The GCALR value is invalid."]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Rdy::NotValid
    }
    #[doc = "The GCALR value is valid."]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Rdy::Valid
    }
}
#[doc = "Field `RDY` writer - Gain Calculation Ready"]
pub type RdyW<'a, REG> = crate::BitWriter<'a, REG, Rdy>;
impl<'a, REG> RdyW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The GCALR value is invalid."]
    #[inline(always)]
    pub fn not_valid(self) -> &'a mut crate::W<REG> {
        self.variant(Rdy::NotValid)
    }
    #[doc = "The GCALR value is valid."]
    #[inline(always)]
    pub fn valid(self) -> &'a mut crate::W<REG> {
        self.variant(Rdy::Valid)
    }
}
impl R {
    #[doc = "Bits 0:16 - Gain Calculation Result"]
    #[inline(always)]
    pub fn gcalr(&self) -> GcalrR {
        GcalrR::new(self.bits & 0x0001_ffff)
    }
    #[doc = "Bit 24 - Gain Calculation Ready"]
    #[inline(always)]
    pub fn rdy(&self) -> RdyR {
        RdyR::new(((self.bits >> 24) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:16 - Gain Calculation Result"]
    #[inline(always)]
    pub fn gcalr(&mut self) -> GcalrW<Gcr0Spec> {
        GcalrW::new(self, 0)
    }
    #[doc = "Bit 24 - Gain Calculation Ready"]
    #[inline(always)]
    pub fn rdy(&mut self) -> RdyW<Gcr0Spec> {
        RdyW::new(self, 24)
    }
}
#[doc = "Gain Calculation Result\n\nYou can [`read`](crate::Reg::read) this register and get [`gcr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gcr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Gcr0Spec;
impl crate::RegisterSpec for Gcr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gcr0::R`](R) reader structure"]
impl crate::Readable for Gcr0Spec {}
#[doc = "`write(|w| ..)` method takes [`gcr0::W`](W) writer structure"]
impl crate::Writable for Gcr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GCR0 to value 0x0001_0000"]
impl crate::Resettable for Gcr0Spec {
    const RESET_VALUE: u32 = 0x0001_0000;
}
