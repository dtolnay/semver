use semver::{BuildMetadata, Error, Prerelease, Version};
use std::fmt::Display;

#[track_caller]
pub(super) fn version(text: &str) -> Version {
    Version::parse(text).unwrap()
}

#[track_caller]
pub(super) fn version_err(text: &str) -> Error {
    Version::parse(text).unwrap_err()
}

#[track_caller]
pub(super) fn prerelease(text: &str) -> Prerelease {
    Prerelease::new(text).unwrap()
}

#[track_caller]
pub(super) fn build_metadata(text: &str) -> BuildMetadata {
    BuildMetadata::new(text).unwrap()
}

#[track_caller]
pub(super) fn assert_to_string(value: impl Display, expected: &str) {
    assert_eq!(value.to_string(), expected);
}
