#[doc = "Register `PCOR` reader"]
pub type R = crate::R<PcorSpec>;
#[doc = "Register `PCOR` writer"]
pub type W = crate::W<PcorSpec>;
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco0 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco0> for bool {
    #[inline(always)]
    fn from(variant: Ptco0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO0` reader - Port Clear Output"]
pub type Ptco0R = crate::BitReader<Ptco0>;
impl Ptco0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco0 {
        match self.bits {
            false => Ptco0::Ptco0,
            true => Ptco0::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco0::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco0::Ptco1
    }
}
#[doc = "Field `PTCO0` writer - Port Clear Output"]
pub type Ptco0W<'a, REG> = crate::BitWriter<'a, REG, Ptco0>;
impl<'a, REG> Ptco0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco0::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco0::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco1 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco1> for bool {
    #[inline(always)]
    fn from(variant: Ptco1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO1` reader - Port Clear Output"]
pub type Ptco1R = crate::BitReader<Ptco1>;
impl Ptco1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco1 {
        match self.bits {
            false => Ptco1::Ptco0,
            true => Ptco1::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco1::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco1::Ptco1
    }
}
#[doc = "Field `PTCO1` writer - Port Clear Output"]
pub type Ptco1W<'a, REG> = crate::BitWriter<'a, REG, Ptco1>;
impl<'a, REG> Ptco1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco1::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco1::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco2 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco2> for bool {
    #[inline(always)]
    fn from(variant: Ptco2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO2` reader - Port Clear Output"]
pub type Ptco2R = crate::BitReader<Ptco2>;
impl Ptco2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco2 {
        match self.bits {
            false => Ptco2::Ptco0,
            true => Ptco2::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco2::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco2::Ptco1
    }
}
#[doc = "Field `PTCO2` writer - Port Clear Output"]
pub type Ptco2W<'a, REG> = crate::BitWriter<'a, REG, Ptco2>;
impl<'a, REG> Ptco2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco2::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco2::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco3 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco3> for bool {
    #[inline(always)]
    fn from(variant: Ptco3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO3` reader - Port Clear Output"]
pub type Ptco3R = crate::BitReader<Ptco3>;
impl Ptco3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco3 {
        match self.bits {
            false => Ptco3::Ptco0,
            true => Ptco3::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco3::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco3::Ptco1
    }
}
#[doc = "Field `PTCO3` writer - Port Clear Output"]
pub type Ptco3W<'a, REG> = crate::BitWriter<'a, REG, Ptco3>;
impl<'a, REG> Ptco3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco3::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco3::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco4 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco4> for bool {
    #[inline(always)]
    fn from(variant: Ptco4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO4` reader - Port Clear Output"]
pub type Ptco4R = crate::BitReader<Ptco4>;
impl Ptco4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco4 {
        match self.bits {
            false => Ptco4::Ptco0,
            true => Ptco4::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco4::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco4::Ptco1
    }
}
#[doc = "Field `PTCO4` writer - Port Clear Output"]
pub type Ptco4W<'a, REG> = crate::BitWriter<'a, REG, Ptco4>;
impl<'a, REG> Ptco4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco4::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco4::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco5 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco5> for bool {
    #[inline(always)]
    fn from(variant: Ptco5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO5` reader - Port Clear Output"]
pub type Ptco5R = crate::BitReader<Ptco5>;
impl Ptco5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco5 {
        match self.bits {
            false => Ptco5::Ptco0,
            true => Ptco5::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco5::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco5::Ptco1
    }
}
#[doc = "Field `PTCO5` writer - Port Clear Output"]
pub type Ptco5W<'a, REG> = crate::BitWriter<'a, REG, Ptco5>;
impl<'a, REG> Ptco5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco5::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco5::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco6 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco6> for bool {
    #[inline(always)]
    fn from(variant: Ptco6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO6` reader - Port Clear Output"]
pub type Ptco6R = crate::BitReader<Ptco6>;
impl Ptco6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco6 {
        match self.bits {
            false => Ptco6::Ptco0,
            true => Ptco6::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco6::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco6::Ptco1
    }
}
#[doc = "Field `PTCO6` writer - Port Clear Output"]
pub type Ptco6W<'a, REG> = crate::BitWriter<'a, REG, Ptco6>;
impl<'a, REG> Ptco6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco6::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco6::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco7 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco7> for bool {
    #[inline(always)]
    fn from(variant: Ptco7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO7` reader - Port Clear Output"]
pub type Ptco7R = crate::BitReader<Ptco7>;
impl Ptco7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco7 {
        match self.bits {
            false => Ptco7::Ptco0,
            true => Ptco7::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco7::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco7::Ptco1
    }
}
#[doc = "Field `PTCO7` writer - Port Clear Output"]
pub type Ptco7W<'a, REG> = crate::BitWriter<'a, REG, Ptco7>;
impl<'a, REG> Ptco7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco7::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco7::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco8 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco8> for bool {
    #[inline(always)]
    fn from(variant: Ptco8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO8` reader - Port Clear Output"]
pub type Ptco8R = crate::BitReader<Ptco8>;
impl Ptco8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco8 {
        match self.bits {
            false => Ptco8::Ptco0,
            true => Ptco8::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco8::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco8::Ptco1
    }
}
#[doc = "Field `PTCO8` writer - Port Clear Output"]
pub type Ptco8W<'a, REG> = crate::BitWriter<'a, REG, Ptco8>;
impl<'a, REG> Ptco8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco8::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco8::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco9 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco9> for bool {
    #[inline(always)]
    fn from(variant: Ptco9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO9` reader - Port Clear Output"]
pub type Ptco9R = crate::BitReader<Ptco9>;
impl Ptco9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco9 {
        match self.bits {
            false => Ptco9::Ptco0,
            true => Ptco9::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco9::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco9::Ptco1
    }
}
#[doc = "Field `PTCO9` writer - Port Clear Output"]
pub type Ptco9W<'a, REG> = crate::BitWriter<'a, REG, Ptco9>;
impl<'a, REG> Ptco9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco9::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco9::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco10 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco10> for bool {
    #[inline(always)]
    fn from(variant: Ptco10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO10` reader - Port Clear Output"]
pub type Ptco10R = crate::BitReader<Ptco10>;
impl Ptco10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco10 {
        match self.bits {
            false => Ptco10::Ptco0,
            true => Ptco10::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco10::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco10::Ptco1
    }
}
#[doc = "Field `PTCO10` writer - Port Clear Output"]
pub type Ptco10W<'a, REG> = crate::BitWriter<'a, REG, Ptco10>;
impl<'a, REG> Ptco10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco10::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco10::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco11 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco11> for bool {
    #[inline(always)]
    fn from(variant: Ptco11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO11` reader - Port Clear Output"]
pub type Ptco11R = crate::BitReader<Ptco11>;
impl Ptco11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco11 {
        match self.bits {
            false => Ptco11::Ptco0,
            true => Ptco11::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco11::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco11::Ptco1
    }
}
#[doc = "Field `PTCO11` writer - Port Clear Output"]
pub type Ptco11W<'a, REG> = crate::BitWriter<'a, REG, Ptco11>;
impl<'a, REG> Ptco11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco11::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco11::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco12 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco12> for bool {
    #[inline(always)]
    fn from(variant: Ptco12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO12` reader - Port Clear Output"]
pub type Ptco12R = crate::BitReader<Ptco12>;
impl Ptco12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco12 {
        match self.bits {
            false => Ptco12::Ptco0,
            true => Ptco12::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco12::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco12::Ptco1
    }
}
#[doc = "Field `PTCO12` writer - Port Clear Output"]
pub type Ptco12W<'a, REG> = crate::BitWriter<'a, REG, Ptco12>;
impl<'a, REG> Ptco12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco12::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco12::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco13 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco13> for bool {
    #[inline(always)]
    fn from(variant: Ptco13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO13` reader - Port Clear Output"]
pub type Ptco13R = crate::BitReader<Ptco13>;
impl Ptco13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco13 {
        match self.bits {
            false => Ptco13::Ptco0,
            true => Ptco13::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco13::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco13::Ptco1
    }
}
#[doc = "Field `PTCO13` writer - Port Clear Output"]
pub type Ptco13W<'a, REG> = crate::BitWriter<'a, REG, Ptco13>;
impl<'a, REG> Ptco13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco13::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco13::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco14 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco14> for bool {
    #[inline(always)]
    fn from(variant: Ptco14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO14` reader - Port Clear Output"]
pub type Ptco14R = crate::BitReader<Ptco14>;
impl Ptco14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco14 {
        match self.bits {
            false => Ptco14::Ptco0,
            true => Ptco14::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco14::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco14::Ptco1
    }
}
#[doc = "Field `PTCO14` writer - Port Clear Output"]
pub type Ptco14W<'a, REG> = crate::BitWriter<'a, REG, Ptco14>;
impl<'a, REG> Ptco14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco14::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco14::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco15 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco15> for bool {
    #[inline(always)]
    fn from(variant: Ptco15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO15` reader - Port Clear Output"]
pub type Ptco15R = crate::BitReader<Ptco15>;
impl Ptco15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco15 {
        match self.bits {
            false => Ptco15::Ptco0,
            true => Ptco15::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco15::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco15::Ptco1
    }
}
#[doc = "Field `PTCO15` writer - Port Clear Output"]
pub type Ptco15W<'a, REG> = crate::BitWriter<'a, REG, Ptco15>;
impl<'a, REG> Ptco15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco15::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco15::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco16 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco16> for bool {
    #[inline(always)]
    fn from(variant: Ptco16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO16` reader - Port Clear Output"]
pub type Ptco16R = crate::BitReader<Ptco16>;
impl Ptco16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco16 {
        match self.bits {
            false => Ptco16::Ptco0,
            true => Ptco16::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco16::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco16::Ptco1
    }
}
#[doc = "Field `PTCO16` writer - Port Clear Output"]
pub type Ptco16W<'a, REG> = crate::BitWriter<'a, REG, Ptco16>;
impl<'a, REG> Ptco16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco16::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco16::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco17 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco17> for bool {
    #[inline(always)]
    fn from(variant: Ptco17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO17` reader - Port Clear Output"]
pub type Ptco17R = crate::BitReader<Ptco17>;
impl Ptco17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco17 {
        match self.bits {
            false => Ptco17::Ptco0,
            true => Ptco17::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco17::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco17::Ptco1
    }
}
#[doc = "Field `PTCO17` writer - Port Clear Output"]
pub type Ptco17W<'a, REG> = crate::BitWriter<'a, REG, Ptco17>;
impl<'a, REG> Ptco17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco17::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco17::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco18 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco18> for bool {
    #[inline(always)]
    fn from(variant: Ptco18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO18` reader - Port Clear Output"]
pub type Ptco18R = crate::BitReader<Ptco18>;
impl Ptco18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco18 {
        match self.bits {
            false => Ptco18::Ptco0,
            true => Ptco18::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco18::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco18::Ptco1
    }
}
#[doc = "Field `PTCO18` writer - Port Clear Output"]
pub type Ptco18W<'a, REG> = crate::BitWriter<'a, REG, Ptco18>;
impl<'a, REG> Ptco18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco18::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco18::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco19 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco19> for bool {
    #[inline(always)]
    fn from(variant: Ptco19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO19` reader - Port Clear Output"]
pub type Ptco19R = crate::BitReader<Ptco19>;
impl Ptco19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco19 {
        match self.bits {
            false => Ptco19::Ptco0,
            true => Ptco19::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco19::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco19::Ptco1
    }
}
#[doc = "Field `PTCO19` writer - Port Clear Output"]
pub type Ptco19W<'a, REG> = crate::BitWriter<'a, REG, Ptco19>;
impl<'a, REG> Ptco19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco19::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco19::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco20 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco20> for bool {
    #[inline(always)]
    fn from(variant: Ptco20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO20` reader - Port Clear Output"]
pub type Ptco20R = crate::BitReader<Ptco20>;
impl Ptco20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco20 {
        match self.bits {
            false => Ptco20::Ptco0,
            true => Ptco20::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco20::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco20::Ptco1
    }
}
#[doc = "Field `PTCO20` writer - Port Clear Output"]
pub type Ptco20W<'a, REG> = crate::BitWriter<'a, REG, Ptco20>;
impl<'a, REG> Ptco20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco20::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco20::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco21 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco21> for bool {
    #[inline(always)]
    fn from(variant: Ptco21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO21` reader - Port Clear Output"]
pub type Ptco21R = crate::BitReader<Ptco21>;
impl Ptco21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco21 {
        match self.bits {
            false => Ptco21::Ptco0,
            true => Ptco21::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco21::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco21::Ptco1
    }
}
#[doc = "Field `PTCO21` writer - Port Clear Output"]
pub type Ptco21W<'a, REG> = crate::BitWriter<'a, REG, Ptco21>;
impl<'a, REG> Ptco21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco21::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco21::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco22 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco22> for bool {
    #[inline(always)]
    fn from(variant: Ptco22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO22` reader - Port Clear Output"]
pub type Ptco22R = crate::BitReader<Ptco22>;
impl Ptco22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco22 {
        match self.bits {
            false => Ptco22::Ptco0,
            true => Ptco22::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco22::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco22::Ptco1
    }
}
#[doc = "Field `PTCO22` writer - Port Clear Output"]
pub type Ptco22W<'a, REG> = crate::BitWriter<'a, REG, Ptco22>;
impl<'a, REG> Ptco22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco22::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco22::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco23 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco23> for bool {
    #[inline(always)]
    fn from(variant: Ptco23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO23` reader - Port Clear Output"]
pub type Ptco23R = crate::BitReader<Ptco23>;
impl Ptco23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco23 {
        match self.bits {
            false => Ptco23::Ptco0,
            true => Ptco23::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco23::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco23::Ptco1
    }
}
#[doc = "Field `PTCO23` writer - Port Clear Output"]
pub type Ptco23W<'a, REG> = crate::BitWriter<'a, REG, Ptco23>;
impl<'a, REG> Ptco23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco23::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco23::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco24 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco24> for bool {
    #[inline(always)]
    fn from(variant: Ptco24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO24` reader - Port Clear Output"]
pub type Ptco24R = crate::BitReader<Ptco24>;
impl Ptco24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco24 {
        match self.bits {
            false => Ptco24::Ptco0,
            true => Ptco24::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco24::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco24::Ptco1
    }
}
#[doc = "Field `PTCO24` writer - Port Clear Output"]
pub type Ptco24W<'a, REG> = crate::BitWriter<'a, REG, Ptco24>;
impl<'a, REG> Ptco24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco24::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco24::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco25 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco25> for bool {
    #[inline(always)]
    fn from(variant: Ptco25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO25` reader - Port Clear Output"]
pub type Ptco25R = crate::BitReader<Ptco25>;
impl Ptco25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco25 {
        match self.bits {
            false => Ptco25::Ptco0,
            true => Ptco25::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco25::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco25::Ptco1
    }
}
#[doc = "Field `PTCO25` writer - Port Clear Output"]
pub type Ptco25W<'a, REG> = crate::BitWriter<'a, REG, Ptco25>;
impl<'a, REG> Ptco25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco25::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco25::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco26 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco26> for bool {
    #[inline(always)]
    fn from(variant: Ptco26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO26` reader - Port Clear Output"]
pub type Ptco26R = crate::BitReader<Ptco26>;
impl Ptco26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco26 {
        match self.bits {
            false => Ptco26::Ptco0,
            true => Ptco26::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco26::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco26::Ptco1
    }
}
#[doc = "Field `PTCO26` writer - Port Clear Output"]
pub type Ptco26W<'a, REG> = crate::BitWriter<'a, REG, Ptco26>;
impl<'a, REG> Ptco26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco26::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco26::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco27 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco27> for bool {
    #[inline(always)]
    fn from(variant: Ptco27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO27` reader - Port Clear Output"]
pub type Ptco27R = crate::BitReader<Ptco27>;
impl Ptco27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco27 {
        match self.bits {
            false => Ptco27::Ptco0,
            true => Ptco27::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco27::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco27::Ptco1
    }
}
#[doc = "Field `PTCO27` writer - Port Clear Output"]
pub type Ptco27W<'a, REG> = crate::BitWriter<'a, REG, Ptco27>;
impl<'a, REG> Ptco27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco27::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco27::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco28 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco28> for bool {
    #[inline(always)]
    fn from(variant: Ptco28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO28` reader - Port Clear Output"]
pub type Ptco28R = crate::BitReader<Ptco28>;
impl Ptco28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco28 {
        match self.bits {
            false => Ptco28::Ptco0,
            true => Ptco28::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco28::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco28::Ptco1
    }
}
#[doc = "Field `PTCO28` writer - Port Clear Output"]
pub type Ptco28W<'a, REG> = crate::BitWriter<'a, REG, Ptco28>;
impl<'a, REG> Ptco28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco28::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco28::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco29 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco29> for bool {
    #[inline(always)]
    fn from(variant: Ptco29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO29` reader - Port Clear Output"]
pub type Ptco29R = crate::BitReader<Ptco29>;
impl Ptco29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco29 {
        match self.bits {
            false => Ptco29::Ptco0,
            true => Ptco29::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco29::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco29::Ptco1
    }
}
#[doc = "Field `PTCO29` writer - Port Clear Output"]
pub type Ptco29W<'a, REG> = crate::BitWriter<'a, REG, Ptco29>;
impl<'a, REG> Ptco29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco29::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco29::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco30 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco30> for bool {
    #[inline(always)]
    fn from(variant: Ptco30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO30` reader - Port Clear Output"]
pub type Ptco30R = crate::BitReader<Ptco30>;
impl Ptco30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco30 {
        match self.bits {
            false => Ptco30::Ptco0,
            true => Ptco30::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco30::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco30::Ptco1
    }
}
#[doc = "Field `PTCO30` writer - Port Clear Output"]
pub type Ptco30W<'a, REG> = crate::BitWriter<'a, REG, Ptco30>;
impl<'a, REG> Ptco30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco30::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco30::Ptco1)
    }
}
#[doc = "Port Clear Output\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptco31 {
    #[doc = "0: No change"]
    Ptco0 = 0,
    #[doc = "1: Corresponding field in PDOR becomes 0"]
    Ptco1 = 1,
}
impl From<Ptco31> for bool {
    #[inline(always)]
    fn from(variant: Ptco31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTCO31` reader - Port Clear Output"]
pub type Ptco31R = crate::BitReader<Ptco31>;
impl Ptco31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptco31 {
        match self.bits {
            false => Ptco31::Ptco0,
            true => Ptco31::Ptco1,
        }
    }
    #[doc = "No change"]
    #[inline(always)]
    pub fn is_ptco0(&self) -> bool {
        *self == Ptco31::Ptco0
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn is_ptco1(&self) -> bool {
        *self == Ptco31::Ptco1
    }
}
#[doc = "Field `PTCO31` writer - Port Clear Output"]
pub type Ptco31W<'a, REG> = crate::BitWriter<'a, REG, Ptco31>;
impl<'a, REG> Ptco31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No change"]
    #[inline(always)]
    pub fn ptco0(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco31::Ptco0)
    }
    #[doc = "Corresponding field in PDOR becomes 0"]
    #[inline(always)]
    pub fn ptco1(self) -> &'a mut crate::W<REG> {
        self.variant(Ptco31::Ptco1)
    }
}
impl R {
    #[doc = "Bit 0 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco0(&self) -> Ptco0R {
        Ptco0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco1(&self) -> Ptco1R {
        Ptco1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco2(&self) -> Ptco2R {
        Ptco2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco3(&self) -> Ptco3R {
        Ptco3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco4(&self) -> Ptco4R {
        Ptco4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco5(&self) -> Ptco5R {
        Ptco5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco6(&self) -> Ptco6R {
        Ptco6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco7(&self) -> Ptco7R {
        Ptco7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco8(&self) -> Ptco8R {
        Ptco8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco9(&self) -> Ptco9R {
        Ptco9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco10(&self) -> Ptco10R {
        Ptco10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco11(&self) -> Ptco11R {
        Ptco11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco12(&self) -> Ptco12R {
        Ptco12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco13(&self) -> Ptco13R {
        Ptco13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco14(&self) -> Ptco14R {
        Ptco14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco15(&self) -> Ptco15R {
        Ptco15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco16(&self) -> Ptco16R {
        Ptco16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco17(&self) -> Ptco17R {
        Ptco17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco18(&self) -> Ptco18R {
        Ptco18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco19(&self) -> Ptco19R {
        Ptco19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco20(&self) -> Ptco20R {
        Ptco20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco21(&self) -> Ptco21R {
        Ptco21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco22(&self) -> Ptco22R {
        Ptco22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco23(&self) -> Ptco23R {
        Ptco23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco24(&self) -> Ptco24R {
        Ptco24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco25(&self) -> Ptco25R {
        Ptco25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco26(&self) -> Ptco26R {
        Ptco26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco27(&self) -> Ptco27R {
        Ptco27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco28(&self) -> Ptco28R {
        Ptco28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco29(&self) -> Ptco29R {
        Ptco29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco30(&self) -> Ptco30R {
        Ptco30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco31(&self) -> Ptco31R {
        Ptco31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco0(&mut self) -> Ptco0W<PcorSpec> {
        Ptco0W::new(self, 0)
    }
    #[doc = "Bit 1 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco1(&mut self) -> Ptco1W<PcorSpec> {
        Ptco1W::new(self, 1)
    }
    #[doc = "Bit 2 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco2(&mut self) -> Ptco2W<PcorSpec> {
        Ptco2W::new(self, 2)
    }
    #[doc = "Bit 3 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco3(&mut self) -> Ptco3W<PcorSpec> {
        Ptco3W::new(self, 3)
    }
    #[doc = "Bit 4 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco4(&mut self) -> Ptco4W<PcorSpec> {
        Ptco4W::new(self, 4)
    }
    #[doc = "Bit 5 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco5(&mut self) -> Ptco5W<PcorSpec> {
        Ptco5W::new(self, 5)
    }
    #[doc = "Bit 6 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco6(&mut self) -> Ptco6W<PcorSpec> {
        Ptco6W::new(self, 6)
    }
    #[doc = "Bit 7 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco7(&mut self) -> Ptco7W<PcorSpec> {
        Ptco7W::new(self, 7)
    }
    #[doc = "Bit 8 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco8(&mut self) -> Ptco8W<PcorSpec> {
        Ptco8W::new(self, 8)
    }
    #[doc = "Bit 9 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco9(&mut self) -> Ptco9W<PcorSpec> {
        Ptco9W::new(self, 9)
    }
    #[doc = "Bit 10 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco10(&mut self) -> Ptco10W<PcorSpec> {
        Ptco10W::new(self, 10)
    }
    #[doc = "Bit 11 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco11(&mut self) -> Ptco11W<PcorSpec> {
        Ptco11W::new(self, 11)
    }
    #[doc = "Bit 12 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco12(&mut self) -> Ptco12W<PcorSpec> {
        Ptco12W::new(self, 12)
    }
    #[doc = "Bit 13 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco13(&mut self) -> Ptco13W<PcorSpec> {
        Ptco13W::new(self, 13)
    }
    #[doc = "Bit 14 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco14(&mut self) -> Ptco14W<PcorSpec> {
        Ptco14W::new(self, 14)
    }
    #[doc = "Bit 15 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco15(&mut self) -> Ptco15W<PcorSpec> {
        Ptco15W::new(self, 15)
    }
    #[doc = "Bit 16 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco16(&mut self) -> Ptco16W<PcorSpec> {
        Ptco16W::new(self, 16)
    }
    #[doc = "Bit 17 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco17(&mut self) -> Ptco17W<PcorSpec> {
        Ptco17W::new(self, 17)
    }
    #[doc = "Bit 18 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco18(&mut self) -> Ptco18W<PcorSpec> {
        Ptco18W::new(self, 18)
    }
    #[doc = "Bit 19 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco19(&mut self) -> Ptco19W<PcorSpec> {
        Ptco19W::new(self, 19)
    }
    #[doc = "Bit 20 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco20(&mut self) -> Ptco20W<PcorSpec> {
        Ptco20W::new(self, 20)
    }
    #[doc = "Bit 21 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco21(&mut self) -> Ptco21W<PcorSpec> {
        Ptco21W::new(self, 21)
    }
    #[doc = "Bit 22 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco22(&mut self) -> Ptco22W<PcorSpec> {
        Ptco22W::new(self, 22)
    }
    #[doc = "Bit 23 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco23(&mut self) -> Ptco23W<PcorSpec> {
        Ptco23W::new(self, 23)
    }
    #[doc = "Bit 24 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco24(&mut self) -> Ptco24W<PcorSpec> {
        Ptco24W::new(self, 24)
    }
    #[doc = "Bit 25 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco25(&mut self) -> Ptco25W<PcorSpec> {
        Ptco25W::new(self, 25)
    }
    #[doc = "Bit 26 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco26(&mut self) -> Ptco26W<PcorSpec> {
        Ptco26W::new(self, 26)
    }
    #[doc = "Bit 27 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco27(&mut self) -> Ptco27W<PcorSpec> {
        Ptco27W::new(self, 27)
    }
    #[doc = "Bit 28 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco28(&mut self) -> Ptco28W<PcorSpec> {
        Ptco28W::new(self, 28)
    }
    #[doc = "Bit 29 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco29(&mut self) -> Ptco29W<PcorSpec> {
        Ptco29W::new(self, 29)
    }
    #[doc = "Bit 30 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco30(&mut self) -> Ptco30W<PcorSpec> {
        Ptco30W::new(self, 30)
    }
    #[doc = "Bit 31 - Port Clear Output"]
    #[inline(always)]
    pub fn ptco31(&mut self) -> Ptco31W<PcorSpec> {
        Ptco31W::new(self, 31)
    }
}
#[doc = "Port Clear Output\n\nYou can [`read`](crate::Reg::read) this register and get [`pcor::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcor::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PcorSpec;
impl crate::RegisterSpec for PcorSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pcor::R`](R) reader structure"]
impl crate::Readable for PcorSpec {}
#[doc = "`write(|w| ..)` method takes [`pcor::W`](W) writer structure"]
impl crate::Writable for PcorSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PCOR to value 0"]
impl crate::Resettable for PcorSpec {}
