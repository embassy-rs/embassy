#[doc = "Register `SR` reader"]
pub type R = crate::R<SrSpec>;
#[doc = "Register `SR` writer"]
pub type W = crate::W<SrSpec>;
#[doc = "Digital Tamper Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dtf {
    #[doc = "0: TDET tampering not detected"]
    NoDet = 0,
    #[doc = "1: TDET tampering detected"]
    Det = 1,
}
impl From<Dtf> for bool {
    #[inline(always)]
    fn from(variant: Dtf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DTF` reader - Digital Tamper Flag"]
pub type DtfR = crate::BitReader<Dtf>;
impl DtfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dtf {
        match self.bits {
            false => Dtf::NoDet,
            true => Dtf::Det,
        }
    }
    #[doc = "TDET tampering not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Dtf::NoDet
    }
    #[doc = "TDET tampering detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Dtf::Det
    }
}
#[doc = "Field `DTF` writer - Digital Tamper Flag"]
pub type DtfW<'a, REG> = crate::BitWriter1C<'a, REG, Dtf>;
impl<'a, REG> DtfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "TDET tampering not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Dtf::NoDet)
    }
    #[doc = "TDET tampering detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Dtf::Det)
    }
}
#[doc = "Tamper Acknowledge Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taf {
    #[doc = "0: Digital Tamper Flag (SR\\[DTF\\]) is clear or chip reset has not occurred after Digital Tamper Flag (SR\\[DTF\\]) was set."]
    NotOccur = 0,
    #[doc = "1: Chip reset has occurred after Digital Tamper Flag (SR\\[DTF\\]) was set."]
    Occur = 1,
}
impl From<Taf> for bool {
    #[inline(always)]
    fn from(variant: Taf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TAF` reader - Tamper Acknowledge Flag"]
pub type TafR = crate::BitReader<Taf>;
impl TafR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Taf {
        match self.bits {
            false => Taf::NotOccur,
            true => Taf::Occur,
        }
    }
    #[doc = "Digital Tamper Flag (SR\\[DTF\\]) is clear or chip reset has not occurred after Digital Tamper Flag (SR\\[DTF\\]) was set."]
    #[inline(always)]
    pub fn is_not_occur(&self) -> bool {
        *self == Taf::NotOccur
    }
    #[doc = "Chip reset has occurred after Digital Tamper Flag (SR\\[DTF\\]) was set."]
    #[inline(always)]
    pub fn is_occur(&self) -> bool {
        *self == Taf::Occur
    }
}
#[doc = "Field `TAF` writer - Tamper Acknowledge Flag"]
pub type TafW<'a, REG> = crate::BitWriter1C<'a, REG, Taf>;
impl<'a, REG> TafW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Digital Tamper Flag (SR\\[DTF\\]) is clear or chip reset has not occurred after Digital Tamper Flag (SR\\[DTF\\]) was set."]
    #[inline(always)]
    pub fn not_occur(self) -> &'a mut crate::W<REG> {
        self.variant(Taf::NotOccur)
    }
    #[doc = "Chip reset has occurred after Digital Tamper Flag (SR\\[DTF\\]) was set."]
    #[inline(always)]
    pub fn occur(self) -> &'a mut crate::W<REG> {
        self.variant(Taf::Occur)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif0 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif0> for bool {
    #[inline(always)]
    fn from(variant: Tif0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF0` reader - Tamper Input n Flag"]
pub type Tif0R = crate::BitReader<Tif0>;
impl Tif0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif0 {
        match self.bits {
            false => Tif0::NoDet,
            true => Tif0::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif0::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif0::Det
    }
}
#[doc = "Field `TIF0` writer - Tamper Input n Flag"]
pub type Tif0W<'a, REG> = crate::BitWriter1C<'a, REG, Tif0>;
impl<'a, REG> Tif0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif0::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif0::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif1 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif1> for bool {
    #[inline(always)]
    fn from(variant: Tif1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF1` reader - Tamper Input n Flag"]
pub type Tif1R = crate::BitReader<Tif1>;
impl Tif1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif1 {
        match self.bits {
            false => Tif1::NoDet,
            true => Tif1::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif1::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif1::Det
    }
}
#[doc = "Field `TIF1` writer - Tamper Input n Flag"]
pub type Tif1W<'a, REG> = crate::BitWriter1C<'a, REG, Tif1>;
impl<'a, REG> Tif1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif1::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif1::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif2 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif2> for bool {
    #[inline(always)]
    fn from(variant: Tif2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF2` reader - Tamper Input n Flag"]
pub type Tif2R = crate::BitReader<Tif2>;
impl Tif2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif2 {
        match self.bits {
            false => Tif2::NoDet,
            true => Tif2::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif2::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif2::Det
    }
}
#[doc = "Field `TIF2` writer - Tamper Input n Flag"]
pub type Tif2W<'a, REG> = crate::BitWriter1C<'a, REG, Tif2>;
impl<'a, REG> Tif2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif2::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif2::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif3 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif3> for bool {
    #[inline(always)]
    fn from(variant: Tif3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF3` reader - Tamper Input n Flag"]
pub type Tif3R = crate::BitReader<Tif3>;
impl Tif3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif3 {
        match self.bits {
            false => Tif3::NoDet,
            true => Tif3::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif3::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif3::Det
    }
}
#[doc = "Field `TIF3` writer - Tamper Input n Flag"]
pub type Tif3W<'a, REG> = crate::BitWriter1C<'a, REG, Tif3>;
impl<'a, REG> Tif3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif3::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif3::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif4 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif4> for bool {
    #[inline(always)]
    fn from(variant: Tif4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF4` reader - Tamper Input n Flag"]
pub type Tif4R = crate::BitReader<Tif4>;
impl Tif4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif4 {
        match self.bits {
            false => Tif4::NoDet,
            true => Tif4::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif4::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif4::Det
    }
}
#[doc = "Field `TIF4` writer - Tamper Input n Flag"]
pub type Tif4W<'a, REG> = crate::BitWriter1C<'a, REG, Tif4>;
impl<'a, REG> Tif4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif4::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif4::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif5 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif5> for bool {
    #[inline(always)]
    fn from(variant: Tif5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF5` reader - Tamper Input n Flag"]
pub type Tif5R = crate::BitReader<Tif5>;
impl Tif5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif5 {
        match self.bits {
            false => Tif5::NoDet,
            true => Tif5::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif5::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif5::Det
    }
}
#[doc = "Field `TIF5` writer - Tamper Input n Flag"]
pub type Tif5W<'a, REG> = crate::BitWriter1C<'a, REG, Tif5>;
impl<'a, REG> Tif5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif5::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif5::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif6 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif6> for bool {
    #[inline(always)]
    fn from(variant: Tif6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF6` reader - Tamper Input n Flag"]
pub type Tif6R = crate::BitReader<Tif6>;
impl Tif6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif6 {
        match self.bits {
            false => Tif6::NoDet,
            true => Tif6::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif6::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif6::Det
    }
}
#[doc = "Field `TIF6` writer - Tamper Input n Flag"]
pub type Tif6W<'a, REG> = crate::BitWriter1C<'a, REG, Tif6>;
impl<'a, REG> Tif6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif6::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif6::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif7 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif7> for bool {
    #[inline(always)]
    fn from(variant: Tif7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF7` reader - Tamper Input n Flag"]
pub type Tif7R = crate::BitReader<Tif7>;
impl Tif7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif7 {
        match self.bits {
            false => Tif7::NoDet,
            true => Tif7::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif7::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif7::Det
    }
}
#[doc = "Field `TIF7` writer - Tamper Input n Flag"]
pub type Tif7W<'a, REG> = crate::BitWriter1C<'a, REG, Tif7>;
impl<'a, REG> Tif7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif7::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif7::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif8 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif8> for bool {
    #[inline(always)]
    fn from(variant: Tif8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF8` reader - Tamper Input n Flag"]
pub type Tif8R = crate::BitReader<Tif8>;
impl Tif8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif8 {
        match self.bits {
            false => Tif8::NoDet,
            true => Tif8::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif8::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif8::Det
    }
}
#[doc = "Field `TIF8` writer - Tamper Input n Flag"]
pub type Tif8W<'a, REG> = crate::BitWriter1C<'a, REG, Tif8>;
impl<'a, REG> Tif8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif8::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif8::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif9 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif9> for bool {
    #[inline(always)]
    fn from(variant: Tif9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF9` reader - Tamper Input n Flag"]
pub type Tif9R = crate::BitReader<Tif9>;
impl Tif9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif9 {
        match self.bits {
            false => Tif9::NoDet,
            true => Tif9::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif9::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif9::Det
    }
}
#[doc = "Field `TIF9` writer - Tamper Input n Flag"]
pub type Tif9W<'a, REG> = crate::BitWriter1C<'a, REG, Tif9>;
impl<'a, REG> Tif9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif9::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif9::Det)
    }
}
#[doc = "Tamper Input n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tif10 {
    #[doc = "0: On-chip tamper not detected"]
    NoDet = 0,
    #[doc = "1: On-chip tamper detected"]
    Det = 1,
}
impl From<Tif10> for bool {
    #[inline(always)]
    fn from(variant: Tif10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIF10` reader - Tamper Input n Flag"]
pub type Tif10R = crate::BitReader<Tif10>;
impl Tif10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tif10 {
        match self.bits {
            false => Tif10::NoDet,
            true => Tif10::Det,
        }
    }
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tif10::NoDet
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tif10::Det
    }
}
#[doc = "Field `TIF10` writer - Tamper Input n Flag"]
pub type Tif10W<'a, REG> = crate::BitWriter1C<'a, REG, Tif10>;
impl<'a, REG> Tif10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "On-chip tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif10::NoDet)
    }
    #[doc = "On-chip tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tif10::Det)
    }
}
#[doc = "Tamper Pin n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpf0 {
    #[doc = "0: Pin tamper not detected"]
    NoDet = 0,
    #[doc = "1: Pin tamper detected"]
    Det = 1,
}
impl From<Tpf0> for bool {
    #[inline(always)]
    fn from(variant: Tpf0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPF0` reader - Tamper Pin n Flag"]
pub type Tpf0R = crate::BitReader<Tpf0>;
impl Tpf0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpf0 {
        match self.bits {
            false => Tpf0::NoDet,
            true => Tpf0::Det,
        }
    }
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tpf0::NoDet
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tpf0::Det
    }
}
#[doc = "Field `TPF0` writer - Tamper Pin n Flag"]
pub type Tpf0W<'a, REG> = crate::BitWriter1C<'a, REG, Tpf0>;
impl<'a, REG> Tpf0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf0::NoDet)
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf0::Det)
    }
}
#[doc = "Tamper Pin n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpf1 {
    #[doc = "0: Pin tamper not detected"]
    NoDet = 0,
    #[doc = "1: Pin tamper detected"]
    Det = 1,
}
impl From<Tpf1> for bool {
    #[inline(always)]
    fn from(variant: Tpf1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPF1` reader - Tamper Pin n Flag"]
pub type Tpf1R = crate::BitReader<Tpf1>;
impl Tpf1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpf1 {
        match self.bits {
            false => Tpf1::NoDet,
            true => Tpf1::Det,
        }
    }
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tpf1::NoDet
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tpf1::Det
    }
}
#[doc = "Field `TPF1` writer - Tamper Pin n Flag"]
pub type Tpf1W<'a, REG> = crate::BitWriter1C<'a, REG, Tpf1>;
impl<'a, REG> Tpf1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf1::NoDet)
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf1::Det)
    }
}
#[doc = "Tamper Pin n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpf2 {
    #[doc = "0: Pin tamper not detected"]
    NoDet = 0,
    #[doc = "1: Pin tamper detected"]
    Det = 1,
}
impl From<Tpf2> for bool {
    #[inline(always)]
    fn from(variant: Tpf2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPF2` reader - Tamper Pin n Flag"]
pub type Tpf2R = crate::BitReader<Tpf2>;
impl Tpf2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpf2 {
        match self.bits {
            false => Tpf2::NoDet,
            true => Tpf2::Det,
        }
    }
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tpf2::NoDet
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tpf2::Det
    }
}
#[doc = "Field `TPF2` writer - Tamper Pin n Flag"]
pub type Tpf2W<'a, REG> = crate::BitWriter1C<'a, REG, Tpf2>;
impl<'a, REG> Tpf2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf2::NoDet)
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf2::Det)
    }
}
#[doc = "Tamper Pin n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpf3 {
    #[doc = "0: Pin tamper not detected"]
    NoDet = 0,
    #[doc = "1: Pin tamper detected"]
    Det = 1,
}
impl From<Tpf3> for bool {
    #[inline(always)]
    fn from(variant: Tpf3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPF3` reader - Tamper Pin n Flag"]
pub type Tpf3R = crate::BitReader<Tpf3>;
impl Tpf3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpf3 {
        match self.bits {
            false => Tpf3::NoDet,
            true => Tpf3::Det,
        }
    }
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tpf3::NoDet
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tpf3::Det
    }
}
#[doc = "Field `TPF3` writer - Tamper Pin n Flag"]
pub type Tpf3W<'a, REG> = crate::BitWriter1C<'a, REG, Tpf3>;
impl<'a, REG> Tpf3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf3::NoDet)
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf3::Det)
    }
}
#[doc = "Tamper Pin n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpf4 {
    #[doc = "0: Pin tamper not detected"]
    NoDet = 0,
    #[doc = "1: Pin tamper detected"]
    Det = 1,
}
impl From<Tpf4> for bool {
    #[inline(always)]
    fn from(variant: Tpf4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPF4` reader - Tamper Pin n Flag"]
pub type Tpf4R = crate::BitReader<Tpf4>;
impl Tpf4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpf4 {
        match self.bits {
            false => Tpf4::NoDet,
            true => Tpf4::Det,
        }
    }
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tpf4::NoDet
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tpf4::Det
    }
}
#[doc = "Field `TPF4` writer - Tamper Pin n Flag"]
pub type Tpf4W<'a, REG> = crate::BitWriter1C<'a, REG, Tpf4>;
impl<'a, REG> Tpf4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf4::NoDet)
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf4::Det)
    }
}
#[doc = "Tamper Pin n Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpf5 {
    #[doc = "0: Pin tamper not detected"]
    NoDet = 0,
    #[doc = "1: Pin tamper detected"]
    Det = 1,
}
impl From<Tpf5> for bool {
    #[inline(always)]
    fn from(variant: Tpf5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPF5` reader - Tamper Pin n Flag"]
pub type Tpf5R = crate::BitReader<Tpf5>;
impl Tpf5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpf5 {
        match self.bits {
            false => Tpf5::NoDet,
            true => Tpf5::Det,
        }
    }
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn is_no_det(&self) -> bool {
        *self == Tpf5::NoDet
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn is_det(&self) -> bool {
        *self == Tpf5::Det
    }
}
#[doc = "Field `TPF5` writer - Tamper Pin n Flag"]
pub type Tpf5W<'a, REG> = crate::BitWriter1C<'a, REG, Tpf5>;
impl<'a, REG> Tpf5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin tamper not detected"]
    #[inline(always)]
    pub fn no_det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf5::NoDet)
    }
    #[doc = "Pin tamper detected"]
    #[inline(always)]
    pub fn det(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf5::Det)
    }
}
impl R {
    #[doc = "Bit 0 - Digital Tamper Flag"]
    #[inline(always)]
    pub fn dtf(&self) -> DtfR {
        DtfR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Tamper Acknowledge Flag"]
    #[inline(always)]
    pub fn taf(&self) -> TafR {
        TafR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif0(&self) -> Tif0R {
        Tif0R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif1(&self) -> Tif1R {
        Tif1R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif2(&self) -> Tif2R {
        Tif2R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif3(&self) -> Tif3R {
        Tif3R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif4(&self) -> Tif4R {
        Tif4R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif5(&self) -> Tif5R {
        Tif5R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif6(&self) -> Tif6R {
        Tif6R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif7(&self) -> Tif7R {
        Tif7R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif8(&self) -> Tif8R {
        Tif8R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif9(&self) -> Tif9R {
        Tif9R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif10(&self) -> Tif10R {
        Tif10R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 16 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf0(&self) -> Tpf0R {
        Tpf0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf1(&self) -> Tpf1R {
        Tpf1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf2(&self) -> Tpf2R {
        Tpf2R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf3(&self) -> Tpf3R {
        Tpf3R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf4(&self) -> Tpf4R {
        Tpf4R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf5(&self) -> Tpf5R {
        Tpf5R::new(((self.bits >> 21) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Digital Tamper Flag"]
    #[inline(always)]
    pub fn dtf(&mut self) -> DtfW<SrSpec> {
        DtfW::new(self, 0)
    }
    #[doc = "Bit 1 - Tamper Acknowledge Flag"]
    #[inline(always)]
    pub fn taf(&mut self) -> TafW<SrSpec> {
        TafW::new(self, 1)
    }
    #[doc = "Bit 2 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif0(&mut self) -> Tif0W<SrSpec> {
        Tif0W::new(self, 2)
    }
    #[doc = "Bit 3 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif1(&mut self) -> Tif1W<SrSpec> {
        Tif1W::new(self, 3)
    }
    #[doc = "Bit 4 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif2(&mut self) -> Tif2W<SrSpec> {
        Tif2W::new(self, 4)
    }
    #[doc = "Bit 5 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif3(&mut self) -> Tif3W<SrSpec> {
        Tif3W::new(self, 5)
    }
    #[doc = "Bit 6 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif4(&mut self) -> Tif4W<SrSpec> {
        Tif4W::new(self, 6)
    }
    #[doc = "Bit 7 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif5(&mut self) -> Tif5W<SrSpec> {
        Tif5W::new(self, 7)
    }
    #[doc = "Bit 8 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif6(&mut self) -> Tif6W<SrSpec> {
        Tif6W::new(self, 8)
    }
    #[doc = "Bit 9 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif7(&mut self) -> Tif7W<SrSpec> {
        Tif7W::new(self, 9)
    }
    #[doc = "Bit 10 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif8(&mut self) -> Tif8W<SrSpec> {
        Tif8W::new(self, 10)
    }
    #[doc = "Bit 11 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif9(&mut self) -> Tif9W<SrSpec> {
        Tif9W::new(self, 11)
    }
    #[doc = "Bit 12 - Tamper Input n Flag"]
    #[inline(always)]
    pub fn tif10(&mut self) -> Tif10W<SrSpec> {
        Tif10W::new(self, 12)
    }
    #[doc = "Bit 16 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf0(&mut self) -> Tpf0W<SrSpec> {
        Tpf0W::new(self, 16)
    }
    #[doc = "Bit 17 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf1(&mut self) -> Tpf1W<SrSpec> {
        Tpf1W::new(self, 17)
    }
    #[doc = "Bit 18 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf2(&mut self) -> Tpf2W<SrSpec> {
        Tpf2W::new(self, 18)
    }
    #[doc = "Bit 19 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf3(&mut self) -> Tpf3W<SrSpec> {
        Tpf3W::new(self, 19)
    }
    #[doc = "Bit 20 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf4(&mut self) -> Tpf4W<SrSpec> {
        Tpf4W::new(self, 20)
    }
    #[doc = "Bit 21 - Tamper Pin n Flag"]
    #[inline(always)]
    pub fn tpf5(&mut self) -> Tpf5W<SrSpec> {
        Tpf5W::new(self, 21)
    }
}
#[doc = "Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SrSpec;
impl crate::RegisterSpec for SrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sr::R`](R) reader structure"]
impl crate::Readable for SrSpec {}
#[doc = "`write(|w| ..)` method takes [`sr::W`](W) writer structure"]
impl crate::Writable for SrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x003f_1fff;
}
#[doc = "`reset()` method sets SR to value 0"]
impl crate::Resettable for SrSpec {}
