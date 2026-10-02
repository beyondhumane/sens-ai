pub mod card;
pub mod comments;
pub mod copies;
pub mod dependencies;
pub mod integrity;
pub mod judge;
pub mod orphans;
pub mod protected;
pub mod relevant;
pub mod review;
pub mod verdict;

pub const CANON: &str = include_str!("canon.md");
pub const VERSION: &str = "v1.1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_canon_stays_short_enough_to_be_read_every_session() {
        assert!(CANON.lines().count() <= 40, "{} lines", CANON.lines().count());
    }

    #[test]
    fn the_ladder_goes_from_need_to_new_code_in_order() {
        let steps = ["Is it needed?", "already have it?", "standard library", "installed dependency", "Only then write new code"];
        let places: Vec<usize> = steps.iter().map(|step| CANON.find(step).unwrap_or_else(|| panic!("missing {step}"))).collect();
        assert!(places.windows(2).all(|pair| pair[0] < pair[1]), "{places:?}");
    }

    #[test]
    fn less_code_never_means_fewer_tests_or_trusting_sens_to_run_them() {
        assert!(CANON.contains("A test that proves the change is part of the change"));
        assert!(CANON.contains("never means skipping tests"));
        assert!(CANON.contains("Its approval is not a test run"));
    }

    #[test]
    fn the_canon_names_its_own_version() {
        assert_eq!(CANON.lines().next(), Some(format!("# Sens Canon {VERSION}").as_str()));
    }
}
