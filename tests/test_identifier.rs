#![allow(
    clippy::eq_op,
    clippy::needless_pass_by_value,
    clippy::toplevel_ref_arg,
    clippy::wildcard_imports
)]

mod util;

use crate::util::*;
use semver::Prerelease;

#[test]
fn test_new() {
    fn test(identifier: Prerelease, expected: &str) {
        assert_eq!(identifier.is_empty(), expected.is_empty());
        assert_eq!(identifier.len(), expected.len());
        assert_eq!(identifier.as_str(), expected);
        assert_eq!(identifier, identifier);
        assert_eq!(identifier, identifier.clone());
    }

    let ref mut string = String::new();
    let limit = if cfg!(miri) { 40 } else { 280 }; // miri is slow
    for _ in 0..limit {
        test(prerelease(string), string);
        string.push('1');
    }

    if !cfg!(miri) {
        let ref string = string.repeat(20000);
        test(prerelease(string), string);
    }
}

#[test]
fn test_eq() {
    assert_eq!(prerelease("-"), prerelease("-"));
    assert_ne!(prerelease("a"), prerelease("aa"));
    assert_ne!(prerelease("aa"), prerelease("a"));
    assert_ne!(prerelease("aaaaaaaaa"), prerelease("a"));
    assert_ne!(prerelease("a"), prerelease("aaaaaaaaa"));
    assert_ne!(prerelease("aaaaaaaaa"), prerelease("bbbbbbbbb"));
    assert_ne!(build_metadata("1"), build_metadata("001"));
}

#[test]
fn test_comparator() {
    let compatator = comparator("4.4.5-44");
    assert_to_string(compatator, "^4.4.5-44");

    let compatator = comparator("2.X");
    assert_to_string(compatator, "2.*");

    let compatator = comparator("2");
    assert_to_string(compatator, "^2");

    let compatator = comparator("5.x.x");
    assert_to_string(compatator, "5.*");
}

#[test]
fn test_prerelease() {
    let err = prerelease_err("88.6688.b.rrrrrrr8.b.6bbbbbbb66.66\0");
    assert_to_string(err, "unexpected character in pre-release identifier");
}

#[test]
fn test_comparator_err() {
    let err = comparator_err("4.4.4-012");
    assert_to_string(err, "invalid leading zero in pre-release identifier");

    let err = comparator_err("5.4.4+4.");
    assert_to_string(err, "empty identifier segment in build metadata");

    let err = comparator_err(">");
    assert_to_string(
        err,
        "unexpected end of input while parsing major version number",
    );

    let err = comparator_err("4.");
    assert_to_string(
        err,
        "unexpected end of input while parsing minor version number",
    );

    let err = comparator_err("4.*.");
    assert_to_string(err, "unexpected character after wildcard in version req");

    let err = comparator_err("55444.4.45-6+6.4.5.45-5644ÿ");
    assert_to_string(err, "unexpected character 'ÿ' after build metadata");
}
