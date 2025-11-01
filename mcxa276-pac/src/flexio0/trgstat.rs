#[doc = "Register `TRGSTAT` reader"]
pub type R = crate::R<TrgstatSpec>;
#[doc = "Register `TRGSTAT` writer"]
pub type W = crate::W<TrgstatSpec>;
#[doc = "External Trigger Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Etsf {
    #[doc = "0: Clear"]
    Clr = 0,
    #[doc = "1: Set"]
    Set = 1,
}
impl From<Etsf> for u8 {
    #[inline(always)]
    fn from(variant: Etsf) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Etsf {
    type Ux = u8;
}
impl crate::IsEnum for Etsf {}
#[doc = "Field `ETSF` reader - External Trigger Status Flag"]
pub type EtsfR = crate::FieldReader<Etsf>;
impl EtsfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Etsf> {
        match self.bits {
            0 => Some(Etsf::Clr),
            1 => Some(Etsf::Set),
            _ => None,
        }
    }
    #[doc = "Clear"]
    #[inline(always)]
    pub fn is_clr(&self) -> bool {
        *self == Etsf::Clr
    }
    #[doc = "Set"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Etsf::Set
    }
}
#[doc = "Field `ETSF` writer - External Trigger Status Flag"]
pub type EtsfW<'a, REG> = crate::FieldWriter<'a, REG, 4, Etsf>;
impl<'a, REG> EtsfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Clear"]
    #[inline(always)]
    pub fn clr(self) -> &'a mut crate::W<REG> {
        self.variant(Etsf::Clr)
    }
    #[doc = "Set"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Etsf::Set)
    }
}
impl R {
    #[doc = "Bits 0:3 - External Trigger Status Flag"]
    #[inline(always)]
    pub fn etsf(&self) -> EtsfR {
        EtsfR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - External Trigger Status Flag"]
    #[inline(always)]
    pub fn etsf(&mut self) -> EtsfW<TrgstatSpec> {
        EtsfW::new(self, 0)
    }
}
#[doc = "Trigger Status\n\nYou can [`read`](crate::Reg::read) this register and get [`trgstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trgstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TrgstatSpec;
impl crate::RegisterSpec for TrgstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`trgstat::R`](R) reader structure"]
impl crate::Readable for TrgstatSpec {}
#[doc = "`write(|w| ..)` method takes [`trgstat::W`](W) writer structure"]
impl crate::Writable for TrgstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0f;
}
#[doc = "`reset()` method sets TRGSTAT to value 0"]
impl crate::Resettable for TrgstatSpec {}
