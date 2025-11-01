#[doc = "Register `EICHEN` reader"]
pub type R = crate::R<EichenSpec>;
#[doc = "Register `EICHEN` writer"]
pub type W = crate::W<EichenSpec>;
#[doc = "Error Injection Channel 0 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eich0en {
    #[doc = "0: Error injection is disabled on Error Injection Channel 0"]
    Disable = 0,
    #[doc = "1: Error injection is enabled on Error Injection Channel 0"]
    Enable = 1,
}
impl From<Eich0en> for bool {
    #[inline(always)]
    fn from(variant: Eich0en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EICH0EN` reader - Error Injection Channel 0 Enable"]
pub type Eich0enR = crate::BitReader<Eich0en>;
impl Eich0enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Eich0en {
        match self.bits {
            false => Eich0en::Disable,
            true => Eich0en::Enable,
        }
    }
    #[doc = "Error injection is disabled on Error Injection Channel 0"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Eich0en::Disable
    }
    #[doc = "Error injection is enabled on Error Injection Channel 0"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Eich0en::Enable
    }
}
#[doc = "Field `EICH0EN` writer - Error Injection Channel 0 Enable"]
pub type Eich0enW<'a, REG> = crate::BitWriter<'a, REG, Eich0en>;
impl<'a, REG> Eich0enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error injection is disabled on Error Injection Channel 0"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Eich0en::Disable)
    }
    #[doc = "Error injection is enabled on Error Injection Channel 0"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Eich0en::Enable)
    }
}
impl R {
    #[doc = "Bit 31 - Error Injection Channel 0 Enable"]
    #[inline(always)]
    pub fn eich0en(&self) -> Eich0enR {
        Eich0enR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 31 - Error Injection Channel 0 Enable"]
    #[inline(always)]
    pub fn eich0en(&mut self) -> Eich0enW<EichenSpec> {
        Eich0enW::new(self, 31)
    }
}
#[doc = "Error Injection Channel Enable register\n\nYou can [`read`](crate::Reg::read) this register and get [`eichen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eichen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EichenSpec;
impl crate::RegisterSpec for EichenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`eichen::R`](R) reader structure"]
impl crate::Readable for EichenSpec {}
#[doc = "`write(|w| ..)` method takes [`eichen::W`](W) writer structure"]
impl crate::Writable for EichenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EICHEN to value 0"]
impl crate::Resettable for EichenSpec {}
