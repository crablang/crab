use super::UnstableFeatures;

#[test]
fn crablangc_bootstrap_parsing() {
    let is_bootstrap = |env, krate| {
        std::env::set_var("CRABLANGC_BOOTSTRAP", env);
        matches!(UnstableFeatures::from_environment(krate), UnstableFeatures::Cheat)
    };
    assert!(is_bootstrap("1", None));
    assert!(is_bootstrap("1", Some("x")));
    // CRABLANGC_BOOTSTRAP allows specifying a specific crate
    assert!(is_bootstrap("x", Some("x")));
    // CRABLANGC_BOOTSTRAP allows multiple comma-delimited crates
    assert!(is_bootstrap("x,y,z", Some("x")));
    assert!(is_bootstrap("x,y,z", Some("y")));
    // Crate that aren't specified do not get unstable features
    assert!(!is_bootstrap("x", Some("a")));
    assert!(!is_bootstrap("x,y,z", Some("a")));
    assert!(!is_bootstrap("x,y,z", None));

    // this is technically a breaking change, but there are no stability guarantees for CRABLANGC_BOOTSTRAP
    assert!(!is_bootstrap("0", None));
}
