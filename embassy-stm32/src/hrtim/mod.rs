//! High Resolution Timer (HRTIM)

mod traits;

use core::marker::PhantomData;

pub use traits::Instance;

pub mod advanced_pwm;
pub mod bridge_converter;
pub mod resonant_converter;
pub use bridge_converter::BridgeConverter;
pub use resonant_converter::ResonantConverter;

/// HRTIM burst controller instance.
pub struct BurstController<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM master instance.
pub struct Master<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel A instance.
pub struct ChA<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel B instance.
pub struct ChB<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel C instance.
pub struct ChC<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel D instance.
pub struct ChD<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel E instance.
pub struct ChE<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel F instance.
#[cfg(hrtim_v2)]
pub struct ChF<T: Instance> {
    phantom: PhantomData<T>,
}

trait SealedAdvancedChannel<T: Instance> {
    fn raw() -> usize;
}

/// Advanced channel instance trait.
#[allow(private_bounds)]
pub trait AdvancedChannel<T: Instance>: SealedAdvancedChannel<T> {}

macro_rules! advanced_channel_impl {
    ($new_chx:ident, $new_chx_with_config:ident, $channel:tt, $ch_num:expr, $pin_trait:ident, $complementary_pin_trait:ident) => {
        impl<T: Instance> SealedAdvancedChannel<T> for $channel<T> {
            fn raw() -> usize {
                $ch_num
            }
        }
        impl<T: Instance> AdvancedChannel<T> for $channel<T> {}
    };
}

advanced_channel_impl!(
    new_cha,
    new_cha_with_config,
    ChA,
    0,
    ChannelAPin,
    ChannelAComplementaryPin
);
advanced_channel_impl!(
    new_chb,
    new_chb_with_config,
    ChB,
    1,
    ChannelBPin,
    ChannelBComplementaryPin
);
advanced_channel_impl!(
    new_chc,
    new_chc_with_config,
    ChC,
    2,
    ChannelCPin,
    ChannelCComplementaryPin
);
advanced_channel_impl!(
    new_chd,
    new_chd_with_config,
    ChD,
    3,
    ChannelDPin,
    ChannelDComplementaryPin
);
advanced_channel_impl!(
    new_che,
    new_che_with_config,
    ChE,
    4,
    ChannelEPin,
    ChannelEComplementaryPin
);
#[cfg(hrtim_v2)]
advanced_channel_impl!(
    new_chf,
    new_chf_with_config,
    ChF,
    5,
    ChannelFPin,
    ChannelFComplementaryPin
);

pin_trait!(ChannelAPin, Instance);
pin_trait!(ChannelAComplementaryPin, Instance);
pin_trait!(ChannelBPin, Instance);
pin_trait!(ChannelBComplementaryPin, Instance);
pin_trait!(ChannelCPin, Instance);
pin_trait!(ChannelCComplementaryPin, Instance);
pin_trait!(ChannelDPin, Instance);
pin_trait!(ChannelDComplementaryPin, Instance);
pin_trait!(ChannelEPin, Instance);
pin_trait!(ChannelEComplementaryPin, Instance);
#[cfg(hrtim_v2)]
pin_trait!(ChannelFPin, Instance);
#[cfg(hrtim_v2)]
pin_trait!(ChannelFComplementaryPin, Instance);
