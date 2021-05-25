#![no_main]

use libfuzzer_sys::fuzz_target;
use semver::VersionReq;
use std::str;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(string) = str::from_utf8(bytes) {
        if let Ok(v1) = VersionReq::parse(string) {
            let v2 = VersionReq::parse(&v1.to_string()).unwrap();
            assert_eq!(v1, v2);
        }
    }
});
