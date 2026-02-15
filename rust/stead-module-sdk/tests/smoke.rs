use stead_module_sdk::crate_identity;

#[test]
fn exposes_crate_identity() {
    assert_eq!(crate_identity(), "stead-module-sdk");
}
