#[doc = "Register `MBC0_MEMN_GLBAC5` reader"]
pub type R = crate::R<Mbc0MemnGlbac5Spec>;
#[doc = "Register `MBC0_MEMN_GLBAC5` writer"]
pub type W = crate::W<Mbc0MemnGlbac5Spec>;
#[doc = "NonsecureUser Execute\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nux {
    #[doc = "0: Execute access is not allowed in Nonsecure User mode."]
    Notallowed = 0,
    #[doc = "1: Execute access is allowed in Nonsecure User mode."]
    Allowed = 1,
}
impl From<Nux> for bool {
    #[inline(always)]
    fn from(variant: Nux) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NUX` reader - NonsecureUser Execute"]
pub type NuxR = crate::BitReader<Nux>;
impl NuxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nux {
        match self.bits {
            false => Nux::Notallowed,
            true => Nux::Allowed,
        }
    }
    #[doc = "Execute access is not allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nux::Notallowed
    }
    #[doc = "Execute access is allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nux::Allowed
    }
}
#[doc = "Field `NUX` writer - NonsecureUser Execute"]
pub type NuxW<'a, REG> = crate::BitWriter<'a, REG, Nux>;
impl<'a, REG> NuxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute access is not allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nux::Notallowed)
    }
    #[doc = "Execute access is allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nux::Allowed)
    }
}
#[doc = "NonsecureUser Write\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nuw {
    #[doc = "0: Write access is not allowed in Nonsecure User mode."]
    Notallowed = 0,
    #[doc = "1: Write access is allowed in Nonsecure User mode."]
    Allowed = 1,
}
impl From<Nuw> for bool {
    #[inline(always)]
    fn from(variant: Nuw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NUW` reader - NonsecureUser Write"]
pub type NuwR = crate::BitReader<Nuw>;
impl NuwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nuw {
        match self.bits {
            false => Nuw::Notallowed,
            true => Nuw::Allowed,
        }
    }
    #[doc = "Write access is not allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nuw::Notallowed
    }
    #[doc = "Write access is allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nuw::Allowed
    }
}
#[doc = "Field `NUW` writer - NonsecureUser Write"]
pub type NuwW<'a, REG> = crate::BitWriter<'a, REG, Nuw>;
impl<'a, REG> NuwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Write access is not allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nuw::Notallowed)
    }
    #[doc = "Write access is allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nuw::Allowed)
    }
}
#[doc = "NonsecureUser Read\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nur {
    #[doc = "0: Read access is not allowed in Nonsecure User mode."]
    Notallowed = 0,
    #[doc = "1: Read access is allowed in Nonsecure User mode."]
    Allowed = 1,
}
impl From<Nur> for bool {
    #[inline(always)]
    fn from(variant: Nur) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NUR` reader - NonsecureUser Read"]
pub type NurR = crate::BitReader<Nur>;
impl NurR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nur {
        match self.bits {
            false => Nur::Notallowed,
            true => Nur::Allowed,
        }
    }
    #[doc = "Read access is not allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Nur::Notallowed
    }
    #[doc = "Read access is allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Nur::Allowed
    }
}
#[doc = "Field `NUR` writer - NonsecureUser Read"]
pub type NurW<'a, REG> = crate::BitWriter<'a, REG, Nur>;
impl<'a, REG> NurW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Read access is not allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nur::Notallowed)
    }
    #[doc = "Read access is allowed in Nonsecure User mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Nur::Allowed)
    }
}
#[doc = "NonsecurePriv Execute\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Npx {
    #[doc = "0: Execute access is not allowed in Nonsecure Privilege mode."]
    Notallowed = 0,
    #[doc = "1: Execute access is allowed in Nonsecure Privilege mode."]
    Allowed = 1,
}
impl From<Npx> for bool {
    #[inline(always)]
    fn from(variant: Npx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NPX` reader - NonsecurePriv Execute"]
pub type NpxR = crate::BitReader<Npx>;
impl NpxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Npx {
        match self.bits {
            false => Npx::Notallowed,
            true => Npx::Allowed,
        }
    }
    #[doc = "Execute access is not allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Npx::Notallowed
    }
    #[doc = "Execute access is allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Npx::Allowed
    }
}
#[doc = "Field `NPX` writer - NonsecurePriv Execute"]
pub type NpxW<'a, REG> = crate::BitWriter<'a, REG, Npx>;
impl<'a, REG> NpxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute access is not allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Npx::Notallowed)
    }
    #[doc = "Execute access is allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Npx::Allowed)
    }
}
#[doc = "NonsecurePriv Write\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Npw {
    #[doc = "0: Write access is not allowed in Nonsecure Privilege mode."]
    Notallowed = 0,
    #[doc = "1: Write access is allowed in Nonsecure Privilege mode."]
    Allowed = 1,
}
impl From<Npw> for bool {
    #[inline(always)]
    fn from(variant: Npw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NPW` reader - NonsecurePriv Write"]
pub type NpwR = crate::BitReader<Npw>;
impl NpwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Npw {
        match self.bits {
            false => Npw::Notallowed,
            true => Npw::Allowed,
        }
    }
    #[doc = "Write access is not allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Npw::Notallowed
    }
    #[doc = "Write access is allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Npw::Allowed
    }
}
#[doc = "Field `NPW` writer - NonsecurePriv Write"]
pub type NpwW<'a, REG> = crate::BitWriter<'a, REG, Npw>;
impl<'a, REG> NpwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Write access is not allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Npw::Notallowed)
    }
    #[doc = "Write access is allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Npw::Allowed)
    }
}
#[doc = "NonsecurePriv Read\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Npr {
    #[doc = "0: Read access is not allowed in Nonsecure Privilege mode."]
    Notallowed = 0,
    #[doc = "1: Read access is allowed in Nonsecure Privilege mode."]
    Allowed = 1,
}
impl From<Npr> for bool {
    #[inline(always)]
    fn from(variant: Npr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NPR` reader - NonsecurePriv Read"]
pub type NprR = crate::BitReader<Npr>;
impl NprR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Npr {
        match self.bits {
            false => Npr::Notallowed,
            true => Npr::Allowed,
        }
    }
    #[doc = "Read access is not allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Npr::Notallowed
    }
    #[doc = "Read access is allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Npr::Allowed
    }
}
#[doc = "Field `NPR` writer - NonsecurePriv Read"]
pub type NprW<'a, REG> = crate::BitWriter<'a, REG, Npr>;
impl<'a, REG> NprW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Read access is not allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Npr::Notallowed)
    }
    #[doc = "Read access is allowed in Nonsecure Privilege mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Npr::Allowed)
    }
}
#[doc = "SecureUser Execute\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sux {
    #[doc = "0: Execute access is not allowed in Secure User mode."]
    Notallowed = 0,
    #[doc = "1: Execute access is allowed in Secure User mode."]
    Allowed = 1,
}
impl From<Sux> for bool {
    #[inline(always)]
    fn from(variant: Sux) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUX` reader - SecureUser Execute"]
pub type SuxR = crate::BitReader<Sux>;
impl SuxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sux {
        match self.bits {
            false => Sux::Notallowed,
            true => Sux::Allowed,
        }
    }
    #[doc = "Execute access is not allowed in Secure User mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Sux::Notallowed
    }
    #[doc = "Execute access is allowed in Secure User mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Sux::Allowed
    }
}
#[doc = "Field `SUX` writer - SecureUser Execute"]
pub type SuxW<'a, REG> = crate::BitWriter<'a, REG, Sux>;
impl<'a, REG> SuxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute access is not allowed in Secure User mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Sux::Notallowed)
    }
    #[doc = "Execute access is allowed in Secure User mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Sux::Allowed)
    }
}
#[doc = "SecureUser Write\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suw {
    #[doc = "0: Write access is not allowed in Secure User mode."]
    Notallowed = 0,
    #[doc = "1: Write access is allowed in Secure User mode."]
    Allowed = 1,
}
impl From<Suw> for bool {
    #[inline(always)]
    fn from(variant: Suw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUW` reader - SecureUser Write"]
pub type SuwR = crate::BitReader<Suw>;
impl SuwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Suw {
        match self.bits {
            false => Suw::Notallowed,
            true => Suw::Allowed,
        }
    }
    #[doc = "Write access is not allowed in Secure User mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Suw::Notallowed
    }
    #[doc = "Write access is allowed in Secure User mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Suw::Allowed
    }
}
#[doc = "Field `SUW` writer - SecureUser Write"]
pub type SuwW<'a, REG> = crate::BitWriter<'a, REG, Suw>;
impl<'a, REG> SuwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Write access is not allowed in Secure User mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Suw::Notallowed)
    }
    #[doc = "Write access is allowed in Secure User mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Suw::Allowed)
    }
}
#[doc = "SecureUser Read\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sur {
    #[doc = "0: Read access is not allowed in Secure User mode."]
    Notallowed = 0,
    #[doc = "1: Read access is allowed in Secure User mode."]
    Allowed = 1,
}
impl From<Sur> for bool {
    #[inline(always)]
    fn from(variant: Sur) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUR` reader - SecureUser Read"]
pub type SurR = crate::BitReader<Sur>;
impl SurR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sur {
        match self.bits {
            false => Sur::Notallowed,
            true => Sur::Allowed,
        }
    }
    #[doc = "Read access is not allowed in Secure User mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Sur::Notallowed
    }
    #[doc = "Read access is allowed in Secure User mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Sur::Allowed
    }
}
#[doc = "Field `SUR` writer - SecureUser Read"]
pub type SurW<'a, REG> = crate::BitWriter<'a, REG, Sur>;
impl<'a, REG> SurW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Read access is not allowed in Secure User mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Sur::Notallowed)
    }
    #[doc = "Read access is allowed in Secure User mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Sur::Allowed)
    }
}
#[doc = "SecurePriv Execute\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spx {
    #[doc = "0: Execute access is not allowed in Secure Privilege mode."]
    Notallowed = 0,
    #[doc = "1: Execute access is allowed in Secure Privilege mode."]
    Allowed = 1,
}
impl From<Spx> for bool {
    #[inline(always)]
    fn from(variant: Spx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPX` reader - SecurePriv Execute"]
pub type SpxR = crate::BitReader<Spx>;
impl SpxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spx {
        match self.bits {
            false => Spx::Notallowed,
            true => Spx::Allowed,
        }
    }
    #[doc = "Execute access is not allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Spx::Notallowed
    }
    #[doc = "Execute access is allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Spx::Allowed
    }
}
#[doc = "Field `SPX` writer - SecurePriv Execute"]
pub type SpxW<'a, REG> = crate::BitWriter<'a, REG, Spx>;
impl<'a, REG> SpxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Execute access is not allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Spx::Notallowed)
    }
    #[doc = "Execute access is allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Spx::Allowed)
    }
}
#[doc = "SecurePriv Write\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spw {
    #[doc = "0: Write access is not allowed in Secure Privilege mode."]
    Notallowed = 0,
    #[doc = "1: Write access is allowed in Secure Privilege mode."]
    Allowed = 1,
}
impl From<Spw> for bool {
    #[inline(always)]
    fn from(variant: Spw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPW` reader - SecurePriv Write"]
pub type SpwR = crate::BitReader<Spw>;
impl SpwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spw {
        match self.bits {
            false => Spw::Notallowed,
            true => Spw::Allowed,
        }
    }
    #[doc = "Write access is not allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Spw::Notallowed
    }
    #[doc = "Write access is allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Spw::Allowed
    }
}
#[doc = "Field `SPW` writer - SecurePriv Write"]
pub type SpwW<'a, REG> = crate::BitWriter<'a, REG, Spw>;
impl<'a, REG> SpwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Write access is not allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Spw::Notallowed)
    }
    #[doc = "Write access is allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Spw::Allowed)
    }
}
#[doc = "SecurePriv Read\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spr {
    #[doc = "0: Read access is not allowed in Secure Privilege mode."]
    Notallowed = 0,
    #[doc = "1: Read access is allowed in Secure Privilege mode."]
    Allowed = 1,
}
impl From<Spr> for bool {
    #[inline(always)]
    fn from(variant: Spr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPR` reader - SecurePriv Read"]
pub type SprR = crate::BitReader<Spr>;
impl SprR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spr {
        match self.bits {
            false => Spr::Notallowed,
            true => Spr::Allowed,
        }
    }
    #[doc = "Read access is not allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn is_notallowed(&self) -> bool {
        *self == Spr::Notallowed
    }
    #[doc = "Read access is allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn is_allowed(&self) -> bool {
        *self == Spr::Allowed
    }
}
#[doc = "Field `SPR` writer - SecurePriv Read"]
pub type SprW<'a, REG> = crate::BitWriter<'a, REG, Spr>;
impl<'a, REG> SprW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Read access is not allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn notallowed(self) -> &'a mut crate::W<REG> {
        self.variant(Spr::Notallowed)
    }
    #[doc = "Read access is allowed in Secure Privilege mode."]
    #[inline(always)]
    pub fn allowed(self) -> &'a mut crate::W<REG> {
        self.variant(Spr::Allowed)
    }
}
#[doc = "LOCK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lk {
    #[doc = "0: This register is not locked and can be altered."]
    Unlocked = 0,
    #[doc = "1: This register is locked and cannot be altered."]
    Locked = 1,
}
impl From<Lk> for bool {
    #[inline(always)]
    fn from(variant: Lk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LK` reader - LOCK"]
pub type LkR = crate::BitReader<Lk>;
impl LkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lk {
        match self.bits {
            false => Lk::Unlocked,
            true => Lk::Locked,
        }
    }
    #[doc = "This register is not locked and can be altered."]
    #[inline(always)]
    pub fn is_unlocked(&self) -> bool {
        *self == Lk::Unlocked
    }
    #[doc = "This register is locked and cannot be altered."]
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        *self == Lk::Locked
    }
}
#[doc = "Field `LK` writer - LOCK"]
pub type LkW<'a, REG> = crate::BitWriter<'a, REG, Lk>;
impl<'a, REG> LkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "This register is not locked and can be altered."]
    #[inline(always)]
    pub fn unlocked(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::Unlocked)
    }
    #[doc = "This register is locked and cannot be altered."]
    #[inline(always)]
    pub fn locked(self) -> &'a mut crate::W<REG> {
        self.variant(Lk::Locked)
    }
}
impl R {
    #[doc = "Bit 0 - NonsecureUser Execute"]
    #[inline(always)]
    pub fn nux(&self) -> NuxR {
        NuxR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - NonsecureUser Write"]
    #[inline(always)]
    pub fn nuw(&self) -> NuwR {
        NuwR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - NonsecureUser Read"]
    #[inline(always)]
    pub fn nur(&self) -> NurR {
        NurR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 4 - NonsecurePriv Execute"]
    #[inline(always)]
    pub fn npx(&self) -> NpxR {
        NpxR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - NonsecurePriv Write"]
    #[inline(always)]
    pub fn npw(&self) -> NpwR {
        NpwR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - NonsecurePriv Read"]
    #[inline(always)]
    pub fn npr(&self) -> NprR {
        NprR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 8 - SecureUser Execute"]
    #[inline(always)]
    pub fn sux(&self) -> SuxR {
        SuxR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - SecureUser Write"]
    #[inline(always)]
    pub fn suw(&self) -> SuwR {
        SuwR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - SecureUser Read"]
    #[inline(always)]
    pub fn sur(&self) -> SurR {
        SurR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 12 - SecurePriv Execute"]
    #[inline(always)]
    pub fn spx(&self) -> SpxR {
        SpxR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - SecurePriv Write"]
    #[inline(always)]
    pub fn spw(&self) -> SpwR {
        SpwR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - SecurePriv Read"]
    #[inline(always)]
    pub fn spr(&self) -> SprR {
        SprR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 31 - LOCK"]
    #[inline(always)]
    pub fn lk(&self) -> LkR {
        LkR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - NonsecureUser Execute"]
    #[inline(always)]
    pub fn nux(&mut self) -> NuxW<Mbc0MemnGlbac5Spec> {
        NuxW::new(self, 0)
    }
    #[doc = "Bit 1 - NonsecureUser Write"]
    #[inline(always)]
    pub fn nuw(&mut self) -> NuwW<Mbc0MemnGlbac5Spec> {
        NuwW::new(self, 1)
    }
    #[doc = "Bit 2 - NonsecureUser Read"]
    #[inline(always)]
    pub fn nur(&mut self) -> NurW<Mbc0MemnGlbac5Spec> {
        NurW::new(self, 2)
    }
    #[doc = "Bit 4 - NonsecurePriv Execute"]
    #[inline(always)]
    pub fn npx(&mut self) -> NpxW<Mbc0MemnGlbac5Spec> {
        NpxW::new(self, 4)
    }
    #[doc = "Bit 5 - NonsecurePriv Write"]
    #[inline(always)]
    pub fn npw(&mut self) -> NpwW<Mbc0MemnGlbac5Spec> {
        NpwW::new(self, 5)
    }
    #[doc = "Bit 6 - NonsecurePriv Read"]
    #[inline(always)]
    pub fn npr(&mut self) -> NprW<Mbc0MemnGlbac5Spec> {
        NprW::new(self, 6)
    }
    #[doc = "Bit 8 - SecureUser Execute"]
    #[inline(always)]
    pub fn sux(&mut self) -> SuxW<Mbc0MemnGlbac5Spec> {
        SuxW::new(self, 8)
    }
    #[doc = "Bit 9 - SecureUser Write"]
    #[inline(always)]
    pub fn suw(&mut self) -> SuwW<Mbc0MemnGlbac5Spec> {
        SuwW::new(self, 9)
    }
    #[doc = "Bit 10 - SecureUser Read"]
    #[inline(always)]
    pub fn sur(&mut self) -> SurW<Mbc0MemnGlbac5Spec> {
        SurW::new(self, 10)
    }
    #[doc = "Bit 12 - SecurePriv Execute"]
    #[inline(always)]
    pub fn spx(&mut self) -> SpxW<Mbc0MemnGlbac5Spec> {
        SpxW::new(self, 12)
    }
    #[doc = "Bit 13 - SecurePriv Write"]
    #[inline(always)]
    pub fn spw(&mut self) -> SpwW<Mbc0MemnGlbac5Spec> {
        SpwW::new(self, 13)
    }
    #[doc = "Bit 14 - SecurePriv Read"]
    #[inline(always)]
    pub fn spr(&mut self) -> SprW<Mbc0MemnGlbac5Spec> {
        SprW::new(self, 14)
    }
    #[doc = "Bit 31 - LOCK"]
    #[inline(always)]
    pub fn lk(&mut self) -> LkW<Mbc0MemnGlbac5Spec> {
        LkW::new(self, 31)
    }
}
#[doc = "MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac5::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac5::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mbc0MemnGlbac5Spec;
impl crate::RegisterSpec for Mbc0MemnGlbac5Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mbc0_memn_glbac5::R`](R) reader structure"]
impl crate::Readable for Mbc0MemnGlbac5Spec {}
#[doc = "`write(|w| ..)` method takes [`mbc0_memn_glbac5::W`](W) writer structure"]
impl crate::Writable for Mbc0MemnGlbac5Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MBC0_MEMN_GLBAC5 to value 0x1100"]
impl crate::Resettable for Mbc0MemnGlbac5Spec {
    const RESET_VALUE: u32 = 0x1100;
}
