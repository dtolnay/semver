mod util;

use crate::util::*;
use semver::BareVersion;

#[test]
fn test_parse() {
    let err = bare_version_err("");
    assert_to_string(err, "empty string, expected a semver version");

    let err = bare_version_err("  ");
    assert_to_string(
        err,
        "unexpected character ' ' while parsing major version number",
    );

    let err = bare_version_err("1");
    assert_to_string(
        err,
        "unexpected end of input while parsing major version number",
    );

    let parsed = bare_version("1.2");
    let expected = BareVersion::new(1, 2, None);
    assert_eq!(parsed, expected);
    let expected = BareVersion {
        major: 1,
        minor: 2,
        patch: None,
    };
    assert_eq!(parsed, expected);

    let err = bare_version_err("1.2.3-");
    assert_to_string(err, "unexpected character '-' after patch version number");

    let err = bare_version_err("a.b.c");
    assert_to_string(
        err,
        "unexpected character 'a' while parsing major version number",
    );

    let parsed = bare_version("1.2.3");
    let expected = BareVersion::new(1, 2, Some(3));
    assert_eq!(parsed, expected);
    let expected = BareVersion {
        major: 1,
        minor: 2,
        patch: Some(3),
    };
    assert_eq!(parsed, expected);
}

#[test]
fn test_eq() {
    assert_eq!(bare_version("1.2.3"), bare_version("1.2.3"));
}

#[test]
fn test_ne() {
    assert_ne!(bare_version("0.0.0"), bare_version("0.0.1"));
    assert_ne!(bare_version("0.0.0"), bare_version("0.1.0"));
    assert_ne!(bare_version("0.0.0"), bare_version("1.0.0"));
}

#[test]
fn test_display() {
    assert_to_string(bare_version("1.2.3"), "1.2.3");
}

#[test]
fn test_lt() {
    assert!(bare_version("0.0.0") < bare_version("1.2.3"));
    assert!(bare_version("1.0.0") < bare_version("1.2.3"));
    assert!(bare_version("1.2.0") < bare_version("1.2.3"));
    assert!(bare_version("1.2") < bare_version("1.2.3"));
}

#[test]
fn test_le() {
    assert!(bare_version("0.0.0") <= bare_version("1.2.3"));
    assert!(bare_version("1.0.0") <= bare_version("1.2.3"));
    assert!(bare_version("1.2.0") <= bare_version("1.2.3"));
    assert!(bare_version("1.2") <= bare_version("1.2.3"));
    assert!(bare_version("1.2.3") <= bare_version("1.2.3"));
}

#[test]
fn test_gt() {
    assert!(bare_version("1.2.3") > bare_version("0.0.0"));
    assert!(bare_version("1.2.3") > bare_version("1.0.0"));
    assert!(bare_version("1.2.3") > bare_version("1.2.0"));
    assert!(bare_version("1.2.3") > bare_version("1.2"));
}

#[test]
fn test_ge() {
    assert!(bare_version("1.2.3") >= bare_version("0.0.0"));
    assert!(bare_version("1.2.3") >= bare_version("1.0.0"));
    assert!(bare_version("1.2.3") >= bare_version("1.2.0"));
    assert!(bare_version("1.2.3") >= bare_version("1.2"));
    assert!(bare_version("1.2.3") >= bare_version("1.2.3"));
}

#[test]
fn test_align() {
    let version = bare_version("1.2.3");
    assert_eq!("1.2.3           ", format!("{:16}", version));
    assert_eq!("*****1.2.3******", format!("{:*^16}", version));
    assert_eq!("           1.2.3", format!("{:>16}", version));
}
