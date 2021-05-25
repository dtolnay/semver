#![no_main]

use libfuzzer_sys::fuzz_target;
use semver::{BuildMetadata, Prerelease, Version};
use std::str;

fuzz_target!(|bytes: &[u8]| {
    if let Some([a, b, c]) = three_versions(bytes) {
        if a < b && b < c && c < a {
            panic!("{0} < {1} < {2} < {0}", a, b, c);
        }
    }
});

fn three_versions(bytes: &[u8]) -> Option<[Version; 3]> {
    let mut inputs = str::from_utf8(bytes).ok()?.split(' ');
    Some([
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: inputs.next().map(Prerelease::new)?.ok()?,
            build: inputs.next().map(BuildMetadata::new)?.ok()?,
        },
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: inputs.next().map(Prerelease::new)?.ok()?,
            build: inputs.next().map(BuildMetadata::new)?.ok()?,
        },
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: inputs.next().map(Prerelease::new)?.ok()?,
            build: inputs.next().map(BuildMetadata::new)?.ok()?,
        },
    ])
}
