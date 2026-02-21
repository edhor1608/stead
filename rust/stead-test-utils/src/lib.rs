use std::fs;
use std::io;
use std::path::PathBuf;

pub fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative)
}

pub fn load_fixture(relative: &str) -> io::Result<String> {
    fs::read_to_string(fixture_path(relative))
}
