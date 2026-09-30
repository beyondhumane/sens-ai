use sens_agent::session::shorten;

#[test]
fn a_long_title_ends_on_the_last_whole_word() {
    assert_eq!(
        shorten("refactor the session storage so archived conversations keep their folder"),
        "refactor the session storage so archived conversations…"
    );
}

#[test]
fn a_word_that_ends_right_at_the_limit_is_kept() {
    let fits = format!("{}palabras", "palabra ".repeat(6));
    assert_eq!(fits.chars().count(), 56);
    assert_eq!(shorten(&format!("{fits} y más")), format!("{fits}…"));
}

#[test]
fn words_with_accents_are_cut_whole() {
    assert_eq!(shorten(&"canción ".repeat(10)), format!("{}…", "canción ".repeat(7).trim_end()));
}

#[test]
fn a_first_word_too_long_is_still_cut_at_the_limit() {
    assert_eq!(shorten(&format!("{} fin", "x".repeat(80))), format!("{}…", "x".repeat(56)));
}

#[test]
fn a_short_title_is_only_tidied() {
    assert_eq!(shorten("  arregla   el panel "), "arregla el panel");
    let exact = "y".repeat(56);
    assert_eq!(shorten(&exact), exact);
}
