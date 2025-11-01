#[doc = "Register `PINSTAT` reader"]
pub type R = crate::R<PinstatSpec>;
#[doc = "Register `PINSTAT` writer"]
pub type W = crate::W<PinstatSpec>;
#[doc = "Pin Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Psf {
    #[doc = "0: Clear"]
    Clr = 0,
    #[doc = "1: Set"]
    Set = 1,
}
impl From<Psf> for u32 {
    #[inline(always)]
    fn from(variant: Psf) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Psf {
    type Ux = u32;
}
impl crate::IsEnum for Psf {}
#[doc = "Field `PSF` reader - Pin Status Flag"]
pub type PsfR = crate::FieldReader<Psf>;
impl PsfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Psf> {
        match self.bits {
            0 => Some(Psf::Clr),
            1 => Some(Psf::Set),
            _ => None,
        }
    }
    #[doc = "Clear"]
    #[inline(always)]
    pub fn is_clr(&self) -> bool {
        *self == Psf::Clr
    }
    #[doc = "Set"]
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        *self == Psf::Set
    }
}
#[doc = "Field `PSF` writer - Pin Status Flag"]
pub type PsfW<'a, REG> = crate::FieldWriter<'a, REG, 32, Psf>;
impl<'a, REG> PsfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u32>,
{
    #[doc = "Clear"]
    #[inline(always)]
    pub fn clr(self) -> &'a mut crate::W<REG> {
        self.variant(Psf::Clr)
    }
    #[doc = "Set"]
    #[inline(always)]
    pub fn set_(self) -> &'a mut crate::W<REG> {
        self.variant(Psf::Set)
    }
}
impl R {
    #[doc = "Bits 0:31 - Pin Status Flag"]
    #[inline(always)]
    pub fn psf(&self) -> PsfR {
        PsfR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Pin Status Flag"]
    #[inline(always)]
    pub fn psf(&mut self) -> PsfW<PinstatSpec> {
        PsfW::new(self, 0)
    }
}
#[doc = "Pin Status\n\nYou can [`read`](crate::Reg::read) this register and get [`pinstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinstatSpec;
impl crate::RegisterSpec for PinstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinstat::R`](R) reader structure"]
impl crate::Readable for PinstatSpec {}
#[doc = "`write(|w| ..)` method takes [`pinstat::W`](W) writer structure"]
impl crate::Writable for PinstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xffff_ffff;
}
#[doc = "`reset()` method sets PINSTAT to value 0"]
impl crate::Resettable for PinstatSpec {}
