#[doc = "Register `SWDATAH` writer"]
pub type W = crate::W<SwdatahSpec>;
#[doc = "Field `DATA0` writer - Data 0"]
pub type Data0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA1` writer - Data 1"]
pub type Data1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "End of Message\n\nValue on reset: 0"]
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
#[doc = "Field `END` writer - End of Message"]
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
impl W {
    #[doc = "Bits 0:7 - Data 0"]
    #[inline(always)]
    pub fn data0(&mut self) -> Data0W<SwdatahSpec> {
        Data0W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data 1"]
    #[inline(always)]
    pub fn data1(&mut self) -> Data1W<SwdatahSpec> {
        Data1W::new(self, 8)
    }
    #[doc = "Bit 16 - End of Message"]
    #[inline(always)]
    pub fn end(&mut self) -> EndW<SwdatahSpec> {
        EndW::new(self, 16)
    }
}
#[doc = "Target Write Data Halfword\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatah::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwdatahSpec;
impl crate::RegisterSpec for SwdatahSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`swdatah::W`](W) writer structure"]
impl crate::Writable for SwdatahSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWDATAH to value 0"]
impl crate::Resettable for SwdatahSpec {}
