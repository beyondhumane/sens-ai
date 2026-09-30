use sens_agent::session;

fn root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("sens-accept-ids-{}", std::process::id())).join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn a_legacy_id_opens_a_session() {
    let root = root("legacy");
    assert_eq!(session::open_as(&root, "ab12cd34").unwrap(), "ab12cd34");
    assert!(session::exists(&root, "ab12cd34"));
    assert!(matches!(session::read(&root, "ab12cd34")[0], session::Entry::Opened { .. }));
}

#[test]
fn a_legacy_id_can_be_created_whole() {
    let root = root("created");
    session::create(&root, "0f9e8d7c", &[session::Entry::Opened { at: 1, root: "p".into() }]).unwrap();
    assert!(session::exists(&root, "0f9e8d7c"));
}

#[test]
fn a_uuid_still_opens_a_session() {
    let root = root("uuid");
    let id = session::fresh_id();
    assert_eq!(session::open_as(&root, &id).unwrap(), id);
}

#[test]
fn anything_else_is_still_refused() {
    let root = root("refused");
    for bad in ["../x", "ab12cd3", "ab12cd3g", "AB12CD34", "ab12cd345", "", "a/b", "ab12-d34", "../ab12cd34", "ab12cd34/"] {
        assert!(session::open_as(&root, bad).is_err(), "{bad}");
    }
}

#[test]
fn a_legacy_id_is_not_a_uuid() {
    assert!(!session::is_uuid("ab12cd34"));
}
