#[doc = "Register `PCR1` reader"]
pub type R = crate::R<Pcr1Spec>;
#[doc = "Register `PCR1` writer"]
pub type W = crate::W<Pcr1Spec>;
#[doc = "Pull Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ps {
    #[doc = "0: Enables internal pulldown resistor"]
    Ps0 = 0,
    #[doc = "1: Enables internal pullup resistor"]
    Ps1 = 1,
}
impl From<Ps> for bool {
    #[inline(always)]
    fn from(variant: Ps) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PS` reader - Pull Select"]
pub type PsR = crate::BitReader<Ps>;
impl PsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ps {
        match self.bits {
            false => Ps::Ps0,
            true => Ps::Ps1,
        }
    }
    #[doc = "Enables internal pulldown resistor"]
    #[inline(always)]
    pub fn is_ps0(&self) -> bool {
        *self == Ps::Ps0
    }
    #[doc = "Enables internal pullup resistor"]
    #[inline(always)]
    pub fn is_ps1(&self) -> bool {
        *self == Ps::Ps1
    }
}
#[doc = "Field `PS` writer - Pull Select"]
pub type PsW<'a, REG> = crate::BitWriter<'a, REG, Ps>;
impl<'a, REG> PsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enables internal pulldown resistor"]
    #[inline(always)]
    pub fn ps0(self) -> &'a mut crate::W<REG> {
        self.variant(Ps::Ps0)
    }
    #[doc = "Enables internal pullup resistor"]
    #[inline(always)]
    pub fn ps1(self) -> &'a mut crate::W<REG> {
        self.variant(Ps::Ps1)
    }
}
#[doc = "Pull Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pe {
    #[doc = "0: Disables"]
    Pe0 = 0,
    #[doc = "1: Enables"]
    Pe1 = 1,
}
impl From<Pe> for bool {
    #[inline(always)]
    fn from(variant: Pe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PE` reader - Pull Enable"]
pub type PeR = crate::BitReader<Pe>;
impl PeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pe {
        match self.bits {
            false => Pe::Pe0,
            true => Pe::Pe1,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_pe0(&self) -> bool {
        *self == Pe::Pe0
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_pe1(&self) -> bool {
        *self == Pe::Pe1
    }
}
#[doc = "Field `PE` writer - Pull Enable"]
pub type PeW<'a, REG> = crate::BitWriter<'a, REG, Pe>;
impl<'a, REG> PeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn pe0(self) -> &'a mut crate::W<REG> {
        self.variant(Pe::Pe0)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn pe1(self) -> &'a mut crate::W<REG> {
        self.variant(Pe::Pe1)
    }
}
#[doc = "Slew Rate Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sre {
    #[doc = "0: Fast"]
    Sre0 = 0,
    #[doc = "1: Slow"]
    Sre1 = 1,
}
impl From<Sre> for bool {
    #[inline(always)]
    fn from(variant: Sre) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SRE` reader - Slew Rate Enable"]
pub type SreR = crate::BitReader<Sre>;
impl SreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sre {
        match self.bits {
            false => Sre::Sre0,
            true => Sre::Sre1,
        }
    }
    #[doc = "Fast"]
    #[inline(always)]
    pub fn is_sre0(&self) -> bool {
        *self == Sre::Sre0
    }
    #[doc = "Slow"]
    #[inline(always)]
    pub fn is_sre1(&self) -> bool {
        *self == Sre::Sre1
    }
}
#[doc = "Field `SRE` writer - Slew Rate Enable"]
pub type SreW<'a, REG> = crate::BitWriter<'a, REG, Sre>;
impl<'a, REG> SreW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Fast"]
    #[inline(always)]
    pub fn sre0(self) -> &'a mut crate::W<REG> {
        self.variant(Sre::Sre0)
    }
    #[doc = "Slow"]
    #[inline(always)]
    pub fn sre1(self) -> &'a mut crate::W<REG> {
        self.variant(Sre::Sre1)
    }
}
#[doc = "Open Drain Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ode {
    #[doc = "0: Disables"]
    Ode0 = 0,
    #[doc = "1: Enables"]
    Ode1 = 1,
}
impl From<Ode> for bool {
    #[inline(always)]
    fn from(variant: Ode) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ODE` reader - Open Drain Enable"]
pub type OdeR = crate::BitReader<Ode>;
impl OdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ode {
        match self.bits {
            false => Ode::Ode0,
            true => Ode::Ode1,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_ode0(&self) -> bool {
        *self == Ode::Ode0
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_ode1(&self) -> bool {
        *self == Ode::Ode1
    }
}
#[doc = "Field `ODE` writer - Open Drain Enable"]
pub type OdeW<'a, REG> = crate::BitWriter<'a, REG, Ode>;
impl<'a, REG> OdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn ode0(self) -> &'a mut crate::W<REG> {
        self.variant(Ode::Ode0)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn ode1(self) -> &'a mut crate::W<REG> {
        self.variant(Ode::Ode1)
    }
}
#[doc = "Drive Strength Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dse {
    #[doc = "0: Low"]
    Dse0 = 0,
    #[doc = "1: High"]
    Dse1 = 1,
}
impl From<Dse> for bool {
    #[inline(always)]
    fn from(variant: Dse) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DSE` reader - Drive Strength Enable"]
pub type DseR = crate::BitReader<Dse>;
impl DseR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dse {
        match self.bits {
            false => Dse::Dse0,
            true => Dse::Dse1,
        }
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_dse0(&self) -> bool {
        *self == Dse::Dse0
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn is_dse1(&self) -> bool {
        *self == Dse::Dse1
    }
}
#[doc = "Field `DSE` writer - Drive Strength Enable"]
pub type DseW<'a, REG> = crate::BitWriter<'a, REG, Dse>;
impl<'a, REG> DseW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low"]
    #[inline(always)]
    pub fn dse0(self) -> &'a mut crate::W<REG> {
        self.variant(Dse::Dse0)
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn dse1(self) -> &'a mut crate::W<REG> {
        self.variant(Dse::Dse1)
    }
}
#[doc = "Pin Multiplex Control\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mux {
    #[doc = "0: Alternative 0 (GPIO)"]
    Mux00 = 0,
    #[doc = "1: Alternative 1 (chip-specific)"]
    Mux01 = 1,
    #[doc = "2: Alternative 2 (chip-specific)"]
    Mux10 = 2,
    #[doc = "3: Alternative 3 (chip-specific)"]
    Mux11 = 3,
    #[doc = "4: Alternative 4 (chip-specific)"]
    Mux100 = 4,
    #[doc = "5: Alternative 5 (chip-specific)"]
    Mux101 = 5,
    #[doc = "6: Alternative 6 (chip-specific)"]
    Mux110 = 6,
    #[doc = "7: Alternative 7 (chip-specific)"]
    Mux111 = 7,
}
impl From<Mux> for u8 {
    #[inline(always)]
    fn from(variant: Mux) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mux {
    type Ux = u8;
}
impl crate::IsEnum for Mux {}
#[doc = "Field `MUX` reader - Pin Multiplex Control"]
pub type MuxR = crate::FieldReader<Mux>;
impl MuxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mux {
        match self.bits {
            0 => Mux::Mux00,
            1 => Mux::Mux01,
            2 => Mux::Mux10,
            3 => Mux::Mux11,
            4 => Mux::Mux100,
            5 => Mux::Mux101,
            6 => Mux::Mux110,
            7 => Mux::Mux111,
            _ => unreachable!(),
        }
    }
    #[doc = "Alternative 0 (GPIO)"]
    #[inline(always)]
    pub fn is_mux00(&self) -> bool {
        *self == Mux::Mux00
    }
    #[doc = "Alternative 1 (chip-specific)"]
    #[inline(always)]
    pub fn is_mux01(&self) -> bool {
        *self == Mux::Mux01
    }
    #[doc = "Alternative 2 (chip-specific)"]
    #[inline(always)]
    pub fn is_mux10(&self) -> bool {
        *self == Mux::Mux10
    }
    #[doc = "Alternative 3 (chip-specific)"]
    #[inline(always)]
    pub fn is_mux11(&self) -> bool {
        *self == Mux::Mux11
    }
    #[doc = "Alternative 4 (chip-specific)"]
    #[inline(always)]
    pub fn is_mux100(&self) -> bool {
        *self == Mux::Mux100
    }
    #[doc = "Alternative 5 (chip-specific)"]
    #[inline(always)]
    pub fn is_mux101(&self) -> bool {
        *self == Mux::Mux101
    }
    #[doc = "Alternative 6 (chip-specific)"]
    #[inline(always)]
    pub fn is_mux110(&self) -> bool {
        *self == Mux::Mux110
    }
    #[doc = "Alternative 7 (chip-specific)"]
    #[inline(always)]
    pub fn is_mux111(&self) -> bool {
        *self == Mux::Mux111
    }
}
#[doc = "Field `MUX` writer - Pin Multiplex Control"]
pub type MuxW<'a, REG> = crate::FieldWriter<'a, REG, 3, Mux, crate::Safe>;
impl<'a, REG> MuxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Alternative 0 (GPIO)"]
    #[inline(always)]
    pub fn mux00(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux00)
    }
    #[doc = "Alternative 1 (chip-specific)"]
    #[inline(always)]
    pub fn mux01(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux01)
    }
    #[doc = "Alternative 2 (chip-specific)"]
    #[inline(always)]
    pub fn mux10(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux10)
    }
    #[doc = "Alternative 3 (chip-specific)"]
    #[inline(always)]
    pub fn mux11(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux11)
    }
    #[doc = "Alternative 4 (chip-specific)"]
    #[inline(always)]
    pub fn mux100(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux100)
    }
    #[doc = "Alternative 5 (chip-specific)"]
    #[inline(always)]
    pub fn mux101(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux101)
    }
    #[doc = "Alternative 6 (chip-specific)"]
    #[inline(always)]
    pub fn mux110(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux110)
    }
    #[doc = "Alternative 7 (chip-specific)"]
    #[inline(always)]
    pub fn mux111(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Mux111)
    }
}
#[doc = "Input Buffer Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibe {
    #[doc = "0: Disables"]
    Ibe0 = 0,
    #[doc = "1: Enables"]
    Ibe1 = 1,
}
impl From<Ibe> for bool {
    #[inline(always)]
    fn from(variant: Ibe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBE` reader - Input Buffer Enable"]
pub type IbeR = crate::BitReader<Ibe>;
impl IbeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibe {
        match self.bits {
            false => Ibe::Ibe0,
            true => Ibe::Ibe1,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_ibe0(&self) -> bool {
        *self == Ibe::Ibe0
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_ibe1(&self) -> bool {
        *self == Ibe::Ibe1
    }
}
#[doc = "Field `IBE` writer - Input Buffer Enable"]
pub type IbeW<'a, REG> = crate::BitWriter<'a, REG, Ibe>;
impl<'a, REG> IbeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn ibe0(self) -> &'a mut crate::W<REG> {
        self.variant(Ibe::Ibe0)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn ibe1(self) -> &'a mut crate::W<REG> {
        self.variant(Ibe::Ibe1)
    }
}
#[doc = "Invert Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inv {
    #[doc = "0: Does not invert"]
    Inv0 = 0,
    #[doc = "1: Inverts"]
    Inv1 = 1,
}
impl From<Inv> for bool {
    #[inline(always)]
    fn from(variant: Inv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INV` reader - Invert Input"]
pub type InvR = crate::BitReader<Inv>;
impl InvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Inv {
        match self.bits {
            false => Inv::Inv0,
            true => Inv::Inv1,
        }
    }
    #[doc = "Does not invert"]
    #[inline(always)]
    pub fn is_inv0(&self) -> bool {
        *self == Inv::Inv0
    }
    #[doc = "Inverts"]
    #[inline(always)]
    pub fn is_inv1(&self) -> bool {
        *self == Inv::Inv1
    }
}
#[doc = "Field `INV` writer - Invert Input"]
pub type InvW<'a, REG> = crate::BitWriter<'a, REG, Inv>;
impl<'a, REG> InvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not invert"]
    #[inline(always)]
    pub fn inv0(self) -> &'a mut crate::W<REG> {
        self.variant(Inv::Inv0)
    }
    #[doc = "Inverts"]
    #[inline(always)]
    pub fn inv1(self) -> &'a mut crate::W<REG> {
        self.variant(Inv::Inv1)
    }
}
#[doc = "Lock Register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lk {
    #[doc = "0: Does not lock"]
    Lk0 = 0,
    #[doc = "1: Locks"]
    Lk1 = 1,
}
impl From<Lk> for bool {
    #[inline(always)]
    fn from(variant: Lk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LK` reader - Lock Register"]
pub type LkR = crate::BitReader<Lk>;
impl LkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lk {
        match self.bits {
            false => Lk::Lk0,
            true => Lk::Lk1,
        }
    }
    #[doc = "Does not lock"]
    #[inline(always)]
    pub fn is_lk0(&self) -> bool {
        *self == Lk::Lk0
    }
    #[doc = "Locks"]
    #[inline(always)]
    pub fn is_lk1(&self) -> bool {
        *self == Lk::Lk1
    }
}
#[doc = "Field `LK` writer - Lock Register"]
pub type LkW<'a, REG> = crate::BitWriter<'a, REG, Lk>;
impl<'a, REG> LkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Does not lock"]
    #[inline(always)]
    pub fn lk0(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::Lk0)
    }
    #[doc = "Locks"]
    #[inline(always)]
    pub fn lk1(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::Lk1)
    }
}
impl R {
    #[doc = "Bit 0 - Pull Select"]
    #[inline(always)]
    pub fn ps(&self) -> PsR {
        PsR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Pull Enable"]
    #[inline(always)]
    pub fn pe(&self) -> PeR {
        PeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 3 - Slew Rate Enable"]
    #[inline(always)]
    pub fn sre(&self) -> SreR {
        SreR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 5 - Open Drain Enable"]
    #[inline(always)]
    pub fn ode(&self) -> OdeR {
        OdeR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Drive Strength Enable"]
    #[inline(always)]
    pub fn dse(&self) -> DseR {
        DseR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bits 8:10 - Pin Multiplex Control"]
    #[inline(always)]
    pub fn mux(&self) -> MuxR {
        MuxR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bit 12 - Input Buffer Enable"]
    #[inline(always)]
    pub fn ibe(&self) -> IbeR {
        IbeR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Invert Input"]
    #[inline(always)]
    pub fn inv(&self) -> InvR {
        InvR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - Lock Register"]
    #[inline(always)]
    pub fn lk(&self) -> LkR {
        LkR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Pull Select"]
    #[inline(always)]
    pub fn ps(&mut self) -> PsW<Pcr1Spec> {
        PsW::new(self, 0)
    }
    #[doc = "Bit 1 - Pull Enable"]
    #[inline(always)]
    pub fn pe(&mut self) -> PeW<Pcr1Spec> {
        PeW::new(self, 1)
    }
    #[doc = "Bit 3 - Slew Rate Enable"]
    #[inline(always)]
    pub fn sre(&mut self) -> SreW<Pcr1Spec> {
        SreW::new(self, 3)
    }
    #[doc = "Bit 5 - Open Drain Enable"]
    #[inline(always)]
    pub fn ode(&mut self) -> OdeW<Pcr1Spec> {
        OdeW::new(self, 5)
    }
    #[doc = "Bit 6 - Drive Strength Enable"]
    #[inline(always)]
    pub fn dse(&mut self) -> DseW<Pcr1Spec> {
        DseW::new(self, 6)
    }
    #[doc = "Bits 8:10 - Pin Multiplex Control"]
    #[inline(always)]
    pub fn mux(&mut self) -> MuxW<Pcr1Spec> {
        MuxW::new(self, 8)
    }
    #[doc = "Bit 12 - Input Buffer Enable"]
    #[inline(always)]
    pub fn ibe(&mut self) -> IbeW<Pcr1Spec> {
        IbeW::new(self, 12)
    }
    #[doc = "Bit 13 - Invert Input"]
    #[inline(always)]
    pub fn inv(&mut self) -> InvW<Pcr1Spec> {
        InvW::new(self, 13)
    }
    #[doc = "Bit 15 - Lock Register"]
    #[inline(always)]
    pub fn lk(&mut self) -> LkW<Pcr1Spec> {
        LkW::new(self, 15)
    }
}
#[doc = "Pin Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pcr1Spec;
impl crate::RegisterSpec for Pcr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pcr1::R`](R) reader structure"]
impl crate::Readable for Pcr1Spec {}
#[doc = "`write(|w| ..)` method takes [`pcr1::W`](W) writer structure"]
impl crate::Writable for Pcr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PCR1 to value 0x1102"]
impl crate::Resettable for Pcr1Spec {
    const RESET_VALUE: u32 = 0x1102;
}
