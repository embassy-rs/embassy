#[doc = "Register `OTGISTAT` reader"]
pub type R = crate::R<OtgistatSpec>;
#[doc = "Register `OTGISTAT` writer"]
pub type W = crate::W<OtgistatSpec>;
#[doc = "Line State Change Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineStateChg {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Interrupt occurred"]
    IntYes = 1,
}
impl From<LineStateChg> for bool {
    #[inline(always)]
    fn from(variant: LineStateChg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LINE_STATE_CHG` reader - Line State Change Interrupt Flag"]
pub type LineStateChgR = crate::BitReader<LineStateChg>;
impl LineStateChgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LineStateChg {
        match self.bits {
            false => LineStateChg::IntNo,
            true => LineStateChg::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == LineStateChg::IntNo
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == LineStateChg::IntYes
    }
}
#[doc = "Field `LINE_STATE_CHG` writer - Line State Change Interrupt Flag"]
pub type LineStateChgW<'a, REG> = crate::BitWriter1C<'a, REG, LineStateChg>;
impl<'a, REG> LineStateChgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(LineStateChg::IntNo)
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(LineStateChg::IntYes)
    }
}
#[doc = "One Millisecond Timer Timeout Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Onemsec {
    #[doc = "0: Not timed out"]
    IntNo = 0,
    #[doc = "1: Timed out"]
    IntYes = 1,
}
impl From<Onemsec> for bool {
    #[inline(always)]
    fn from(variant: Onemsec) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ONEMSEC` reader - One Millisecond Timer Timeout Flag"]
pub type OnemsecR = crate::BitReader<Onemsec>;
impl OnemsecR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Onemsec {
        match self.bits {
            false => Onemsec::IntNo,
            true => Onemsec::IntYes,
        }
    }
    #[doc = "Not timed out"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Onemsec::IntNo
    }
    #[doc = "Timed out"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Onemsec::IntYes
    }
}
#[doc = "Field `ONEMSEC` writer - One Millisecond Timer Timeout Flag"]
pub type OnemsecW<'a, REG> = crate::BitWriter1C<'a, REG, Onemsec>;
impl<'a, REG> OnemsecW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not timed out"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Onemsec::IntNo)
    }
    #[doc = "Timed out"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Onemsec::IntYes)
    }
}
impl R {
    #[doc = "Bit 5 - Line State Change Interrupt Flag"]
    #[inline(always)]
    pub fn line_state_chg(&self) -> LineStateChgR {
        LineStateChgR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - One Millisecond Timer Timeout Flag"]
    #[inline(always)]
    pub fn onemsec(&self) -> OnemsecR {
        OnemsecR::new(((self.bits >> 6) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 5 - Line State Change Interrupt Flag"]
    #[inline(always)]
    pub fn line_state_chg(&mut self) -> LineStateChgW<OtgistatSpec> {
        LineStateChgW::new(self, 5)
    }
    #[doc = "Bit 6 - One Millisecond Timer Timeout Flag"]
    #[inline(always)]
    pub fn onemsec(&mut self) -> OnemsecW<OtgistatSpec> {
        OnemsecW::new(self, 6)
    }
}
#[doc = "OTG Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`otgistat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`otgistat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OtgistatSpec;
impl crate::RegisterSpec for OtgistatSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`otgistat::R`](R) reader structure"]
impl crate::Readable for OtgistatSpec {}
#[doc = "`write(|w| ..)` method takes [`otgistat::W`](W) writer structure"]
impl crate::Writable for OtgistatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u8 = 0x60;
}
#[doc = "`reset()` method sets OTGISTAT to value 0"]
impl crate::Resettable for OtgistatSpec {}
