pub const DRAWS: usize = 10_000;
pub const SEED: u64 = 0x5E45_2026;

pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(match sorted.len() % 2 {
        0 => (sorted[middle - 1] + sorted[middle]) / 2.0,
        _ => sorted[middle],
    })
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn resample(&mut self, values: &[f64]) -> Vec<f64> {
        (0..values.len()).map(|_| values[(self.next() % values.len() as u64) as usize]).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estimate {
    pub point: f64,
    pub low: f64,
    pub high: f64,
}

pub fn difference(base: &[f64], other: &[f64]) -> Option<Estimate> {
    pooled(&[(base, other)])
}

pub fn pooled(cells: &[(&[f64], &[f64])]) -> Option<Estimate> {
    let usable: Vec<&(&[f64], &[f64])> = cells.iter().filter(|(base, other)| base.len() >= 2 && other.len() >= 2).collect();
    if usable.is_empty() {
        return None;
    }
    let mean_of = |differences: Vec<f64>| differences.iter().sum::<f64>() / differences.len() as f64;
    let point = mean_of(usable.iter().map(|(base, other)| median(other).unwrap_or(0.0) - median(base).unwrap_or(0.0)).collect());
    let mut rng = Rng(SEED);
    let mut draws: Vec<f64> = (0..DRAWS)
        .map(|_| {
            mean_of(
                usable
                    .iter()
                    .map(|(base, other)| median(&rng.resample(other)).unwrap_or(0.0) - median(&rng.resample(base)).unwrap_or(0.0))
                    .collect(),
            )
        })
        .collect();
    draws.sort_by(f64::total_cmp);
    let at = |share: f64| draws[((draws.len() - 1) as f64 * share).round() as usize];
    Some(Estimate { point, low: at(0.025), high: at(0.975) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_median_takes_the_middle_or_the_mean_of_the_two_middles() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn a_clear_difference_has_an_interval_that_excludes_zero() {
        let estimate = difference(&[10.0, 11.0, 12.0], &[2.0, 3.0, 4.0]).unwrap();
        assert_eq!(estimate.point, -8.0);
        assert!(estimate.low <= estimate.point && estimate.point <= estimate.high);
        assert!(estimate.high < 0.0, "{estimate:?}");
    }

    #[test]
    fn the_same_data_gives_the_same_interval_every_time() {
        let base = [5.0, 9.0, 7.0, 6.0];
        let other = [4.0, 8.0, 3.0, 9.0];
        assert_eq!(difference(&base, &other), difference(&base, &other));
    }

    #[test]
    fn a_cell_with_fewer_than_two_runs_is_left_out() {
        assert_eq!(difference(&[1.0], &[2.0, 3.0]), None);
        let pooled_two = pooled(&[(&[1.0, 1.0], &[3.0, 3.0]), (&[5.0], &[0.0, 0.0])]).unwrap();
        assert_eq!(pooled_two.point, 2.0);
    }
}
