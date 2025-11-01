#[doc = "Register `SWDATAB` writer"]
pub type W = crate::W<SwdatabSpec>;
#[doc = "Field `DATA` writer - Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "End\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    #[doc = "0: Not the end"]
    NotEnd = 0,
    #[doc = "1: End"]
    End = 1,
}
impl From<End> for bool {
    #[inline(always)]
    fn from(variant: End) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `END` writer - End"]
pub type EndW<'a, REG> = crate::BitWriter<'a, REG, End>;
impl<'a, REG> EndW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not the end"]
    #[inline(always)]
    pub fn not_end(self) -> &'a mut crate::W<REG> {
        self.variant(End::NotEnd)
    }
    #[doc = "End"]
    #[inline(always)]
    pub fn end(self) -> &'a mut crate::W<REG> {
        self.variant(End::End)
    }
}
#[doc = "End Also\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndAlso {
    #[doc = "0: Not the end"]
    NotEnd = 0,
    #[doc = "1: End"]
    End = 1,
}
impl From<EndAlso> for bool {
    #[inline(always)]
    fn from(variant: EndAlso) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `END_ALSO` writer - End Also"]
pub type EndAlsoW<'a, REG> = crate::BitWriter<'a, REG, EndAlso>;
impl<'a, REG> EndAlsoW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not the end"]
    #[inline(always)]
    pub fn not_end(self) -> &'a mut crate::W<REG> {
        self.variant(EndAlso::NotEnd)
    }
    #[doc = "End"]
    #[inline(always)]
    pub fn end(self) -> &'a mut crate::W<REG> {
        self.variant(EndAlso::End)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<SwdatabSpec> {
        DataW::new(self, 0)
    }
    #[doc = "Bit 8 - End"]
    #[inline(always)]
    pub fn end(&mut self) -> EndW<SwdatabSpec> {
        EndW::new(self, 8)
    }
    #[doc = "Bit 16 - End Also"]
    #[inline(always)]
    pub fn end_also(&mut self) -> EndAlsoW<SwdatabSpec> {
        EndAlsoW::new(self, 16)
    }
}
#[doc = "Target Write Data Byte\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatab::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwdatabSpec;
impl crate::RegisterSpec for SwdatabSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`swdatab::W`](W) writer structure"]
impl crate::Writable for SwdatabSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWDATAB to value 0"]
impl crate::Resettable for SwdatabSpec {}
