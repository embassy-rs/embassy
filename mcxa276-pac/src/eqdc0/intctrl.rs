#[doc = "Register `INTCTRL` reader"]
pub type R = crate::R<IntctrlSpec>;
#[doc = "Register `INTCTRL` writer"]
pub type W = crate::W<IntctrlSpec>;
#[doc = "Simultaneous PHASEA and PHASEB Change Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sabie {
    #[doc = "0: Disabled"]
    Sabie0 = 0,
    #[doc = "1: Enabled"]
    Sabie1 = 1,
}
impl From<Sabie> for bool {
    #[inline(always)]
    fn from(variant: Sabie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SABIE` reader - Simultaneous PHASEA and PHASEB Change Interrupt Enable"]
pub type SabieR = crate::BitReader<Sabie>;
impl SabieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sabie {
        match self.bits {
            false => Sabie::Sabie0,
            true => Sabie::Sabie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_sabie0(&self) -> bool {
        *self == Sabie::Sabie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_sabie1(&self) -> bool {
        *self == Sabie::Sabie1
    }
}
#[doc = "Field `SABIE` writer - Simultaneous PHASEA and PHASEB Change Interrupt Enable"]
pub type SabieW<'a, REG> = crate::BitWriter<'a, REG, Sabie>;
impl<'a, REG> SabieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn sabie0(self) -> &'a mut crate::W<REG> {
        self.variant(Sabie::Sabie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn sabie1(self) -> &'a mut crate::W<REG> {
        self.variant(Sabie::Sabie1)
    }
}
#[doc = "Simultaneous PHASEA and PHASEB Change Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sabirq {
    #[doc = "0: No simultaneous change of PHASEA and PHASEB has occurred"]
    Sabirq0 = 0,
    #[doc = "1: A simultaneous change of PHASEA and PHASEB has occurred"]
    Sabirq1 = 1,
}
impl From<Sabirq> for bool {
    #[inline(always)]
    fn from(variant: Sabirq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SABIRQ` reader - Simultaneous PHASEA and PHASEB Change Interrupt Request"]
pub type SabirqR = crate::BitReader<Sabirq>;
impl SabirqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sabirq {
        match self.bits {
            false => Sabirq::Sabirq0,
            true => Sabirq::Sabirq1,
        }
    }
    #[doc = "No simultaneous change of PHASEA and PHASEB has occurred"]
    #[inline(always)]
    pub fn is_sabirq0(&self) -> bool {
        *self == Sabirq::Sabirq0
    }
    #[doc = "A simultaneous change of PHASEA and PHASEB has occurred"]
    #[inline(always)]
    pub fn is_sabirq1(&self) -> bool {
        *self == Sabirq::Sabirq1
    }
}
#[doc = "Field `SABIRQ` writer - Simultaneous PHASEA and PHASEB Change Interrupt Request"]
pub type SabirqW<'a, REG> = crate::BitWriter1C<'a, REG, Sabirq>;
impl<'a, REG> SabirqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No simultaneous change of PHASEA and PHASEB has occurred"]
    #[inline(always)]
    pub fn sabirq0(self) -> &'a mut crate::W<REG> {
        self.variant(Sabirq::Sabirq0)
    }
    #[doc = "A simultaneous change of PHASEA and PHASEB has occurred"]
    #[inline(always)]
    pub fn sabirq1(self) -> &'a mut crate::W<REG> {
        self.variant(Sabirq::Sabirq1)
    }
}
#[doc = "Count direction change interrupt enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dirie {
    #[doc = "0: Disabled"]
    Dirie0 = 0,
    #[doc = "1: Enabled"]
    Dirie1 = 1,
}
impl From<Dirie> for bool {
    #[inline(always)]
    fn from(variant: Dirie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIRIE` reader - Count direction change interrupt enable"]
pub type DirieR = crate::BitReader<Dirie>;
impl DirieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dirie {
        match self.bits {
            false => Dirie::Dirie0,
            true => Dirie::Dirie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_dirie0(&self) -> bool {
        *self == Dirie::Dirie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_dirie1(&self) -> bool {
        *self == Dirie::Dirie1
    }
}
#[doc = "Field `DIRIE` writer - Count direction change interrupt enable"]
pub type DirieW<'a, REG> = crate::BitWriter<'a, REG, Dirie>;
impl<'a, REG> DirieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn dirie0(self) -> &'a mut crate::W<REG> {
        self.variant(Dirie::Dirie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn dirie1(self) -> &'a mut crate::W<REG> {
        self.variant(Dirie::Dirie1)
    }
}
#[doc = "Count direction change interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dirirq {
    #[doc = "0: Count direction unchanged"]
    Dirirq0 = 0,
    #[doc = "1: Count direction changed"]
    Dirirq1 = 1,
}
impl From<Dirirq> for bool {
    #[inline(always)]
    fn from(variant: Dirirq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIRIRQ` reader - Count direction change interrupt"]
pub type DirirqR = crate::BitReader<Dirirq>;
impl DirirqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dirirq {
        match self.bits {
            false => Dirirq::Dirirq0,
            true => Dirirq::Dirirq1,
        }
    }
    #[doc = "Count direction unchanged"]
    #[inline(always)]
    pub fn is_dirirq0(&self) -> bool {
        *self == Dirirq::Dirirq0
    }
    #[doc = "Count direction changed"]
    #[inline(always)]
    pub fn is_dirirq1(&self) -> bool {
        *self == Dirirq::Dirirq1
    }
}
#[doc = "Field `DIRIRQ` writer - Count direction change interrupt"]
pub type DirirqW<'a, REG> = crate::BitWriter1C<'a, REG, Dirirq>;
impl<'a, REG> DirirqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Count direction unchanged"]
    #[inline(always)]
    pub fn dirirq0(self) -> &'a mut crate::W<REG> {
        self.variant(Dirirq::Dirirq0)
    }
    #[doc = "Count direction changed"]
    #[inline(always)]
    pub fn dirirq1(self) -> &'a mut crate::W<REG> {
        self.variant(Dirirq::Dirirq1)
    }
}
#[doc = "Roll-under Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ruie {
    #[doc = "0: Disabled"]
    Ruie0 = 0,
    #[doc = "1: Enabled"]
    Ruie1 = 1,
}
impl From<Ruie> for bool {
    #[inline(always)]
    fn from(variant: Ruie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RUIE` reader - Roll-under Interrupt Enable"]
pub type RuieR = crate::BitReader<Ruie>;
impl RuieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ruie {
        match self.bits {
            false => Ruie::Ruie0,
            true => Ruie::Ruie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_ruie0(&self) -> bool {
        *self == Ruie::Ruie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_ruie1(&self) -> bool {
        *self == Ruie::Ruie1
    }
}
#[doc = "Field `RUIE` writer - Roll-under Interrupt Enable"]
pub type RuieW<'a, REG> = crate::BitWriter<'a, REG, Ruie>;
impl<'a, REG> RuieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn ruie0(self) -> &'a mut crate::W<REG> {
        self.variant(Ruie::Ruie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn ruie1(self) -> &'a mut crate::W<REG> {
        self.variant(Ruie::Ruie1)
    }
}
#[doc = "Roll-under Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ruirq {
    #[doc = "0: No roll-under has occurred"]
    Ruirq0 = 0,
    #[doc = "1: Roll-under has occurred"]
    Ruirq1 = 1,
}
impl From<Ruirq> for bool {
    #[inline(always)]
    fn from(variant: Ruirq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RUIRQ` reader - Roll-under Interrupt Request"]
pub type RuirqR = crate::BitReader<Ruirq>;
impl RuirqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ruirq {
        match self.bits {
            false => Ruirq::Ruirq0,
            true => Ruirq::Ruirq1,
        }
    }
    #[doc = "No roll-under has occurred"]
    #[inline(always)]
    pub fn is_ruirq0(&self) -> bool {
        *self == Ruirq::Ruirq0
    }
    #[doc = "Roll-under has occurred"]
    #[inline(always)]
    pub fn is_ruirq1(&self) -> bool {
        *self == Ruirq::Ruirq1
    }
}
#[doc = "Field `RUIRQ` writer - Roll-under Interrupt Request"]
pub type RuirqW<'a, REG> = crate::BitWriter1C<'a, REG, Ruirq>;
impl<'a, REG> RuirqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No roll-under has occurred"]
    #[inline(always)]
    pub fn ruirq0(self) -> &'a mut crate::W<REG> {
        self.variant(Ruirq::Ruirq0)
    }
    #[doc = "Roll-under has occurred"]
    #[inline(always)]
    pub fn ruirq1(self) -> &'a mut crate::W<REG> {
        self.variant(Ruirq::Ruirq1)
    }
}
#[doc = "Roll-over Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Roie {
    #[doc = "0: Disabled"]
    Roie = 0,
    #[doc = "1: Enabled"]
    Roie1 = 1,
}
impl From<Roie> for bool {
    #[inline(always)]
    fn from(variant: Roie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ROIE` reader - Roll-over Interrupt Enable"]
pub type RoieR = crate::BitReader<Roie>;
impl RoieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Roie {
        match self.bits {
            false => Roie::Roie,
            true => Roie::Roie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_roie(&self) -> bool {
        *self == Roie::Roie
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_roie1(&self) -> bool {
        *self == Roie::Roie1
    }
}
#[doc = "Field `ROIE` writer - Roll-over Interrupt Enable"]
pub type RoieW<'a, REG> = crate::BitWriter<'a, REG, Roie>;
impl<'a, REG> RoieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn roie(self) -> &'a mut crate::W<REG> {
        self.variant(Roie::Roie)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn roie1(self) -> &'a mut crate::W<REG> {
        self.variant(Roie::Roie1)
    }
}
#[doc = "Roll-over Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Roirq {
    #[doc = "0: No roll-over has occurred"]
    Roirq0 = 0,
    #[doc = "1: Roll-over has occurred"]
    Roirq1 = 1,
}
impl From<Roirq> for bool {
    #[inline(always)]
    fn from(variant: Roirq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ROIRQ` reader - Roll-over Interrupt Request"]
pub type RoirqR = crate::BitReader<Roirq>;
impl RoirqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Roirq {
        match self.bits {
            false => Roirq::Roirq0,
            true => Roirq::Roirq1,
        }
    }
    #[doc = "No roll-over has occurred"]
    #[inline(always)]
    pub fn is_roirq0(&self) -> bool {
        *self == Roirq::Roirq0
    }
    #[doc = "Roll-over has occurred"]
    #[inline(always)]
    pub fn is_roirq1(&self) -> bool {
        *self == Roirq::Roirq1
    }
}
#[doc = "Field `ROIRQ` writer - Roll-over Interrupt Request"]
pub type RoirqW<'a, REG> = crate::BitWriter1C<'a, REG, Roirq>;
impl<'a, REG> RoirqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No roll-over has occurred"]
    #[inline(always)]
    pub fn roirq0(self) -> &'a mut crate::W<REG> {
        self.variant(Roirq::Roirq0)
    }
    #[doc = "Roll-over has occurred"]
    #[inline(always)]
    pub fn roirq1(self) -> &'a mut crate::W<REG> {
        self.variant(Roirq::Roirq1)
    }
}
#[doc = "Compare 0 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp0ie {
    #[doc = "0: Disabled"]
    Cmp0ie0 = 0,
    #[doc = "1: Enabled"]
    Cmp0ie1 = 1,
}
impl From<Cmp0ie> for bool {
    #[inline(always)]
    fn from(variant: Cmp0ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP0IE` reader - Compare 0 Interrupt Enable"]
pub type Cmp0ieR = crate::BitReader<Cmp0ie>;
impl Cmp0ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp0ie {
        match self.bits {
            false => Cmp0ie::Cmp0ie0,
            true => Cmp0ie::Cmp0ie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_cmp0ie0(&self) -> bool {
        *self == Cmp0ie::Cmp0ie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_cmp0ie1(&self) -> bool {
        *self == Cmp0ie::Cmp0ie1
    }
}
#[doc = "Field `CMP0IE` writer - Compare 0 Interrupt Enable"]
pub type Cmp0ieW<'a, REG> = crate::BitWriter<'a, REG, Cmp0ie>;
impl<'a, REG> Cmp0ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn cmp0ie0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp0ie::Cmp0ie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn cmp0ie1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp0ie::Cmp0ie1)
    }
}
#[doc = "Compare 0 Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp0irq {
    #[doc = "0: No match has occurred (the position counter does not match the COMP0 value)"]
    Cmp0irq0 = 0,
    #[doc = "1: COMP match has occurred (the position counter matches the COMP0 value)"]
    Cmp0irq1 = 1,
}
impl From<Cmp0irq> for bool {
    #[inline(always)]
    fn from(variant: Cmp0irq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP0IRQ` reader - Compare 0 Interrupt Request"]
pub type Cmp0irqR = crate::BitReader<Cmp0irq>;
impl Cmp0irqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp0irq {
        match self.bits {
            false => Cmp0irq::Cmp0irq0,
            true => Cmp0irq::Cmp0irq1,
        }
    }
    #[doc = "No match has occurred (the position counter does not match the COMP0 value)"]
    #[inline(always)]
    pub fn is_cmp0irq0(&self) -> bool {
        *self == Cmp0irq::Cmp0irq0
    }
    #[doc = "COMP match has occurred (the position counter matches the COMP0 value)"]
    #[inline(always)]
    pub fn is_cmp0irq1(&self) -> bool {
        *self == Cmp0irq::Cmp0irq1
    }
}
#[doc = "Field `CMP0IRQ` writer - Compare 0 Interrupt Request"]
pub type Cmp0irqW<'a, REG> = crate::BitWriter1C<'a, REG, Cmp0irq>;
impl<'a, REG> Cmp0irqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No match has occurred (the position counter does not match the COMP0 value)"]
    #[inline(always)]
    pub fn cmp0irq0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp0irq::Cmp0irq0)
    }
    #[doc = "COMP match has occurred (the position counter matches the COMP0 value)"]
    #[inline(always)]
    pub fn cmp0irq1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp0irq::Cmp0irq1)
    }
}
#[doc = "Compare1 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp1ie {
    #[doc = "0: Disabled"]
    Cmp1ie0 = 0,
    #[doc = "1: Enabled"]
    Cmp1ie1 = 1,
}
impl From<Cmp1ie> for bool {
    #[inline(always)]
    fn from(variant: Cmp1ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP1IE` reader - Compare1 Interrupt Enable"]
pub type Cmp1ieR = crate::BitReader<Cmp1ie>;
impl Cmp1ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp1ie {
        match self.bits {
            false => Cmp1ie::Cmp1ie0,
            true => Cmp1ie::Cmp1ie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_cmp1ie0(&self) -> bool {
        *self == Cmp1ie::Cmp1ie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_cmp1ie1(&self) -> bool {
        *self == Cmp1ie::Cmp1ie1
    }
}
#[doc = "Field `CMP1IE` writer - Compare1 Interrupt Enable"]
pub type Cmp1ieW<'a, REG> = crate::BitWriter<'a, REG, Cmp1ie>;
impl<'a, REG> Cmp1ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn cmp1ie0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp1ie::Cmp1ie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn cmp1ie1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp1ie::Cmp1ie1)
    }
}
#[doc = "Compare1 Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp1irq {
    #[doc = "0: No match has occurred (the position counter does not match the COMP1 value)"]
    Cmp1irq0 = 0,
    #[doc = "1: COMP1 match has occurred (the position counter matches the COMP1 value)"]
    Cmp1irq1 = 1,
}
impl From<Cmp1irq> for bool {
    #[inline(always)]
    fn from(variant: Cmp1irq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP1IRQ` reader - Compare1 Interrupt Request"]
pub type Cmp1irqR = crate::BitReader<Cmp1irq>;
impl Cmp1irqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp1irq {
        match self.bits {
            false => Cmp1irq::Cmp1irq0,
            true => Cmp1irq::Cmp1irq1,
        }
    }
    #[doc = "No match has occurred (the position counter does not match the COMP1 value)"]
    #[inline(always)]
    pub fn is_cmp1irq0(&self) -> bool {
        *self == Cmp1irq::Cmp1irq0
    }
    #[doc = "COMP1 match has occurred (the position counter matches the COMP1 value)"]
    #[inline(always)]
    pub fn is_cmp1irq1(&self) -> bool {
        *self == Cmp1irq::Cmp1irq1
    }
}
#[doc = "Field `CMP1IRQ` writer - Compare1 Interrupt Request"]
pub type Cmp1irqW<'a, REG> = crate::BitWriter1C<'a, REG, Cmp1irq>;
impl<'a, REG> Cmp1irqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No match has occurred (the position counter does not match the COMP1 value)"]
    #[inline(always)]
    pub fn cmp1irq0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp1irq::Cmp1irq0)
    }
    #[doc = "COMP1 match has occurred (the position counter matches the COMP1 value)"]
    #[inline(always)]
    pub fn cmp1irq1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp1irq::Cmp1irq1)
    }
}
#[doc = "Compare2 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp2ie {
    #[doc = "0: Disabled"]
    Cmp2ie0 = 0,
    #[doc = "1: Enabled"]
    Cmp2ie1 = 1,
}
impl From<Cmp2ie> for bool {
    #[inline(always)]
    fn from(variant: Cmp2ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP2IE` reader - Compare2 Interrupt Enable"]
pub type Cmp2ieR = crate::BitReader<Cmp2ie>;
impl Cmp2ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp2ie {
        match self.bits {
            false => Cmp2ie::Cmp2ie0,
            true => Cmp2ie::Cmp2ie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_cmp2ie0(&self) -> bool {
        *self == Cmp2ie::Cmp2ie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_cmp2ie1(&self) -> bool {
        *self == Cmp2ie::Cmp2ie1
    }
}
#[doc = "Field `CMP2IE` writer - Compare2 Interrupt Enable"]
pub type Cmp2ieW<'a, REG> = crate::BitWriter<'a, REG, Cmp2ie>;
impl<'a, REG> Cmp2ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn cmp2ie0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp2ie::Cmp2ie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn cmp2ie1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp2ie::Cmp2ie1)
    }
}
#[doc = "Compare2 Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp2irq {
    #[doc = "0: No match has occurred (the position counter does not match the COMP2 value)"]
    Cmp2irq0 = 0,
    #[doc = "1: COMP2 match has occurred (the position counter matches the COMP2 value)"]
    Cmp2irq1 = 1,
}
impl From<Cmp2irq> for bool {
    #[inline(always)]
    fn from(variant: Cmp2irq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP2IRQ` reader - Compare2 Interrupt Request"]
pub type Cmp2irqR = crate::BitReader<Cmp2irq>;
impl Cmp2irqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp2irq {
        match self.bits {
            false => Cmp2irq::Cmp2irq0,
            true => Cmp2irq::Cmp2irq1,
        }
    }
    #[doc = "No match has occurred (the position counter does not match the COMP2 value)"]
    #[inline(always)]
    pub fn is_cmp2irq0(&self) -> bool {
        *self == Cmp2irq::Cmp2irq0
    }
    #[doc = "COMP2 match has occurred (the position counter matches the COMP2 value)"]
    #[inline(always)]
    pub fn is_cmp2irq1(&self) -> bool {
        *self == Cmp2irq::Cmp2irq1
    }
}
#[doc = "Field `CMP2IRQ` writer - Compare2 Interrupt Request"]
pub type Cmp2irqW<'a, REG> = crate::BitWriter1C<'a, REG, Cmp2irq>;
impl<'a, REG> Cmp2irqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No match has occurred (the position counter does not match the COMP2 value)"]
    #[inline(always)]
    pub fn cmp2irq0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp2irq::Cmp2irq0)
    }
    #[doc = "COMP2 match has occurred (the position counter matches the COMP2 value)"]
    #[inline(always)]
    pub fn cmp2irq1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp2irq::Cmp2irq1)
    }
}
#[doc = "Compare3 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp3ie {
    #[doc = "0: Disabled"]
    Cmp3ie0 = 0,
    #[doc = "1: Enabled"]
    Cmp3ie1 = 1,
}
impl From<Cmp3ie> for bool {
    #[inline(always)]
    fn from(variant: Cmp3ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP3IE` reader - Compare3 Interrupt Enable"]
pub type Cmp3ieR = crate::BitReader<Cmp3ie>;
impl Cmp3ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp3ie {
        match self.bits {
            false => Cmp3ie::Cmp3ie0,
            true => Cmp3ie::Cmp3ie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_cmp3ie0(&self) -> bool {
        *self == Cmp3ie::Cmp3ie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_cmp3ie1(&self) -> bool {
        *self == Cmp3ie::Cmp3ie1
    }
}
#[doc = "Field `CMP3IE` writer - Compare3 Interrupt Enable"]
pub type Cmp3ieW<'a, REG> = crate::BitWriter<'a, REG, Cmp3ie>;
impl<'a, REG> Cmp3ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn cmp3ie0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp3ie::Cmp3ie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn cmp3ie1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp3ie::Cmp3ie1)
    }
}
#[doc = "Compare3 Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp3irq {
    #[doc = "0: No match has occurred (the position counter does not match the COMP3 value)"]
    Cmp3irq0 = 0,
    #[doc = "1: COMP3 match has occurred (the position counter matches the COMP3 value)"]
    Cmp3irq1 = 1,
}
impl From<Cmp3irq> for bool {
    #[inline(always)]
    fn from(variant: Cmp3irq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP3IRQ` reader - Compare3 Interrupt Request"]
pub type Cmp3irqR = crate::BitReader<Cmp3irq>;
impl Cmp3irqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmp3irq {
        match self.bits {
            false => Cmp3irq::Cmp3irq0,
            true => Cmp3irq::Cmp3irq1,
        }
    }
    #[doc = "No match has occurred (the position counter does not match the COMP3 value)"]
    #[inline(always)]
    pub fn is_cmp3irq0(&self) -> bool {
        *self == Cmp3irq::Cmp3irq0
    }
    #[doc = "COMP3 match has occurred (the position counter matches the COMP3 value)"]
    #[inline(always)]
    pub fn is_cmp3irq1(&self) -> bool {
        *self == Cmp3irq::Cmp3irq1
    }
}
#[doc = "Field `CMP3IRQ` writer - Compare3 Interrupt Request"]
pub type Cmp3irqW<'a, REG> = crate::BitWriter1C<'a, REG, Cmp3irq>;
impl<'a, REG> Cmp3irqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No match has occurred (the position counter does not match the COMP3 value)"]
    #[inline(always)]
    pub fn cmp3irq0(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp3irq::Cmp3irq0)
    }
    #[doc = "COMP3 match has occurred (the position counter matches the COMP3 value)"]
    #[inline(always)]
    pub fn cmp3irq1(self) -> &'a mut crate::W<REG> {
        self.variant(Cmp3irq::Cmp3irq1)
    }
}
impl R {
    #[doc = "Bit 0 - Simultaneous PHASEA and PHASEB Change Interrupt Enable"]
    #[inline(always)]
    pub fn sabie(&self) -> SabieR {
        SabieR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Simultaneous PHASEA and PHASEB Change Interrupt Request"]
    #[inline(always)]
    pub fn sabirq(&self) -> SabirqR {
        SabirqR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Count direction change interrupt enable"]
    #[inline(always)]
    pub fn dirie(&self) -> DirieR {
        DirieR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Count direction change interrupt"]
    #[inline(always)]
    pub fn dirirq(&self) -> DirirqR {
        DirirqR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Roll-under Interrupt Enable"]
    #[inline(always)]
    pub fn ruie(&self) -> RuieR {
        RuieR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Roll-under Interrupt Request"]
    #[inline(always)]
    pub fn ruirq(&self) -> RuirqR {
        RuirqR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Roll-over Interrupt Enable"]
    #[inline(always)]
    pub fn roie(&self) -> RoieR {
        RoieR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Roll-over Interrupt Request"]
    #[inline(always)]
    pub fn roirq(&self) -> RoirqR {
        RoirqR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Compare 0 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp0ie(&self) -> Cmp0ieR {
        Cmp0ieR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Compare 0 Interrupt Request"]
    #[inline(always)]
    pub fn cmp0irq(&self) -> Cmp0irqR {
        Cmp0irqR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Compare1 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp1ie(&self) -> Cmp1ieR {
        Cmp1ieR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Compare1 Interrupt Request"]
    #[inline(always)]
    pub fn cmp1irq(&self) -> Cmp1irqR {
        Cmp1irqR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Compare2 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp2ie(&self) -> Cmp2ieR {
        Cmp2ieR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Compare2 Interrupt Request"]
    #[inline(always)]
    pub fn cmp2irq(&self) -> Cmp2irqR {
        Cmp2irqR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Compare3 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp3ie(&self) -> Cmp3ieR {
        Cmp3ieR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Compare3 Interrupt Request"]
    #[inline(always)]
    pub fn cmp3irq(&self) -> Cmp3irqR {
        Cmp3irqR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Simultaneous PHASEA and PHASEB Change Interrupt Enable"]
    #[inline(always)]
    pub fn sabie(&mut self) -> SabieW<IntctrlSpec> {
        SabieW::new(self, 0)
    }
    #[doc = "Bit 1 - Simultaneous PHASEA and PHASEB Change Interrupt Request"]
    #[inline(always)]
    pub fn sabirq(&mut self) -> SabirqW<IntctrlSpec> {
        SabirqW::new(self, 1)
    }
    #[doc = "Bit 2 - Count direction change interrupt enable"]
    #[inline(always)]
    pub fn dirie(&mut self) -> DirieW<IntctrlSpec> {
        DirieW::new(self, 2)
    }
    #[doc = "Bit 3 - Count direction change interrupt"]
    #[inline(always)]
    pub fn dirirq(&mut self) -> DirirqW<IntctrlSpec> {
        DirirqW::new(self, 3)
    }
    #[doc = "Bit 4 - Roll-under Interrupt Enable"]
    #[inline(always)]
    pub fn ruie(&mut self) -> RuieW<IntctrlSpec> {
        RuieW::new(self, 4)
    }
    #[doc = "Bit 5 - Roll-under Interrupt Request"]
    #[inline(always)]
    pub fn ruirq(&mut self) -> RuirqW<IntctrlSpec> {
        RuirqW::new(self, 5)
    }
    #[doc = "Bit 6 - Roll-over Interrupt Enable"]
    #[inline(always)]
    pub fn roie(&mut self) -> RoieW<IntctrlSpec> {
        RoieW::new(self, 6)
    }
    #[doc = "Bit 7 - Roll-over Interrupt Request"]
    #[inline(always)]
    pub fn roirq(&mut self) -> RoirqW<IntctrlSpec> {
        RoirqW::new(self, 7)
    }
    #[doc = "Bit 8 - Compare 0 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp0ie(&mut self) -> Cmp0ieW<IntctrlSpec> {
        Cmp0ieW::new(self, 8)
    }
    #[doc = "Bit 9 - Compare 0 Interrupt Request"]
    #[inline(always)]
    pub fn cmp0irq(&mut self) -> Cmp0irqW<IntctrlSpec> {
        Cmp0irqW::new(self, 9)
    }
    #[doc = "Bit 10 - Compare1 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp1ie(&mut self) -> Cmp1ieW<IntctrlSpec> {
        Cmp1ieW::new(self, 10)
    }
    #[doc = "Bit 11 - Compare1 Interrupt Request"]
    #[inline(always)]
    pub fn cmp1irq(&mut self) -> Cmp1irqW<IntctrlSpec> {
        Cmp1irqW::new(self, 11)
    }
    #[doc = "Bit 12 - Compare2 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp2ie(&mut self) -> Cmp2ieW<IntctrlSpec> {
        Cmp2ieW::new(self, 12)
    }
    #[doc = "Bit 13 - Compare2 Interrupt Request"]
    #[inline(always)]
    pub fn cmp2irq(&mut self) -> Cmp2irqW<IntctrlSpec> {
        Cmp2irqW::new(self, 13)
    }
    #[doc = "Bit 14 - Compare3 Interrupt Enable"]
    #[inline(always)]
    pub fn cmp3ie(&mut self) -> Cmp3ieW<IntctrlSpec> {
        Cmp3ieW::new(self, 14)
    }
    #[doc = "Bit 15 - Compare3 Interrupt Request"]
    #[inline(always)]
    pub fn cmp3irq(&mut self) -> Cmp3irqW<IntctrlSpec> {
        Cmp3irqW::new(self, 15)
    }
}
#[doc = "Interrupt Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`intctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`intctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IntctrlSpec;
impl crate::RegisterSpec for IntctrlSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`intctrl::R`](R) reader structure"]
impl crate::Readable for IntctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`intctrl::W`](W) writer structure"]
impl crate::Writable for IntctrlSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u16 = 0xaaaa;
}
#[doc = "`reset()` method sets INTCTRL to value 0"]
impl crate::Resettable for IntctrlSpec {}
