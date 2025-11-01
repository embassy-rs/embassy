#[doc = "Register `SHIFTERR` reader"]
pub type R = crate::R<ShifterrSpec>;
#[doc = "Register `SHIFTERR` writer"]
pub type W = crate::W<ShifterrSpec>;
#[doc = "Shifter Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sef {
    #[doc = "0: Clear"]
    Clr = 0,
    #[doc = "1: Set"]
    Set = 1,
}
impl From<Sef> for u8 {
    #[inline(always)]
    fn from(variant: Sef) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sef {
    type Ux = u8;
}
impl crate::IsEnum for Sef {}
#[doc = "Field `SEF` reader - Shifter Error Flag"]
pub type SefR = crate::FieldReader<Sef>;
impl SefR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Sef> {
        match self.bits {
            0 => Some(Sef::Clr),
            1 => Some(Sef::Set),
            _ => None,
        }
    }
    #[doc = "Clear"]
    #[inline(always)]
    pub fn is_clr(&self) -> bool {
        *self == Sef::Clr
    }
    #[doc = "Set"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Sef::Set
    }
}
#[doc = "Field `SEF` writer - Shifter Error Flag"]
pub type SefW<'a, REG> = crate::FieldWriter<'a, REG, 4, Sef>;
impl<'a, REG> SefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Clear"]
    #[inline(always)]
    pub fn clr(self) -> &'a mut crate::W<REG> {
        self.variant(Sef::Clr)
    }
    #[doc = "Set"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Sef::Set)
    }
}
impl R {
    #[doc = "Bits 0:3 - Shifter Error Flag"]
    #[inline(always)]
    pub fn sef(&self) -> SefR {
        SefR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Shifter Error Flag"]
    #[inline(always)]
    pub fn sef(&mut self) -> SefW<ShifterrSpec> {
        SefW::new(self, 0)
    }
}
#[doc = "Shifter Error\n\nYou can [`read`](crate::Reg::read) this register and get [`shifterr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shifterr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShifterrSpec;
impl crate::RegisterSpec for ShifterrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shifterr::R`](R) reader structure"]
impl crate::Readable for ShifterrSpec {}
#[doc = "`write(|w| ..)` method takes [`shifterr::W`](W) writer structure"]
impl crate::Writable for ShifterrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0f;
}
#[doc = "`reset()` method sets SHIFTERR to value 0"]
impl crate::Resettable for ShifterrSpec {}
