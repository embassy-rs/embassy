use crate::pac;
use crate::pac::interrupt;

#[cfg(feature = "rt")]
#[interrupt]
fn GROUP0() {
    use mspm0_metapac::Group0;

    let group = pac::CPUSS.int_group(0);

    // TODO: Decompose to direct u8
    let iidx = group.iidx().read().stat().to_bits();

    let Ok(group) = pac::Group0::try_from(iidx as u8) else {
        debug!("Invalid IIDX for group 0: {}", iidx);
        return;
    };

    match group {
        Group0::WWDT0 => todo!("implement WWDT0"),
        Group0::DEBUGSS => todo!("implement DEBUGSS"),
        Group0::FLASHCTL => todo!("implement FLASHCTL"),
        Group0::SYSCTL => todo!("implement SYSCTL"),
    }
}
