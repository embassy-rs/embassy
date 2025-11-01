#[doc = "Register `RRCR1` reader"]
pub type R = crate::R<Rrcr1Spec>;
#[doc = "Register `RRCR1` writer"]
pub type W = crate::W<Rrcr1Spec>;
#[doc = "Channel 0 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh0en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh0en> for bool {
    #[inline(always)]
    fn from(variant: RrCh0en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH0EN` reader - Channel 0 Input Enable in Trigger Mode"]
pub type RrCh0enR = crate::BitReader<RrCh0en>;
impl RrCh0enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh0en {
        match self.bits {
            false => RrCh0en::Disable,
            true => RrCh0en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh0en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh0en::Enable
    }
}
#[doc = "Field `RR_CH0EN` writer - Channel 0 Input Enable in Trigger Mode"]
pub type RrCh0enW<'a, REG> = crate::BitWriter<'a, REG, RrCh0en>;
impl<'a, REG> RrCh0enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh0en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh0en::Enable)
    }
}
#[doc = "Channel 1 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh1en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh1en> for bool {
    #[inline(always)]
    fn from(variant: RrCh1en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH1EN` reader - Channel 1 Input Enable in Trigger Mode"]
pub type RrCh1enR = crate::BitReader<RrCh1en>;
impl RrCh1enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh1en {
        match self.bits {
            false => RrCh1en::Disable,
            true => RrCh1en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh1en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh1en::Enable
    }
}
#[doc = "Field `RR_CH1EN` writer - Channel 1 Input Enable in Trigger Mode"]
pub type RrCh1enW<'a, REG> = crate::BitWriter<'a, REG, RrCh1en>;
impl<'a, REG> RrCh1enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh1en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh1en::Enable)
    }
}
#[doc = "Channel 2 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh2en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh2en> for bool {
    #[inline(always)]
    fn from(variant: RrCh2en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH2EN` reader - Channel 2 Input Enable in Trigger Mode"]
pub type RrCh2enR = crate::BitReader<RrCh2en>;
impl RrCh2enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh2en {
        match self.bits {
            false => RrCh2en::Disable,
            true => RrCh2en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh2en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh2en::Enable
    }
}
#[doc = "Field `RR_CH2EN` writer - Channel 2 Input Enable in Trigger Mode"]
pub type RrCh2enW<'a, REG> = crate::BitWriter<'a, REG, RrCh2en>;
impl<'a, REG> RrCh2enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh2en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh2en::Enable)
    }
}
#[doc = "Channel 3 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh3en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh3en> for bool {
    #[inline(always)]
    fn from(variant: RrCh3en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH3EN` reader - Channel 3 Input Enable in Trigger Mode"]
pub type RrCh3enR = crate::BitReader<RrCh3en>;
impl RrCh3enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh3en {
        match self.bits {
            false => RrCh3en::Disable,
            true => RrCh3en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh3en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh3en::Enable
    }
}
#[doc = "Field `RR_CH3EN` writer - Channel 3 Input Enable in Trigger Mode"]
pub type RrCh3enW<'a, REG> = crate::BitWriter<'a, REG, RrCh3en>;
impl<'a, REG> RrCh3enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh3en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh3en::Enable)
    }
}
#[doc = "Channel 4 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh4en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh4en> for bool {
    #[inline(always)]
    fn from(variant: RrCh4en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH4EN` reader - Channel 4 Input Enable in Trigger Mode"]
pub type RrCh4enR = crate::BitReader<RrCh4en>;
impl RrCh4enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh4en {
        match self.bits {
            false => RrCh4en::Disable,
            true => RrCh4en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh4en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh4en::Enable
    }
}
#[doc = "Field `RR_CH4EN` writer - Channel 4 Input Enable in Trigger Mode"]
pub type RrCh4enW<'a, REG> = crate::BitWriter<'a, REG, RrCh4en>;
impl<'a, REG> RrCh4enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh4en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh4en::Enable)
    }
}
#[doc = "Channel 5 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh5en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh5en> for bool {
    #[inline(always)]
    fn from(variant: RrCh5en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH5EN` reader - Channel 5 Input Enable in Trigger Mode"]
pub type RrCh5enR = crate::BitReader<RrCh5en>;
impl RrCh5enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh5en {
        match self.bits {
            false => RrCh5en::Disable,
            true => RrCh5en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh5en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh5en::Enable
    }
}
#[doc = "Field `RR_CH5EN` writer - Channel 5 Input Enable in Trigger Mode"]
pub type RrCh5enW<'a, REG> = crate::BitWriter<'a, REG, RrCh5en>;
impl<'a, REG> RrCh5enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh5en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh5en::Enable)
    }
}
#[doc = "Channel 6 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh6en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh6en> for bool {
    #[inline(always)]
    fn from(variant: RrCh6en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH6EN` reader - Channel 6 Input Enable in Trigger Mode"]
pub type RrCh6enR = crate::BitReader<RrCh6en>;
impl RrCh6enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh6en {
        match self.bits {
            false => RrCh6en::Disable,
            true => RrCh6en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh6en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh6en::Enable
    }
}
#[doc = "Field `RR_CH6EN` writer - Channel 6 Input Enable in Trigger Mode"]
pub type RrCh6enW<'a, REG> = crate::BitWriter<'a, REG, RrCh6en>;
impl<'a, REG> RrCh6enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh6en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh6en::Enable)
    }
}
#[doc = "Channel 7 Input Enable in Trigger Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrCh7en {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrCh7en> for bool {
    #[inline(always)]
    fn from(variant: RrCh7en) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_CH7EN` reader - Channel 7 Input Enable in Trigger Mode"]
pub type RrCh7enR = crate::BitReader<RrCh7en>;
impl RrCh7enR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrCh7en {
        match self.bits {
            false => RrCh7en::Disable,
            true => RrCh7en::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrCh7en::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrCh7en::Enable
    }
}
#[doc = "Field `RR_CH7EN` writer - Channel 7 Input Enable in Trigger Mode"]
pub type RrCh7enW<'a, REG> = crate::BitWriter<'a, REG, RrCh7en>;
impl<'a, REG> RrCh7enW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh7en::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrCh7en::Enable)
    }
}
#[doc = "Fixed Port\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixp {
    #[doc = "0: Fix the plus port. Sweep only the inputs to the minus port."]
    FixPlus = 0,
    #[doc = "1: Fix the minus port. Sweep only the inputs to the plus port."]
    FixMinus = 1,
}
impl From<Fixp> for bool {
    #[inline(always)]
    fn from(variant: Fixp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIXP` reader - Fixed Port"]
pub type FixpR = crate::BitReader<Fixp>;
impl FixpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fixp {
        match self.bits {
            false => Fixp::FixPlus,
            true => Fixp::FixMinus,
        }
    }
    #[doc = "Fix the plus port. Sweep only the inputs to the minus port."]
    #[inline(always)]
    pub fn is_fix_plus(&self) -> bool {
        *self == Fixp::FixPlus
    }
    #[doc = "Fix the minus port. Sweep only the inputs to the plus port."]
    #[inline(always)]
    pub fn is_fix_minus(&self) -> bool {
        *self == Fixp::FixMinus
    }
}
#[doc = "Field `FIXP` writer - Fixed Port"]
pub type FixpW<'a, REG> = crate::BitWriter<'a, REG, Fixp>;
impl<'a, REG> FixpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Fix the plus port. Sweep only the inputs to the minus port."]
    #[inline(always)]
    pub fn fix_plus(self) -> &'a mut crate::W<REG> {
        self.variant(Fixp::FixPlus)
    }
    #[doc = "Fix the minus port. Sweep only the inputs to the plus port."]
    #[inline(always)]
    pub fn fix_minus(self) -> &'a mut crate::W<REG> {
        self.variant(Fixp::FixMinus)
    }
}
#[doc = "Fixed Channel Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fixch {
    #[doc = "0: Channel 0"]
    FixCh0 = 0,
    #[doc = "1: Channel 1"]
    FixCh1 = 1,
    #[doc = "2: Channel 2"]
    FixCh2 = 2,
    #[doc = "3: Channel 3"]
    FixCh3 = 3,
    #[doc = "4: Channel 4"]
    FixCh4 = 4,
    #[doc = "5: Channel 5"]
    FixCh5 = 5,
    #[doc = "6: Channel 6"]
    FixCh6 = 6,
    #[doc = "7: Channel 7"]
    FixCh7 = 7,
}
impl From<Fixch> for u8 {
    #[inline(always)]
    fn from(variant: Fixch) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fixch {
    type Ux = u8;
}
impl crate::IsEnum for Fixch {}
#[doc = "Field `FIXCH` reader - Fixed Channel Select"]
pub type FixchR = crate::FieldReader<Fixch>;
impl FixchR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fixch {
        match self.bits {
            0 => Fixch::FixCh0,
            1 => Fixch::FixCh1,
            2 => Fixch::FixCh2,
            3 => Fixch::FixCh3,
            4 => Fixch::FixCh4,
            5 => Fixch::FixCh5,
            6 => Fixch::FixCh6,
            7 => Fixch::FixCh7,
            _ => unreachable!(),
        }
    }
    #[doc = "Channel 0"]
    #[inline(always)]
    pub fn is_fix_ch0(&self) -> bool {
        *self == Fixch::FixCh0
    }
    #[doc = "Channel 1"]
    #[inline(always)]
    pub fn is_fix_ch1(&self) -> bool {
        *self == Fixch::FixCh1
    }
    #[doc = "Channel 2"]
    #[inline(always)]
    pub fn is_fix_ch2(&self) -> bool {
        *self == Fixch::FixCh2
    }
    #[doc = "Channel 3"]
    #[inline(always)]
    pub fn is_fix_ch3(&self) -> bool {
        *self == Fixch::FixCh3
    }
    #[doc = "Channel 4"]
    #[inline(always)]
    pub fn is_fix_ch4(&self) -> bool {
        *self == Fixch::FixCh4
    }
    #[doc = "Channel 5"]
    #[inline(always)]
    pub fn is_fix_ch5(&self) -> bool {
        *self == Fixch::FixCh5
    }
    #[doc = "Channel 6"]
    #[inline(always)]
    pub fn is_fix_ch6(&self) -> bool {
        *self == Fixch::FixCh6
    }
    #[doc = "Channel 7"]
    #[inline(always)]
    pub fn is_fix_ch7(&self) -> bool {
        *self == Fixch::FixCh7
    }
}
#[doc = "Field `FIXCH` writer - Fixed Channel Select"]
pub type FixchW<'a, REG> = crate::FieldWriter<'a, REG, 3, Fixch, crate::Safe>;
impl<'a, REG> FixchW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Channel 0"]
    #[inline(always)]
    pub fn fix_ch0(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh0)
    }
    #[doc = "Channel 1"]
    #[inline(always)]
    pub fn fix_ch1(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh1)
    }
    #[doc = "Channel 2"]
    #[inline(always)]
    pub fn fix_ch2(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh2)
    }
    #[doc = "Channel 3"]
    #[inline(always)]
    pub fn fix_ch3(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh3)
    }
    #[doc = "Channel 4"]
    #[inline(always)]
    pub fn fix_ch4(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh4)
    }
    #[doc = "Channel 5"]
    #[inline(always)]
    pub fn fix_ch5(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh5)
    }
    #[doc = "Channel 6"]
    #[inline(always)]
    pub fn fix_ch6(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh6)
    }
    #[doc = "Channel 7"]
    #[inline(always)]
    pub fn fix_ch7(self) -> &'a mut crate::W<REG> {
        self.variant(Fixch::FixCh7)
    }
}
impl R {
    #[doc = "Bit 0 - Channel 0 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch0en(&self) -> RrCh0enR {
        RrCh0enR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Channel 1 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch1en(&self) -> RrCh1enR {
        RrCh1enR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Channel 2 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch2en(&self) -> RrCh2enR {
        RrCh2enR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Channel 3 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch3en(&self) -> RrCh3enR {
        RrCh3enR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Channel 4 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch4en(&self) -> RrCh4enR {
        RrCh4enR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Channel 5 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch5en(&self) -> RrCh5enR {
        RrCh5enR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Channel 6 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch6en(&self) -> RrCh6enR {
        RrCh6enR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Channel 7 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch7en(&self) -> RrCh7enR {
        RrCh7enR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 16 - Fixed Port"]
    #[inline(always)]
    pub fn fixp(&self) -> FixpR {
        FixpR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bits 20:22 - Fixed Channel Select"]
    #[inline(always)]
    pub fn fixch(&self) -> FixchR {
        FixchR::new(((self.bits >> 20) & 7) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Channel 0 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch0en(&mut self) -> RrCh0enW<Rrcr1Spec> {
        RrCh0enW::new(self, 0)
    }
    #[doc = "Bit 1 - Channel 1 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch1en(&mut self) -> RrCh1enW<Rrcr1Spec> {
        RrCh1enW::new(self, 1)
    }
    #[doc = "Bit 2 - Channel 2 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch2en(&mut self) -> RrCh2enW<Rrcr1Spec> {
        RrCh2enW::new(self, 2)
    }
    #[doc = "Bit 3 - Channel 3 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch3en(&mut self) -> RrCh3enW<Rrcr1Spec> {
        RrCh3enW::new(self, 3)
    }
    #[doc = "Bit 4 - Channel 4 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch4en(&mut self) -> RrCh4enW<Rrcr1Spec> {
        RrCh4enW::new(self, 4)
    }
    #[doc = "Bit 5 - Channel 5 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch5en(&mut self) -> RrCh5enW<Rrcr1Spec> {
        RrCh5enW::new(self, 5)
    }
    #[doc = "Bit 6 - Channel 6 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch6en(&mut self) -> RrCh6enW<Rrcr1Spec> {
        RrCh6enW::new(self, 6)
    }
    #[doc = "Bit 7 - Channel 7 Input Enable in Trigger Mode"]
    #[inline(always)]
    pub fn rr_ch7en(&mut self) -> RrCh7enW<Rrcr1Spec> {
        RrCh7enW::new(self, 7)
    }
    #[doc = "Bit 16 - Fixed Port"]
    #[inline(always)]
    pub fn fixp(&mut self) -> FixpW<Rrcr1Spec> {
        FixpW::new(self, 16)
    }
    #[doc = "Bits 20:22 - Fixed Channel Select"]
    #[inline(always)]
    pub fn fixch(&mut self) -> FixchW<Rrcr1Spec> {
        FixchW::new(self, 20)
    }
}
#[doc = "Round Robin Control Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Rrcr1Spec;
impl crate::RegisterSpec for Rrcr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rrcr1::R`](R) reader structure"]
impl crate::Readable for Rrcr1Spec {}
#[doc = "`write(|w| ..)` method takes [`rrcr1::W`](W) writer structure"]
impl crate::Writable for Rrcr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RRCR1 to value 0"]
impl crate::Resettable for Rrcr1Spec {}
