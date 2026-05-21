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
            let add_error = derivate_fn(&test_state).abs();
            if add_error < best_error {
                best_delta = (i, false);
                best_error = add_error;
            }
            test_state[i] = test_state[i].saturating_sub(2);
            let sub_error = derivate_fn(&test_state).abs();
            if sub_error < best_error {
                best_delta = (i, true);
                best_error = sub_error;
            }

            #[cfg(feature = "defmt")]
            defmt::trace!("Evaluated axis {}, best_error {}", i, best_error);
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
