use defmt::error;

pub fn verify_equal<T: PartialEq + defmt::Format + Copy>(actual: &[T], expected: &[T], label: &str) {
    if actual != expected {
        let mismatch_idx = actual
            .iter()
            .zip(expected.iter())
            .position(|(a, e)| a != e)
            .unwrap_or(actual.len().min(expected.len()));
        error!(
            "{} mismatch len={}/{} first_diff@{} actual={} expected={}",
            label,
            actual.len(),
            expected.len(),
            mismatch_idx,
            actual.get(mismatch_idx),
            expected.get(mismatch_idx),
        );
        let end = (mismatch_idx + 8).min(actual.len());
        let start = mismatch_idx.saturating_sub(2);
        error!("  actual  [{}..{}] = {}", start, end, &actual[start..end]);
        error!("  expected[{}..{}] = {}", start, end, &expected[start..end]);
        panic!();
    }
}
