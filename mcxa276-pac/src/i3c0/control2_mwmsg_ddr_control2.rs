#[doc = "Register `MWMSG_DDR_CONTROL2` writer"]
pub type W = crate::W<Control2MwmsgDdrControl2Spec>;
#[doc = "Field `LEN` writer - Length of Message"]
pub type LenW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
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
    #[doc = "Bits 0:9 - Length of Message"]
    #[inline(always)]
    pub fn len(&mut self) -> LenW<Control2MwmsgDdrControl2Spec> {
        LenW::new(self, 0)
    }
    #[doc = "Bit 14 - End of Message"]
    #[inline(always)]
    pub fn end(&mut self) -> EndW<Control2MwmsgDdrControl2Spec> {
        EndW::new(self, 14)
    }
}
#[doc = "Controller Write Message in DDR Mode Control 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`control2_mwmsg_ddr_control2::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Control2MwmsgDdrControl2Spec;
impl crate::RegisterSpec for Control2MwmsgDdrControl2Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`control2_mwmsg_ddr_control2::W`](W) writer structure"]
impl crate::Writable for Control2MwmsgDdrControl2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MWMSG_DDR_CONTROL2 to value 0"]
impl crate::Resettable for Control2MwmsgDdrControl2Spec {}
