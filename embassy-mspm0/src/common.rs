#[cfg(mspm0g)]
#[allow(unused)]
pub(crate) const fn get_ulpck_frequency() -> u32 {
    40_000_000
}

// TODO: Replace this when the MCLK rate can be adjusted.
#[cfg(any(mspm0c110x, mspm0c1105_c1106))]
pub(crate) const fn get_mclk_frequency() -> u32 {
    24_000_000
}

// TODO: Replace this when the MCLK rate can be adjusted.
#[cfg(any(
    mspm0g110x, mspm0g150x, mspm0g151x, mspm0g310x, mspm0g350x, mspm0g351x, mspm0h321x, mspm0l110x, mspm0l122x,
    mspm0l130x, mspm0l134x, mspm0l222x
))]
pub(crate) const fn get_mclk_frequency() -> u32 {
    32_000_000
}

/// Naive hill-climbing algorithm
///
/// NOTE: this doesn't typically give optimal results as there is no lookahead
pub fn hillclimb<const C: usize, F: Fn(&[u32; C]) -> i32>(mut initial_state: [u32; C], derivate_fn: F) -> [u32; C] {
    let mut last_error = derivate_fn(&initial_state);

    // Check each axis to find best incline
    loop {
        let mut best_error: i32 = i32::MAX;
        let mut best_delta: (usize, bool) = (usize::MAX, false);
        for i in 0..C {
            let mut test_state = initial_state.clone();
            test_state[i] = test_state[i].saturating_add(1);
            let add_error = derivate_fn(&initial_state).abs();
            if add_error < best_error {
                best_delta = (i, false);
                best_error = add_error;
            }
            test_state[i] = test_state[i].saturating_sub(2);
            let sub_error = derivate_fn(&initial_state).abs();
            if sub_error < best_error {
                best_delta = (i, true);
                best_error = sub_error;
            }
        }

        // Surrounding errors are higher than current
        if best_error >= last_error.abs() {
            break;
        }

        last_error = best_error;

        if best_delta.1 {
            initial_state[best_delta.0] -= 1;
        } else {
            initial_state[best_delta.0] += 1;
        }
    }

    initial_state
}

#[cfg(test)]
mod tests {
    use crate::common::hillclimb;

    #[test]
    fn simple_optimization() {
        let target = [8u32, 32u32];
        let val = hillclimb([0u32; 2], |x| {
            (x[0] as i32 - target[0] as i32).abs() + (x[1] as i32 - target[1] as i32)
        });

        assert_eq!(target, val);
    }
}
