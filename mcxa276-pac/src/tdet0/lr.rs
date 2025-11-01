#[doc = "Register `LR` reader"]
pub type R = crate::R<LrSpec>;
#[doc = "Register `LR` writer"]
pub type W = crate::W<LrSpec>;
#[doc = "Control Register Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crl {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Crl> for bool {
    #[inline(always)]
    fn from(variant: Crl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CRL` reader - Control Register Lock"]
pub type CrlR = crate::BitReader<Crl>;
impl CrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Crl {
        match self.bits {
            false => Crl::Lock,
            true => Crl::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Crl::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Crl::NotLock
    }
}
#[doc = "Field `CRL` writer - Control Register Lock"]
pub type CrlW<'a, REG> = crate::BitWriter<'a, REG, Crl>;
impl<'a, REG> CrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Crl::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Crl::NotLock)
    }
}
#[doc = "Status Register Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Srl {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Srl> for bool {
    #[inline(always)]
    fn from(variant: Srl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SRL` reader - Status Register Lock"]
pub type SrlR = crate::BitReader<Srl>;
impl SrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Srl {
        match self.bits {
            false => Srl::Lock,
            true => Srl::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Srl::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Srl::NotLock
    }
}
#[doc = "Field `SRL` writer - Status Register Lock"]
pub type SrlW<'a, REG> = crate::BitWriter<'a, REG, Srl>;
impl<'a, REG> SrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Srl::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Srl::NotLock)
    }
}
#[doc = "Lock Register Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lrl {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Lrl> for bool {
    #[inline(always)]
    fn from(variant: Lrl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LRL` reader - Lock Register Lock"]
pub type LrlR = crate::BitReader<Lrl>;
impl LrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lrl {
        match self.bits {
            false => Lrl::Lock,
            true => Lrl::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Lrl::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Lrl::NotLock
    }
}
#[doc = "Field `LRL` writer - Lock Register Lock"]
pub type LrlW<'a, REG> = crate::BitWriter<'a, REG, Lrl>;
impl<'a, REG> LrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Lrl::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Lrl::NotLock)
    }
}
#[doc = "Interrupt Enable Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Iel {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Iel> for bool {
    #[inline(always)]
    fn from(variant: Iel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IEL` reader - Interrupt Enable Lock"]
pub type IelR = crate::BitReader<Iel>;
impl IelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Iel {
        match self.bits {
            false => Iel::Lock,
            true => Iel::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Iel::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Iel::NotLock
    }
}
#[doc = "Field `IEL` writer - Interrupt Enable Lock"]
pub type IelW<'a, REG> = crate::BitWriter<'a, REG, Iel>;
impl<'a, REG> IelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Iel::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Iel::NotLock)
    }
}
#[doc = "Tamper Seconds Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tsl {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Tsl> for bool {
    #[inline(always)]
    fn from(variant: Tsl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TSL` reader - Tamper Seconds Lock"]
pub type TslR = crate::BitReader<Tsl>;
impl TslR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tsl {
        match self.bits {
            false => Tsl::Lock,
            true => Tsl::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Tsl::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Tsl::NotLock
    }
}
#[doc = "Field `TSL` writer - Tamper Seconds Lock"]
pub type TslW<'a, REG> = crate::BitWriter<'a, REG, Tsl>;
impl<'a, REG> TslW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Tsl::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Tsl::NotLock)
    }
}
#[doc = "Tamper Enable Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tel {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Tel> for bool {
    #[inline(always)]
    fn from(variant: Tel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TEL` reader - Tamper Enable Lock"]
pub type TelR = crate::BitReader<Tel>;
impl TelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tel {
        match self.bits {
            false => Tel::Lock,
            true => Tel::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Tel::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Tel::NotLock
    }
}
#[doc = "Field `TEL` writer - Tamper Enable Lock"]
pub type TelW<'a, REG> = crate::BitWriter<'a, REG, Tel>;
impl<'a, REG> TelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Tel::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Tel::NotLock)
    }
}
#[doc = "Pin Polarity Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ppl {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Ppl> for bool {
    #[inline(always)]
    fn from(variant: Ppl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PPL` reader - Pin Polarity Lock"]
pub type PplR = crate::BitReader<Ppl>;
impl PplR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ppl {
        match self.bits {
            false => Ppl::Lock,
            true => Ppl::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Ppl::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Ppl::NotLock
    }
}
#[doc = "Field `PPL` writer - Pin Polarity Lock"]
pub type PplW<'a, REG> = crate::BitWriter<'a, REG, Ppl>;
impl<'a, REG> PplW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Ppl::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Ppl::NotLock)
    }
}
#[doc = "Glitch Filter Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfl0 {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Gfl0> for bool {
    #[inline(always)]
    fn from(variant: Gfl0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFL0` reader - Glitch Filter Lock"]
pub type Gfl0R = crate::BitReader<Gfl0>;
impl Gfl0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfl0 {
        match self.bits {
            false => Gfl0::Lock,
            true => Gfl0::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Gfl0::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Gfl0::NotLock
    }
}
#[doc = "Field `GFL0` writer - Glitch Filter Lock"]
pub type Gfl0W<'a, REG> = crate::BitWriter<'a, REG, Gfl0>;
impl<'a, REG> Gfl0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl0::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl0::NotLock)
    }
}
#[doc = "Glitch Filter Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfl1 {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Gfl1> for bool {
    #[inline(always)]
    fn from(variant: Gfl1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFL1` reader - Glitch Filter Lock"]
pub type Gfl1R = crate::BitReader<Gfl1>;
impl Gfl1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfl1 {
        match self.bits {
            false => Gfl1::Lock,
            true => Gfl1::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Gfl1::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Gfl1::NotLock
    }
}
#[doc = "Field `GFL1` writer - Glitch Filter Lock"]
pub type Gfl1W<'a, REG> = crate::BitWriter<'a, REG, Gfl1>;
impl<'a, REG> Gfl1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl1::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl1::NotLock)
    }
}
#[doc = "Glitch Filter Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfl2 {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Gfl2> for bool {
    #[inline(always)]
    fn from(variant: Gfl2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFL2` reader - Glitch Filter Lock"]
pub type Gfl2R = crate::BitReader<Gfl2>;
impl Gfl2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfl2 {
        match self.bits {
            false => Gfl2::Lock,
            true => Gfl2::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Gfl2::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Gfl2::NotLock
    }
}
#[doc = "Field `GFL2` writer - Glitch Filter Lock"]
pub type Gfl2W<'a, REG> = crate::BitWriter<'a, REG, Gfl2>;
impl<'a, REG> Gfl2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl2::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl2::NotLock)
    }
}
#[doc = "Glitch Filter Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfl3 {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Gfl3> for bool {
    #[inline(always)]
    fn from(variant: Gfl3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFL3` reader - Glitch Filter Lock"]
pub type Gfl3R = crate::BitReader<Gfl3>;
impl Gfl3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfl3 {
        match self.bits {
            false => Gfl3::Lock,
            true => Gfl3::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Gfl3::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Gfl3::NotLock
    }
}
#[doc = "Field `GFL3` writer - Glitch Filter Lock"]
pub type Gfl3W<'a, REG> = crate::BitWriter<'a, REG, Gfl3>;
impl<'a, REG> Gfl3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl3::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl3::NotLock)
    }
}
#[doc = "Glitch Filter Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfl4 {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Gfl4> for bool {
    #[inline(always)]
    fn from(variant: Gfl4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFL4` reader - Glitch Filter Lock"]
pub type Gfl4R = crate::BitReader<Gfl4>;
impl Gfl4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfl4 {
        match self.bits {
            false => Gfl4::Lock,
            true => Gfl4::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Gfl4::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Gfl4::NotLock
    }
}
#[doc = "Field `GFL4` writer - Glitch Filter Lock"]
pub type Gfl4W<'a, REG> = crate::BitWriter<'a, REG, Gfl4>;
impl<'a, REG> Gfl4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl4::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl4::NotLock)
    }
}
#[doc = "Glitch Filter Lock\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfl5 {
    #[doc = "0: Locked and writes are ignored"]
    Lock = 0,
    #[doc = "1: Not locked and writes complete as normal"]
    NotLock = 1,
}
impl From<Gfl5> for bool {
    #[inline(always)]
    fn from(variant: Gfl5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFL5` reader - Glitch Filter Lock"]
pub type Gfl5R = crate::BitReader<Gfl5>;
impl Gfl5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfl5 {
        match self.bits {
            false => Gfl5::Lock,
            true => Gfl5::NotLock,
        }
    }
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn is_lock(&self) -> bool {
        *self == Gfl5::Lock
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn is_not_lock(&self) -> bool {
        *self == Gfl5::NotLock
    }
}
#[doc = "Field `GFL5` writer - Glitch Filter Lock"]
pub type Gfl5W<'a, REG> = crate::BitWriter<'a, REG, Gfl5>;
impl<'a, REG> Gfl5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Locked and writes are ignored"]
    #[inline(always)]
    pub fn lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl5::Lock)
    }
    #[doc = "Not locked and writes complete as normal"]
    #[inline(always)]
    pub fn not_lock(self) -> &'a mut crate::W<REG> {
        self.variant(Gfl5::NotLock)
    }
}
impl R {
    #[doc = "Bit 4 - Control Register Lock"]
    #[inline(always)]
    pub fn crl(&self) -> CrlR {
        CrlR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Status Register Lock"]
    #[inline(always)]
    pub fn srl(&self) -> SrlR {
        SrlR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Lock Register Lock"]
    #[inline(always)]
    pub fn lrl(&self) -> LrlR {
        LrlR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Interrupt Enable Lock"]
    #[inline(always)]
    pub fn iel(&self) -> IelR {
        IelR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Tamper Seconds Lock"]
    #[inline(always)]
    pub fn tsl(&self) -> TslR {
        TslR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Tamper Enable Lock"]
    #[inline(always)]
    pub fn tel(&self) -> TelR {
        TelR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 11 - Pin Polarity Lock"]
    #[inline(always)]
    pub fn ppl(&self) -> PplR {
        PplR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 16 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl0(&self) -> Gfl0R {
        Gfl0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl1(&self) -> Gfl1R {
        Gfl1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl2(&self) -> Gfl2R {
        Gfl2R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl3(&self) -> Gfl3R {
        Gfl3R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl4(&self) -> Gfl4R {
        Gfl4R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl5(&self) -> Gfl5R {
        Gfl5R::new(((self.bits >> 21) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 4 - Control Register Lock"]
    #[inline(always)]
    pub fn crl(&mut self) -> CrlW<LrSpec> {
        CrlW::new(self, 4)
    }
    #[doc = "Bit 5 - Status Register Lock"]
    #[inline(always)]
    pub fn srl(&mut self) -> SrlW<LrSpec> {
        SrlW::new(self, 5)
    }
    #[doc = "Bit 6 - Lock Register Lock"]
    #[inline(always)]
    pub fn lrl(&mut self) -> LrlW<LrSpec> {
        LrlW::new(self, 6)
    }
    #[doc = "Bit 7 - Interrupt Enable Lock"]
    #[inline(always)]
    pub fn iel(&mut self) -> IelW<LrSpec> {
        IelW::new(self, 7)
    }
    #[doc = "Bit 8 - Tamper Seconds Lock"]
    #[inline(always)]
    pub fn tsl(&mut self) -> TslW<LrSpec> {
        TslW::new(self, 8)
    }
    #[doc = "Bit 9 - Tamper Enable Lock"]
    #[inline(always)]
    pub fn tel(&mut self) -> TelW<LrSpec> {
        TelW::new(self, 9)
    }
    #[doc = "Bit 11 - Pin Polarity Lock"]
    #[inline(always)]
    pub fn ppl(&mut self) -> PplW<LrSpec> {
        PplW::new(self, 11)
    }
    #[doc = "Bit 16 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl0(&mut self) -> Gfl0W<LrSpec> {
        Gfl0W::new(self, 16)
    }
    #[doc = "Bit 17 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl1(&mut self) -> Gfl1W<LrSpec> {
        Gfl1W::new(self, 17)
    }
    #[doc = "Bit 18 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl2(&mut self) -> Gfl2W<LrSpec> {
        Gfl2W::new(self, 18)
    }
    #[doc = "Bit 19 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl3(&mut self) -> Gfl3W<LrSpec> {
        Gfl3W::new(self, 19)
    }
    #[doc = "Bit 20 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl4(&mut self) -> Gfl4W<LrSpec> {
        Gfl4W::new(self, 20)
    }
    #[doc = "Bit 21 - Glitch Filter Lock"]
    #[inline(always)]
    pub fn gfl5(&mut self) -> Gfl5W<LrSpec> {
        Gfl5W::new(self, 21)
    }
}
#[doc = "Lock\n\nYou can [`read`](crate::Reg::read) this register and get [`lr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LrSpec;
impl crate::RegisterSpec for LrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lr::R`](R) reader structure"]
impl crate::Readable for LrSpec {}
#[doc = "`write(|w| ..)` method takes [`lr::W`](W) writer structure"]
impl crate::Writable for LrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LR to value 0x003f_0bf0"]
impl crate::Resettable for LrSpec {
    const RESET_VALUE: u32 = 0x003f_0bf0;
}
