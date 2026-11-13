pub mod card;
pub mod comments;
pub mod copies;
pub mod dependencies;
pub mod design;
pub mod hot;
pub mod integrity;
pub mod judge;
pub mod orphans;
pub mod protected;
pub mod rank;
pub mod relevant;
pub mod review;
pub mod shape;
pub mod slug;
pub mod unity;
pub mod verdict;

pub const CANON: &str = include_str!("canon.md");
pub const UNITY: &str = include_str!("unity.md");
pub const VERSION: &str = "v1.2";

pub fn canon(unity: bool) -> String {
    if unity { format!("{}\n\n{UNITY}", CANON.trim_end()) } else { CANON.to_string() }
}

pub fn edition(unity: bool) -> String {
    if unity { format!("{VERSION}+unity") } else { VERSION.to_string() }
}

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
    fn the_unity_part_stays_short_and_its_ladder_goes_from_data_to_the_frame() {
        assert!(UNITY.lines().count() <= 30, "{} lines", UNITY.lines().count());
        let steps = ["Is it data?", "already have the component", "Will it run every frame?"];
        let places: Vec<usize> = steps.iter().map(|step| UNITY.find(step).unwrap_or_else(|| panic!("missing {step}"))).collect();
        assert!(places.windows(2).all(|pair| pair[0] < pair[1]), "{places:?}");
    }

    #[test]
    fn a_project_that_is_not_unity_gets_exactly_the_canon_and_a_unity_one_gets_both() {
        assert_eq!(canon(false), CANON);
        assert!(canon(true).starts_with(CANON.trim_end()) && canon(true).ends_with(UNITY));
        assert_eq!((edition(false), edition(true)), (VERSION.to_string(), format!("{VERSION}+unity")));
    }

    #[test]
    fn the_canon_names_its_own_version() {
        assert_eq!(CANON.lines().next(), Some(format!("# Sens Canon {VERSION}").as_str()));
    }
}
