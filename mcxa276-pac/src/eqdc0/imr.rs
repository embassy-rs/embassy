#[doc = "Register `IMR` reader"]
pub type R = crate::R<ImrSpec>;
#[doc = "Register `IMR` writer"]
pub type W = crate::W<ImrSpec>;
#[doc = "Field `HOME_ENABLE` reader - HOME_ENABLE"]
pub type HomeEnableR = crate::BitReader;
#[doc = "Field `INDEX_PRESET` reader - INDEX_PRESET"]
pub type IndexPresetR = crate::BitReader;
#[doc = "Field `PHB` reader - PHB"]
pub type PhbR = crate::BitReader;
#[doc = "Field `PHA` reader - PHA"]
pub type PhaR = crate::BitReader;
#[doc = "Field `FHOM_ENA` reader - filter operation on HOME/ENABLE input"]
pub type FhomEnaR = crate::BitReader;
#[doc = "Field `FHOM_ENA` writer - filter operation on HOME/ENABLE input"]
pub type FhomEnaW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FIND_PRE` reader - filter operation on INDEX/PRESET input"]
pub type FindPreR = crate::BitReader;
#[doc = "Field `FIND_PRE` writer - filter operation on INDEX/PRESET input"]
pub type FindPreW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FPHB` reader - filter operation on PHASEB input"]
pub type FphbR = crate::BitReader;
#[doc = "Field `FPHB` writer - filter operation on PHASEB input"]
pub type FphbW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FPHA` reader - filter operation on PHASEA input"]
pub type FphaR = crate::BitReader;
#[doc = "Field `FPHA` writer - filter operation on PHASEA input"]
pub type FphaW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Position Compare 0 Flag Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmpf0 {
    #[doc = "0: When the position counter is less than value of COMP0 register"]
    Cmpf00 = 0,
    #[doc = "1: When the position counter is greater or equal than value of COMP0 register"]
    Cmpf01 = 1,
}
impl From<Cmpf0> for bool {
    #[inline(always)]
    fn from(variant: Cmpf0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMPF0` reader - Position Compare 0 Flag Output"]
pub type Cmpf0R = crate::BitReader<Cmpf0>;
impl Cmpf0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmpf0 {
        match self.bits {
            false => Cmpf0::Cmpf00,
            true => Cmpf0::Cmpf01,
        }
    }
    #[doc = "When the position counter is less than value of COMP0 register"]
    #[inline(always)]
    pub fn is_cmpf00(&self) -> bool {
        *self == Cmpf0::Cmpf00
    }
    #[doc = "When the position counter is greater or equal than value of COMP0 register"]
    #[inline(always)]
    pub fn is_cmpf01(&self) -> bool {
        *self == Cmpf0::Cmpf01
    }
}
#[doc = "Position Compare1 Flag Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp1f {
    #[doc = "0: When the position counter is less than value of COMP1 register"]
    Cmp1f0 = 0,
    #[doc = "1: When the position counter is greater or equal than value of COMP1 register"]
    Cmp1f1 = 1,
}
impl From<Cmp1f> for bool {
    #[inline(always)]
    fn from(variant: Cmp1f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP1F` reader - Position Compare1 Flag Output"]
pub type Cmp1fR = crate::BitReader<Cmp1f>;
impl Cmp1fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp1f {
        match self.bits {
            false => Cmp1f::Cmp1f0,
            true => Cmp1f::Cmp1f1,
        }
    }
    #[doc = "When the position counter is less than value of COMP1 register"]
    #[inline(always)]
    pub fn is_cmp1f0(&self) -> bool {
        *self == Cmp1f::Cmp1f0
    }
    #[doc = "When the position counter is greater or equal than value of COMP1 register"]
    #[inline(always)]
    pub fn is_cmp1f1(&self) -> bool {
        *self == Cmp1f::Cmp1f1
    }
}
#[doc = "Position Compare2 Flag Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp2f {
    #[doc = "0: When the position counter is less than value of COMP2 register"]
    Cmp2f0 = 0,
    #[doc = "1: When the position counter is greater or equal than value of COMP2 register"]
    Cmp2f1 = 1,
}
impl From<Cmp2f> for bool {
    #[inline(always)]
    fn from(variant: Cmp2f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP2F` reader - Position Compare2 Flag Output"]
pub type Cmp2fR = crate::BitReader<Cmp2f>;
impl Cmp2fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp2f {
        match self.bits {
            false => Cmp2f::Cmp2f0,
            true => Cmp2f::Cmp2f1,
        }
    }
    #[doc = "When the position counter is less than value of COMP2 register"]
    #[inline(always)]
    pub fn is_cmp2f0(&self) -> bool {
        *self == Cmp2f::Cmp2f0
    }
    #[doc = "When the position counter is greater or equal than value of COMP2 register"]
    #[inline(always)]
    pub fn is_cmp2f1(&self) -> bool {
        *self == Cmp2f::Cmp2f1
    }
}
#[doc = "Position Compare3 Flag Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp3f {
    #[doc = "0: When the position counter value is less than value of COMP3 register"]
    Cmp3f0 = 0,
    #[doc = "1: When the position counter is greater or equal than value of COMP3 register"]
    Cmp3f1 = 1,
}
impl From<Cmp3f> for bool {
    #[inline(always)]
    fn from(variant: Cmp3f) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP3F` reader - Position Compare3 Flag Output"]
pub type Cmp3fR = crate::BitReader<Cmp3f>;
impl Cmp3fR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp3f {
        match self.bits {
            false => Cmp3f::Cmp3f0,
            true => Cmp3f::Cmp3f1,
        }
    }
    #[doc = "When the position counter value is less than value of COMP3 register"]
    #[inline(always)]
    pub fn is_cmp3f0(&self) -> bool {
        *self == Cmp3f::Cmp3f0
    }
    #[doc = "When the position counter is greater or equal than value of COMP3 register"]
    #[inline(always)]
    pub fn is_cmp3f1(&self) -> bool {
        *self == Cmp3f::Cmp3f1
    }
}
#[doc = "Field `DIRH` reader - Count Direction Flag Hold"]
pub type DirhR = crate::BitReader;
#[doc = "Count Direction Flag Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    #[doc = "0: Current count was in the down direction"]
    Dir0 = 0,
    #[doc = "1: Current count was in the up direction"]
    Dir1 = 1,
}
impl From<Dir> for bool {
    #[inline(always)]
    fn from(variant: Dir) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIR` reader - Count Direction Flag Output"]
pub type DirR = crate::BitReader<Dir>;
impl DirR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dir {
        match self.bits {
            false => Dir::Dir0,
            true => Dir::Dir1,
        }
    }
    #[doc = "Current count was in the down direction"]
    #[inline(always)]
    pub fn is_dir0(&self) -> bool {
        *self == Dir::Dir0
    }
    #[doc = "Current count was in the up direction"]
    #[inline(always)]
    pub fn is_dir1(&self) -> bool {
        *self == Dir::Dir1
    }
}
impl R {
    #[doc = "Bit 0 - HOME_ENABLE"]
    #[inline(always)]
    pub fn home_enable(&self) -> HomeEnableR {
        HomeEnableR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - INDEX_PRESET"]
    #[inline(always)]
    pub fn index_preset(&self) -> IndexPresetR {
        IndexPresetR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - PHB"]
    #[inline(always)]
    pub fn phb(&self) -> PhbR {
        PhbR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - PHA"]
    #[inline(always)]
    pub fn pha(&self) -> PhaR {
        PhaR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - filter operation on HOME/ENABLE input"]
    #[inline(always)]
    pub fn fhom_ena(&self) -> FhomEnaR {
        FhomEnaR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - filter operation on INDEX/PRESET input"]
    #[inline(always)]
    pub fn find_pre(&self) -> FindPreR {
        FindPreR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - filter operation on PHASEB input"]
    #[inline(always)]
    pub fn fphb(&self) -> FphbR {
        FphbR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - filter operation on PHASEA input"]
    #[inline(always)]
    pub fn fpha(&self) -> FphaR {
        FphaR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Position Compare 0 Flag Output"]
    #[inline(always)]
    pub fn cmpf0(&self) -> Cmpf0R {
        Cmpf0R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Position Compare1 Flag Output"]
    #[inline(always)]
    pub fn cmp1f(&self) -> Cmp1fR {
        Cmp1fR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Position Compare2 Flag Output"]
    #[inline(always)]
    pub fn cmp2f(&self) -> Cmp2fR {
        Cmp2fR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Position Compare3 Flag Output"]
    #[inline(always)]
    pub fn cmp3f(&self) -> Cmp3fR {
        Cmp3fR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 14 - Count Direction Flag Hold"]
    #[inline(always)]
    pub fn dirh(&self) -> DirhR {
        DirhR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Count Direction Flag Output"]
    #[inline(always)]
    pub fn dir(&self) -> DirR {
        DirR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 4 - filter operation on HOME/ENABLE input"]
    #[inline(always)]
    pub fn fhom_ena(&mut self) -> FhomEnaW<ImrSpec> {
        FhomEnaW::new(self, 4)
    }
    #[doc = "Bit 5 - filter operation on INDEX/PRESET input"]
    #[inline(always)]
    pub fn find_pre(&mut self) -> FindPreW<ImrSpec> {
        FindPreW::new(self, 5)
    }
    #[doc = "Bit 6 - filter operation on PHASEB input"]
    #[inline(always)]
    pub fn fphb(&mut self) -> FphbW<ImrSpec> {
        FphbW::new(self, 6)
    }
    #[doc = "Bit 7 - filter operation on PHASEA input"]
    #[inline(always)]
    pub fn fpha(&mut self) -> FphaW<ImrSpec> {
        FphaW::new(self, 7)
    }
}
#[doc = "Input Monitor Register\n\nYou can [`read`](crate::Reg::read) this register and get [`imr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`imr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ImrSpec;
impl crate::RegisterSpec for ImrSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`imr::R`](R) reader structure"]
impl crate::Readable for ImrSpec {}
#[doc = "`write(|w| ..)` method takes [`imr::W`](W) writer structure"]
impl crate::Writable for ImrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IMR to value 0"]
impl crate::Resettable for ImrSpec {}
