use stead_resources::crate_identity;

#[test]
fn exposes_crate_identity() {
    assert_eq!(crate_identity(), "stead-resources");
}
