#![no_main]

use libfuzzer_sys::fuzz_target;
use semver::Version;
use std::str;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(string) = str::from_utf8(bytes) {
        let _ = Version::parse(string);
    }
});
