use super::{Hertz, RCC};
use crate::fmt::assert;

const VCO_MIN: u32 = 150_000_000;
const VCO_MAX: u32 = 420_000_000;

#[derive(Default)]
pub struct PllConfig {
    pub p_ck: Option<Hertz>,
    pub q_ck: Option<Hertz>,
    pub r_ck: Option<Hertz>,
}

pub(super) struct PllConfigResults {
    pub ref_x_ck: u32,
    pub pll_x_m: u32,
    pub pll_x_p: u32,
    pub vco_ck_target: u32,
}

fn vco_output_divider_setup(output: u32, plln: usize) -> (u32, u32) {
    let pll_x_p = if plln == 0 {
        if output > VCO_MAX / 2 {
            1
        } else {
            ((VCO_MAX / output) | 1) - 1 // Must be even or unity
        }
    } else {
        // Specific to PLL2/3, will subtract 1 later
        if output > VCO_MAX / 2 {
            1
        } else {
            VCO_MAX / output
        }
    };

    let vco_ck = output + pll_x_p;

    assert!(pll_x_p < 128);
    assert!(vco_ck >= VCO_MIN);
    assert!(vco_ck <= VCO_MAX);

    (vco_ck, pll_x_p)
}

/// # Safety
///
/// Must have exclusive access to the RCC register block
unsafe fn vco_setup(pll_src: u32, requested_output: u32, plln: usize) -> PllConfigResults {
    use crate::pac::rcc::vals::{Pll1rge, Pll1vcosel};

    let (vco_ck_target, pll_x_p) = vco_output_divider_setup(requested_output, plln);

    // Input divisor, resulting in a reference clock in the range
    // 1 to 2 MHz. Choose the highest reference clock (lowest m)
    let pll_x_m = (pll_src + 1_999_999) / 2_000_000;
    assert!(pll_x_m < 64);

    // Calculate resulting reference clock
    let ref_x_ck = pll_src / pll_x_m;
    assert!((1_000_000..=2_000_000).contains(&ref_x_ck));

    RCC.pllcfgr().modify(|w| {
        w.set_pllvcosel(plln, Pll1vcosel::MEDIUMVCO);
        w.set_pllrge(plln, Pll1rge::RANGE1);
    });
    PllConfigResults {
        ref_x_ck,
        pll_x_m,
        pll_x_p,
        vco_ck_target,
    }
}

/// # Safety
///
/// Must have exclusive access to the RCC register block
pub(super) unsafe fn pll_setup(
    pll_src: u32,
    config: &PllConfig,
    plln: usize,
) -> (Option<u32>, Option<u32>, Option<u32>) {
    use crate::pac::rcc::vals::{Divp1, Divp1en, Pll1fracen};

    match config.p_ck {
        Some(requested_output) => {
            let config_results = vco_setup(pll_src, requested_output.0, plln);
            let PllConfigResults {
                ref_x_ck,
                pll_x_m,
                pll_x_p,
                vco_ck_target,
            } = config_results;

            RCC.pllckselr().modify(|w| w.set_divm(plln, pll_x_m as u8));

            // Feedback divider. Integer only
            let pll_x_n = vco_ck_target / ref_x_ck;
            assert!(pll_x_n >= 4);
            assert!(pll_x_n <= 512);
            RCC.plldivr(plln)
                .modify(|w| w.set_divn1((pll_x_n - 1) as u16));

            // No FRACN
            RCC.pllcfgr()
                .modify(|w| w.set_pllfracen(plln, Pll1fracen::RESET));
            let vco_ck = ref_x_ck * pll_x_n;

            RCC.plldivr(plln)
                .modify(|w| w.set_divp1(Divp1((pll_x_p - 1) as u8)));
            RCC.pllcfgr()
                .modify(|w| w.set_divpen(plln, Divp1en::ENABLED));

            // Calulate additional output dividers
            let q_ck = match config.q_ck {
                Some(Hertz(ck)) if ck > 0 => {
                    let div = (vco_ck + ck - 1) / ck;
                    RCC.plldivr(plln).modify(|w| w.set_divq1((div - 1) as u8));
                    RCC.pllcfgr()
                        .modify(|w| w.set_divqen(plln, Divp1en::ENABLED));
                    Some(vco_ck / div)
                }
                _ => None,
            };
            let r_ck = match config.r_ck {
                Some(Hertz(ck)) if ck > 0 => {
                    let div = (vco_ck + ck - 1) / ck;
                    RCC.plldivr(plln).modify(|w| w.set_divr1((div - 1) as u8));
                    RCC.pllcfgr()
                        .modify(|w| w.set_divren(plln, Divp1en::ENABLED));
                    Some(vco_ck / div)
                }
                _ => None,
            };

            (Some(vco_ck / pll_x_p), q_ck, r_ck)
        }
        None => {
            assert!(
                config.q_ck.is_none(),
                "Must set PLL P clock for Q clock to take effect!"
            );
            assert!(
                config.r_ck.is_none(),
                "Must set PLL P clock for R clock to take effect!"
            );
            (None, None, None)
        }
    }
}
