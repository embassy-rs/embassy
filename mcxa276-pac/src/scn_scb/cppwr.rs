#[doc = "Register `CPPWR` reader"]
pub type R = crate::R<CppwrSpec>;
#[doc = "Register `CPPWR` writer"]
pub type W = crate::W<CppwrSpec>;
#[doc = "State UNKNOWN 0.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su0 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su0> for bool {
    #[inline(always)]
    fn from(variant: Su0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU0` reader - State UNKNOWN 0."]
pub type Su0R = crate::BitReader<Su0>;
impl Su0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su0 {
        match self.bits {
            false => Su0::UnknownNotPermitted,
            true => Su0::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su0::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su0::UnknownPermitted
    }
}
#[doc = "Field `SU0` writer - State UNKNOWN 0."]
pub type Su0W<'a, REG> = crate::BitWriter<'a, REG, Su0>;
impl<'a, REG> Su0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su0::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su0::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 0.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus0 {
    #[doc = "0: The SU0 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU0 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus0> for bool {
    #[inline(always)]
    fn from(variant: Sus0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS0` reader - State UNKNOWN Secure only 0."]
pub type Sus0R = crate::BitReader<Sus0>;
impl Sus0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus0 {
        match self.bits {
            false => Sus0::SecureAndNonSecure,
            true => Sus0::SecureOnly,
        }
    }
    #[doc = "The SU0 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus0::SecureAndNonSecure
    }
    #[doc = "The SU0 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus0::SecureOnly
    }
}
#[doc = "Field `SUS0` writer - State UNKNOWN Secure only 0."]
pub type Sus0W<'a, REG> = crate::BitWriter<'a, REG, Sus0>;
impl<'a, REG> Sus0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU0 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus0::SecureAndNonSecure)
    }
    #[doc = "The SU0 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus0::SecureOnly)
    }
}
#[doc = "State UNKNOWN 1.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su1 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su1> for bool {
    #[inline(always)]
    fn from(variant: Su1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU1` reader - State UNKNOWN 1."]
pub type Su1R = crate::BitReader<Su1>;
impl Su1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su1 {
        match self.bits {
            false => Su1::UnknownNotPermitted,
            true => Su1::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su1::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su1::UnknownPermitted
    }
}
#[doc = "Field `SU1` writer - State UNKNOWN 1."]
pub type Su1W<'a, REG> = crate::BitWriter<'a, REG, Su1>;
impl<'a, REG> Su1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su1::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su1::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 1.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus1 {
    #[doc = "0: The SU7 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU7 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus1> for bool {
    #[inline(always)]
    fn from(variant: Sus1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS1` reader - State UNKNOWN Secure only 1."]
pub type Sus1R = crate::BitReader<Sus1>;
impl Sus1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus1 {
        match self.bits {
            false => Sus1::SecureAndNonSecure,
            true => Sus1::SecureOnly,
        }
    }
    #[doc = "The SU7 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus1::SecureAndNonSecure
    }
    #[doc = "The SU7 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus1::SecureOnly
    }
}
#[doc = "Field `SUS1` writer - State UNKNOWN Secure only 1."]
pub type Sus1W<'a, REG> = crate::BitWriter<'a, REG, Sus1>;
impl<'a, REG> Sus1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU7 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus1::SecureAndNonSecure)
    }
    #[doc = "The SU7 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus1::SecureOnly)
    }
}
#[doc = "State UNKNOWN 2.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su2 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su2> for bool {
    #[inline(always)]
    fn from(variant: Su2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU2` reader - State UNKNOWN 2."]
pub type Su2R = crate::BitReader<Su2>;
impl Su2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su2 {
        match self.bits {
            false => Su2::UnknownNotPermitted,
            true => Su2::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su2::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su2::UnknownPermitted
    }
}
#[doc = "Field `SU2` writer - State UNKNOWN 2."]
pub type Su2W<'a, REG> = crate::BitWriter<'a, REG, Su2>;
impl<'a, REG> Su2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su2::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su2::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 2.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus2 {
    #[doc = "0: The SU2 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU2 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus2> for bool {
    #[inline(always)]
    fn from(variant: Sus2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS2` reader - State UNKNOWN Secure only 2."]
pub type Sus2R = crate::BitReader<Sus2>;
impl Sus2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus2 {
        match self.bits {
            false => Sus2::SecureAndNonSecure,
            true => Sus2::SecureOnly,
        }
    }
    #[doc = "The SU2 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus2::SecureAndNonSecure
    }
    #[doc = "The SU2 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus2::SecureOnly
    }
}
#[doc = "Field `SUS2` writer - State UNKNOWN Secure only 2."]
pub type Sus2W<'a, REG> = crate::BitWriter<'a, REG, Sus2>;
impl<'a, REG> Sus2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU2 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus2::SecureAndNonSecure)
    }
    #[doc = "The SU2 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus2::SecureOnly)
    }
}
#[doc = "State UNKNOWN 3.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su3 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su3> for bool {
    #[inline(always)]
    fn from(variant: Su3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU3` reader - State UNKNOWN 3."]
pub type Su3R = crate::BitReader<Su3>;
impl Su3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su3 {
        match self.bits {
            false => Su3::UnknownNotPermitted,
            true => Su3::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su3::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su3::UnknownPermitted
    }
}
#[doc = "Field `SU3` writer - State UNKNOWN 3."]
pub type Su3W<'a, REG> = crate::BitWriter<'a, REG, Su3>;
impl<'a, REG> Su3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su3::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su3::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 3.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus3 {
    #[doc = "0: The SU3 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU3 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus3> for bool {
    #[inline(always)]
    fn from(variant: Sus3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS3` reader - State UNKNOWN Secure only 3."]
pub type Sus3R = crate::BitReader<Sus3>;
impl Sus3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus3 {
        match self.bits {
            false => Sus3::SecureAndNonSecure,
            true => Sus3::SecureOnly,
        }
    }
    #[doc = "The SU3 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus3::SecureAndNonSecure
    }
    #[doc = "The SU3 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus3::SecureOnly
    }
}
#[doc = "Field `SUS3` writer - State UNKNOWN Secure only 3."]
pub type Sus3W<'a, REG> = crate::BitWriter<'a, REG, Sus3>;
impl<'a, REG> Sus3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU3 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus3::SecureAndNonSecure)
    }
    #[doc = "The SU3 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus3::SecureOnly)
    }
}
#[doc = "State UNKNOWN 4.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su4 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su4> for bool {
    #[inline(always)]
    fn from(variant: Su4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU4` reader - State UNKNOWN 4."]
pub type Su4R = crate::BitReader<Su4>;
impl Su4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su4 {
        match self.bits {
            false => Su4::UnknownNotPermitted,
            true => Su4::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su4::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su4::UnknownPermitted
    }
}
#[doc = "Field `SU4` writer - State UNKNOWN 4."]
pub type Su4W<'a, REG> = crate::BitWriter<'a, REG, Su4>;
impl<'a, REG> Su4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su4::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su4::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 4.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus4 {
    #[doc = "0: The SU4 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU4 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus4> for bool {
    #[inline(always)]
    fn from(variant: Sus4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS4` reader - State UNKNOWN Secure only 4."]
pub type Sus4R = crate::BitReader<Sus4>;
impl Sus4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus4 {
        match self.bits {
            false => Sus4::SecureAndNonSecure,
            true => Sus4::SecureOnly,
        }
    }
    #[doc = "The SU4 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus4::SecureAndNonSecure
    }
    #[doc = "The SU4 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus4::SecureOnly
    }
}
#[doc = "Field `SUS4` writer - State UNKNOWN Secure only 4."]
pub type Sus4W<'a, REG> = crate::BitWriter<'a, REG, Sus4>;
impl<'a, REG> Sus4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU4 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus4::SecureAndNonSecure)
    }
    #[doc = "The SU4 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus4::SecureOnly)
    }
}
#[doc = "State UNKNOWN 5.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su5 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su5> for bool {
    #[inline(always)]
    fn from(variant: Su5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU5` reader - State UNKNOWN 5."]
pub type Su5R = crate::BitReader<Su5>;
impl Su5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su5 {
        match self.bits {
            false => Su5::UnknownNotPermitted,
            true => Su5::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su5::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su5::UnknownPermitted
    }
}
#[doc = "Field `SU5` writer - State UNKNOWN 5."]
pub type Su5W<'a, REG> = crate::BitWriter<'a, REG, Su5>;
impl<'a, REG> Su5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su5::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su5::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 5.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus5 {
    #[doc = "0: The SU5 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU5 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus5> for bool {
    #[inline(always)]
    fn from(variant: Sus5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS5` reader - State UNKNOWN Secure only 5."]
pub type Sus5R = crate::BitReader<Sus5>;
impl Sus5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus5 {
        match self.bits {
            false => Sus5::SecureAndNonSecure,
            true => Sus5::SecureOnly,
        }
    }
    #[doc = "The SU5 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus5::SecureAndNonSecure
    }
    #[doc = "The SU5 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus5::SecureOnly
    }
}
#[doc = "Field `SUS5` writer - State UNKNOWN Secure only 5."]
pub type Sus5W<'a, REG> = crate::BitWriter<'a, REG, Sus5>;
impl<'a, REG> Sus5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU5 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus5::SecureAndNonSecure)
    }
    #[doc = "The SU5 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus5::SecureOnly)
    }
}
#[doc = "State UNKNOWN 6.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su6 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su6> for bool {
    #[inline(always)]
    fn from(variant: Su6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU6` reader - State UNKNOWN 6."]
pub type Su6R = crate::BitReader<Su6>;
impl Su6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su6 {
        match self.bits {
            false => Su6::UnknownNotPermitted,
            true => Su6::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su6::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su6::UnknownPermitted
    }
}
#[doc = "Field `SU6` writer - State UNKNOWN 6."]
pub type Su6W<'a, REG> = crate::BitWriter<'a, REG, Su6>;
impl<'a, REG> Su6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su6::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su6::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 6.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus6 {
    #[doc = "0: The SU6 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU6 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus6> for bool {
    #[inline(always)]
    fn from(variant: Sus6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS6` reader - State UNKNOWN Secure only 6."]
pub type Sus6R = crate::BitReader<Sus6>;
impl Sus6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus6 {
        match self.bits {
            false => Sus6::SecureAndNonSecure,
            true => Sus6::SecureOnly,
        }
    }
    #[doc = "The SU6 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus6::SecureAndNonSecure
    }
    #[doc = "The SU6 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus6::SecureOnly
    }
}
#[doc = "Field `SUS6` writer - State UNKNOWN Secure only 6."]
pub type Sus6W<'a, REG> = crate::BitWriter<'a, REG, Sus6>;
impl<'a, REG> Sus6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU6 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus6::SecureAndNonSecure)
    }
    #[doc = "The SU6 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus6::SecureOnly)
    }
}
#[doc = "State UNKNOWN 7.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su7 {
    #[doc = "0: The coprocessor state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The coprocessor state is permitted to become UNKNOWN."]
    UnknownPermitted = 1,
}
impl From<Su7> for bool {
    #[inline(always)]
    fn from(variant: Su7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU7` reader - State UNKNOWN 7."]
pub type Su7R = crate::BitReader<Su7>;
impl Su7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su7 {
        match self.bits {
            false => Su7::UnknownNotPermitted,
            true => Su7::UnknownPermitted,
        }
    }
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su7::UnknownNotPermitted
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su7::UnknownPermitted
    }
}
#[doc = "Field `SU7` writer - State UNKNOWN 7."]
pub type Su7W<'a, REG> = crate::BitWriter<'a, REG, Su7>;
impl<'a, REG> Su7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The coprocessor state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su7::UnknownNotPermitted)
    }
    #[doc = "The coprocessor state is permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su7::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 7.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus7 {
    #[doc = "0: The SU7 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU7 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus7> for bool {
    #[inline(always)]
    fn from(variant: Sus7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS7` reader - State UNKNOWN Secure only 7."]
pub type Sus7R = crate::BitReader<Sus7>;
impl Sus7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus7 {
        match self.bits {
            false => Sus7::SecureAndNonSecure,
            true => Sus7::SecureOnly,
        }
    }
    #[doc = "The SU7 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus7::SecureAndNonSecure
    }
    #[doc = "The SU7 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus7::SecureOnly
    }
}
#[doc = "Field `SUS7` writer - State UNKNOWN Secure only 7."]
pub type Sus7W<'a, REG> = crate::BitWriter<'a, REG, Sus7>;
impl<'a, REG> Sus7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU7 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus7::SecureAndNonSecure)
    }
    #[doc = "The SU7 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus7::SecureOnly)
    }
}
#[doc = "State UNKNOWN 10.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Su10 {
    #[doc = "0: The floating-point state is not permitted to become UNKNOWN."]
    UnknownNotPermitted = 0,
    #[doc = "1: The floating-point state is permitted to become UNKNOWN"]
    UnknownPermitted = 1,
}
impl From<Su10> for bool {
    #[inline(always)]
    fn from(variant: Su10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SU10` reader - State UNKNOWN 10."]
pub type Su10R = crate::BitReader<Su10>;
impl Su10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Su10 {
        match self.bits {
            false => Su10::UnknownNotPermitted,
            true => Su10::UnknownPermitted,
        }
    }
    #[doc = "The floating-point state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn is_unknown_not_permitted(&self) -> bool {
        *self == Su10::UnknownNotPermitted
    }
    #[doc = "The floating-point state is permitted to become UNKNOWN"]
    #[inline(always)]
    pub fn is_unknown_permitted(&self) -> bool {
        *self == Su10::UnknownPermitted
    }
}
#[doc = "Field `SU10` writer - State UNKNOWN 10."]
pub type Su10W<'a, REG> = crate::BitWriter<'a, REG, Su10>;
impl<'a, REG> Su10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The floating-point state is not permitted to become UNKNOWN."]
    #[inline(always)]
    pub fn unknown_not_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su10::UnknownNotPermitted)
    }
    #[doc = "The floating-point state is permitted to become UNKNOWN"]
    #[inline(always)]
    pub fn unknown_permitted(self) -> &'a mut crate::W<REG> {
        self.variant(Su10::UnknownPermitted)
    }
}
#[doc = "State UNKNOWN Secure only 10.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sus10 {
    #[doc = "0: The SU10 field is accessible from both Security states."]
    SecureAndNonSecure = 0,
    #[doc = "1: The SU10 field is only accessible from the Secure state."]
    SecureOnly = 1,
}
impl From<Sus10> for bool {
    #[inline(always)]
    fn from(variant: Sus10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SUS10` reader - State UNKNOWN Secure only 10."]
pub type Sus10R = crate::BitReader<Sus10>;
impl Sus10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sus10 {
        match self.bits {
            false => Sus10::SecureAndNonSecure,
            true => Sus10::SecureOnly,
        }
    }
    #[doc = "The SU10 field is accessible from both Security states."]
    #[inline(always)]
    pub fn is_secure_and_non_secure(&self) -> bool {
        *self == Sus10::SecureAndNonSecure
    }
    #[doc = "The SU10 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn is_secure_only(&self) -> bool {
        *self == Sus10::SecureOnly
    }
}
#[doc = "Field `SUS10` writer - State UNKNOWN Secure only 10."]
pub type Sus10W<'a, REG> = crate::BitWriter<'a, REG, Sus10>;
impl<'a, REG> Sus10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The SU10 field is accessible from both Security states."]
    #[inline(always)]
    pub fn secure_and_non_secure(self) -> &'a mut crate::W<REG> {
        self.variant(Sus10::SecureAndNonSecure)
    }
    #[doc = "The SU10 field is only accessible from the Secure state."]
    #[inline(always)]
    pub fn secure_only(self) -> &'a mut crate::W<REG> {
        self.variant(Sus10::SecureOnly)
    }
}
#[doc = "Field `SU11` reader - State UNKNOWN 11."]
pub type Su11R = crate::BitReader;
#[doc = "Field `SU11` writer - State UNKNOWN 11."]
pub type Su11W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SUS11` reader - State UNKNOWN Secure only 11."]
pub type Sus11R = crate::BitReader;
#[doc = "Field `SUS11` writer - State UNKNOWN Secure only 11."]
pub type Sus11W<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - State UNKNOWN 0."]
    #[inline(always)]
    pub fn su0(&self) -> Su0R {
        Su0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - State UNKNOWN Secure only 0."]
    #[inline(always)]
    pub fn sus0(&self) -> Sus0R {
        Sus0R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - State UNKNOWN 1."]
    #[inline(always)]
    pub fn su1(&self) -> Su1R {
        Su1R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - State UNKNOWN Secure only 1."]
    #[inline(always)]
    pub fn sus1(&self) -> Sus1R {
        Sus1R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - State UNKNOWN 2."]
    #[inline(always)]
    pub fn su2(&self) -> Su2R {
        Su2R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - State UNKNOWN Secure only 2."]
    #[inline(always)]
    pub fn sus2(&self) -> Sus2R {
        Sus2R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - State UNKNOWN 3."]
    #[inline(always)]
    pub fn su3(&self) -> Su3R {
        Su3R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - State UNKNOWN Secure only 3."]
    #[inline(always)]
    pub fn sus3(&self) -> Sus3R {
        Sus3R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - State UNKNOWN 4."]
    #[inline(always)]
    pub fn su4(&self) -> Su4R {
        Su4R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - State UNKNOWN Secure only 4."]
    #[inline(always)]
    pub fn sus4(&self) -> Sus4R {
        Sus4R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - State UNKNOWN 5."]
    #[inline(always)]
    pub fn su5(&self) -> Su5R {
        Su5R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - State UNKNOWN Secure only 5."]
    #[inline(always)]
    pub fn sus5(&self) -> Sus5R {
        Sus5R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - State UNKNOWN 6."]
    #[inline(always)]
    pub fn su6(&self) -> Su6R {
        Su6R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - State UNKNOWN Secure only 6."]
    #[inline(always)]
    pub fn sus6(&self) -> Sus6R {
        Sus6R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - State UNKNOWN 7."]
    #[inline(always)]
    pub fn su7(&self) -> Su7R {
        Su7R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - State UNKNOWN Secure only 7."]
    #[inline(always)]
    pub fn sus7(&self) -> Sus7R {
        Sus7R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 20 - State UNKNOWN 10."]
    #[inline(always)]
    pub fn su10(&self) -> Su10R {
        Su10R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - State UNKNOWN Secure only 10."]
    #[inline(always)]
    pub fn sus10(&self) -> Sus10R {
        Sus10R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - State UNKNOWN 11."]
    #[inline(always)]
    pub fn su11(&self) -> Su11R {
        Su11R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - State UNKNOWN Secure only 11."]
    #[inline(always)]
    pub fn sus11(&self) -> Sus11R {
        Sus11R::new(((self.bits >> 23) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - State UNKNOWN 0."]
    #[inline(always)]
    pub fn su0(&mut self) -> Su0W<CppwrSpec> {
        Su0W::new(self, 0)
    }
    #[doc = "Bit 1 - State UNKNOWN Secure only 0."]
    #[inline(always)]
    pub fn sus0(&mut self) -> Sus0W<CppwrSpec> {
        Sus0W::new(self, 1)
    }
    #[doc = "Bit 2 - State UNKNOWN 1."]
    #[inline(always)]
    pub fn su1(&mut self) -> Su1W<CppwrSpec> {
        Su1W::new(self, 2)
    }
    #[doc = "Bit 3 - State UNKNOWN Secure only 1."]
    #[inline(always)]
    pub fn sus1(&mut self) -> Sus1W<CppwrSpec> {
        Sus1W::new(self, 3)
    }
    #[doc = "Bit 4 - State UNKNOWN 2."]
    #[inline(always)]
    pub fn su2(&mut self) -> Su2W<CppwrSpec> {
        Su2W::new(self, 4)
    }
    #[doc = "Bit 5 - State UNKNOWN Secure only 2."]
    #[inline(always)]
    pub fn sus2(&mut self) -> Sus2W<CppwrSpec> {
        Sus2W::new(self, 5)
    }
    #[doc = "Bit 6 - State UNKNOWN 3."]
    #[inline(always)]
    pub fn su3(&mut self) -> Su3W<CppwrSpec> {
        Su3W::new(self, 6)
    }
    #[doc = "Bit 7 - State UNKNOWN Secure only 3."]
    #[inline(always)]
    pub fn sus3(&mut self) -> Sus3W<CppwrSpec> {
        Sus3W::new(self, 7)
    }
    #[doc = "Bit 8 - State UNKNOWN 4."]
    #[inline(always)]
    pub fn su4(&mut self) -> Su4W<CppwrSpec> {
        Su4W::new(self, 8)
    }
    #[doc = "Bit 9 - State UNKNOWN Secure only 4."]
    #[inline(always)]
    pub fn sus4(&mut self) -> Sus4W<CppwrSpec> {
        Sus4W::new(self, 9)
    }
    #[doc = "Bit 10 - State UNKNOWN 5."]
    #[inline(always)]
    pub fn su5(&mut self) -> Su5W<CppwrSpec> {
        Su5W::new(self, 10)
    }
    #[doc = "Bit 11 - State UNKNOWN Secure only 5."]
    #[inline(always)]
    pub fn sus5(&mut self) -> Sus5W<CppwrSpec> {
        Sus5W::new(self, 11)
    }
    #[doc = "Bit 12 - State UNKNOWN 6."]
    #[inline(always)]
    pub fn su6(&mut self) -> Su6W<CppwrSpec> {
        Su6W::new(self, 12)
    }
    #[doc = "Bit 13 - State UNKNOWN Secure only 6."]
    #[inline(always)]
    pub fn sus6(&mut self) -> Sus6W<CppwrSpec> {
        Sus6W::new(self, 13)
    }
    #[doc = "Bit 14 - State UNKNOWN 7."]
    #[inline(always)]
    pub fn su7(&mut self) -> Su7W<CppwrSpec> {
        Su7W::new(self, 14)
    }
    #[doc = "Bit 15 - State UNKNOWN Secure only 7."]
    #[inline(always)]
    pub fn sus7(&mut self) -> Sus7W<CppwrSpec> {
        Sus7W::new(self, 15)
    }
    #[doc = "Bit 20 - State UNKNOWN 10."]
    #[inline(always)]
    pub fn su10(&mut self) -> Su10W<CppwrSpec> {
        Su10W::new(self, 20)
    }
    #[doc = "Bit 21 - State UNKNOWN Secure only 10."]
    #[inline(always)]
    pub fn sus10(&mut self) -> Sus10W<CppwrSpec> {
        Sus10W::new(self, 21)
    }
    #[doc = "Bit 22 - State UNKNOWN 11."]
    #[inline(always)]
    pub fn su11(&mut self) -> Su11W<CppwrSpec> {
        Su11W::new(self, 22)
    }
    #[doc = "Bit 23 - State UNKNOWN Secure only 11."]
    #[inline(always)]
    pub fn sus11(&mut self) -> Sus11W<CppwrSpec> {
        Sus11W::new(self, 23)
    }
}
#[doc = "Coprocessor Power Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cppwr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cppwr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CppwrSpec;
impl crate::RegisterSpec for CppwrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cppwr::R`](R) reader structure"]
impl crate::Readable for CppwrSpec {}
#[doc = "`write(|w| ..)` method takes [`cppwr::W`](W) writer structure"]
impl crate::Writable for CppwrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CPPWR to value 0"]
impl crate::Resettable for CppwrSpec {}
