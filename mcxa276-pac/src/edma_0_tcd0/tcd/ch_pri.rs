#[doc = "Register `CH_PRI` reader"]
pub type R = crate::R<ChPriSpec>;
#[doc = "Register `CH_PRI` writer"]
pub type W = crate::W<ChPriSpec>;
#[doc = "Field `APL` reader - Arbitration Priority Level"]
pub type AplR = crate::FieldReader;
#[doc = "Field `APL` writer - Arbitration Priority Level"]
pub type AplW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Disable Preempt Ability\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dpa {
    #[doc = "0: Channel can suspend a lower-priority channel"]
    Suspend = 0,
    #[doc = "1: Channel cannot suspend any other channel, regardless of channel priority"]
    CannotSuspend = 1,
}
impl From<Dpa> for bool {
    #[inline(always)]
    fn from(variant: Dpa) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DPA` reader - Disable Preempt Ability"]
pub type DpaR = crate::BitReader<Dpa>;
impl DpaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dpa {
        match self.bits {
            false => Dpa::Suspend,
            true => Dpa::CannotSuspend,
        }
    }
    #[doc = "Channel can suspend a lower-priority channel"]
    #[inline(always)]
    pub fn is_suspend(&self) -> bool {
        *self == Dpa::Suspend
    }
    #[doc = "Channel cannot suspend any other channel, regardless of channel priority"]
    #[inline(always)]
    pub fn is_cannot_suspend(&self) -> bool {
        *self == Dpa::CannotSuspend
    }
}
#[doc = "Field `DPA` writer - Disable Preempt Ability"]
pub type DpaW<'a, REG> = crate::BitWriter<'a, REG, Dpa>;
impl<'a, REG> DpaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Channel can suspend a lower-priority channel"]
    #[inline(always)]
    pub fn suspend(self) -> &'a mut crate::W<REG> {
        self.variant(Dpa::Suspend)
    }
    #[doc = "Channel cannot suspend any other channel, regardless of channel priority"]
    #[inline(always)]
    pub fn cannot_suspend(self) -> &'a mut crate::W<REG> {
        self.variant(Dpa::CannotSuspend)
    }
}
#[doc = "Enable Channel Preemption\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ecp {
    #[doc = "0: Channel cannot be suspended by a higher-priority channel's service request"]
    CannotSuspend = 0,
    #[doc = "1: Channel can be temporarily suspended by a higher-priority channel's service request"]
    Suspend = 1,
}
impl From<Ecp> for bool {
    #[inline(always)]
    fn from(variant: Ecp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ECP` reader - Enable Channel Preemption"]
pub type EcpR = crate::BitReader<Ecp>;
impl EcpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ecp {
        match self.bits {
            false => Ecp::CannotSuspend,
            true => Ecp::Suspend,
        }
    }
    #[doc = "Channel cannot be suspended by a higher-priority channel's service request"]
    #[inline(always)]
    pub fn is_cannot_suspend(&self) -> bool {
        *self == Ecp::CannotSuspend
    }
    #[doc = "Channel can be temporarily suspended by a higher-priority channel's service request"]
    #[inline(always)]
    pub fn is_suspend(&self) -> bool {
        *self == Ecp::Suspend
    }
}
#[doc = "Field `ECP` writer - Enable Channel Preemption"]
pub type EcpW<'a, REG> = crate::BitWriter<'a, REG, Ecp>;
impl<'a, REG> EcpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Channel cannot be suspended by a higher-priority channel's service request"]
    #[inline(always)]
    pub fn cannot_suspend(self) -> &'a mut crate::W<REG> {
        self.variant(Ecp::CannotSuspend)
    }
    #[doc = "Channel can be temporarily suspended by a higher-priority channel's service request"]
    #[inline(always)]
    pub fn suspend(self) -> &'a mut crate::W<REG> {
        self.variant(Ecp::Suspend)
    }
}
impl R {
    #[doc = "Bits 0:2 - Arbitration Priority Level"]
    #[inline(always)]
    pub fn apl(&self) -> AplR {
        AplR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 30 - Disable Preempt Ability"]
    #[inline(always)]
    pub fn dpa(&self) -> DpaR {
        DpaR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Enable Channel Preemption"]
    #[inline(always)]
    pub fn ecp(&self) -> EcpR {
        EcpR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:2 - Arbitration Priority Level"]
    #[inline(always)]
    pub fn apl(&mut self) -> AplW<ChPriSpec> {
        AplW::new(self, 0)
    }
    #[doc = "Bit 30 - Disable Preempt Ability"]
    #[inline(always)]
    pub fn dpa(&mut self) -> DpaW<ChPriSpec> {
        DpaW::new(self, 30)
    }
    #[doc = "Bit 31 - Enable Channel Preemption"]
    #[inline(always)]
    pub fn ecp(&mut self) -> EcpW<ChPriSpec> {
        EcpW::new(self, 31)
    }
}
#[doc = "Channel Priority\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_pri::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_pri::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChPriSpec;
impl crate::RegisterSpec for ChPriSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ch_pri::R`](R) reader structure"]
impl crate::Readable for ChPriSpec {}
#[doc = "`write(|w| ..)` method takes [`ch_pri::W`](W) writer structure"]
impl crate::Writable for ChPriSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CH_PRI to value 0"]
impl crate::Resettable for ChPriSpec {}
