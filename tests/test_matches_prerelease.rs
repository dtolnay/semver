#![cfg(not(test_node_semver))]
#![allow(
    clippy::missing_panics_doc,
    clippy::shadow_unrelated,
    clippy::toplevel_ref_arg,
    clippy::wildcard_imports
)]

mod util;

use crate::util::*;

use semver::VersionReq;
#[cfg_attr(not(no_track_caller), track_caller)]
fn assert_prerelease_match_all(req: &VersionReq, versions: &[&str]) {
    for string in versions {
        let parsed = version(string);
        assert!(req.matches_prerelease(&parsed), "did not match {}", string);
    }
}

#[cfg_attr(not(no_track_caller), track_caller)]
fn assert_prerelease_match_none(req: &VersionReq, versions: &[&str]) {
    for string in versions {
        let parsed = version(string);
        assert!(!req.matches_prerelease(&parsed), "matched {}", string);
    }
}

#[test]
fn patch_caret() {
    let ref r = req("0.0.7");
    // Match >=0.0.7, <0.0.8-0, that's only match 0.0.7
    assert_prerelease_match_all(r, &["0.0.7"]);
    // Not Match <0.7.0
    assert_prerelease_match_none(r, &["0.0.6", "0.0.7-0", "0.0.7-pre"]);
    // Not Match >=0.8.0-0
    assert_prerelease_match_none(r, &["0.0.8-0", "0.0.8-pre.1", "0.0.8"]);

    let ref r = req("0.0.7-0");
    // Match >=0.0.7-0, <0.0.8-0
    assert_prerelease_match_all(r, &["0.0.7-0", "0.0.7-pre", "0.0.7"]);
    // Not Match <0.7.0-0
    assert_prerelease_match_none(r, &["0.0.6-0", "0.0.6-pre", "0.0.6"]);
    // Not Match >=0.0.8-0
    assert_prerelease_match_none(r, &["0.0.8-0", "0.0.8-pre.1", "0.0.8"]);
}

#[test]
fn minor_caret() {
    let ref r = req("0.8.0");
    // Match >=0.8.0, <0.9.0-0
    assert_prerelease_match_all(r, &["0.8.0", "0.8.1-0", "0.8.1-pre", "0.8.1"]);
    // Not Match <0.8.0
    assert_prerelease_match_none(r, &["0.1.0", "0.8.0-pre"]);
    // Not Match >=0.9.0

    let ref r = req("0.8.0-0");
    // Match >=0.8.0-0, <0.9.0-0
    assert_prerelease_match_all(r, &["0.8.0-0", "0.8.0", "0.8.1-pre", "0.8.1"]);
    // Not Match <0.8.0-0
    assert_prerelease_match_none(r, &["0.1.0", "0.7.7-pre", "0.7.7"]);
    // Not Match >=0.9.0
    assert_prerelease_match_none(r, &["0.9.0-0", "0.9.0", "1.0.0-pre", "2.0.0-0"]);
}

#[test]
fn major_caret() {
    let ref r = req("1.2.3");
    // Match >=1.2.3, <2.0.0-0
    assert_prerelease_match_all(r, &["1.2.3", "1.2.4-0", "1.2.4", "1.8.8"]);
    // Not Match < 1.2.3
    assert_prerelease_match_none(r, &["0.8.8", "1.2.0", "1.2.3-pre"]);
    // Not Match >= 2.0.0-0
    assert_prerelease_match_none(r, &["2.0.0-0", "2.0.0", "2.0.1"]);

    let ref r = req("1.2.3-0");
    // Match >=1.2.3-0, <2.0.0-0
    assert_prerelease_match_all(r, &["1.2.3-pre", "1.2.3", "1.2.4-0", "1.2.4"]);
    // Not Match < 1.2.3-0, >= 2.0.0-0
    assert_prerelease_match_none(r, &["1.2.0", "2.0.0-0", "2.0.0"]);
}

#[test]
fn test_non_caret() {
    let ref r: VersionReq = req(">=1.0.0");
    assert_prerelease_match_all(r, &["1.0.0", "1.0.1-pre", "2.0.0-0", "2.0.0"]);
    assert_prerelease_match_none(r, &["0.9.9", "0.10.0", "1.0.0-pre.0"]);

    let ref r: VersionReq = req(">=1.0.0-pre.1");
    assert_prerelease_match_all(r, &["1.0.0-pre.1", "1.0.0", "1.0.1-pre", "2.0.0-0"]);
    assert_prerelease_match_none(r, &["0.9.9", "0.10.0", "1.0.0-pre.0"]);

    let ref r: VersionReq = req("<=1.0.0");
    assert_prerelease_match_all(r, &["0.9.9", "0.10.0", "1.0.0-pre", "1.0.0"]);
    assert_prerelease_match_none(r, &["1.0.1", "1.0.1-pre", "1.1.0", "2.0.0-0"]);

    let ref r: VersionReq = req("<=1.0.0-0");
    assert_prerelease_match_all(r, &["0.9.9", "0.1.0", "0.10.0", "1.0.0-0"]);
    assert_prerelease_match_none(r, &["1.0.0-pre", "1.0.1-pre", "1.1.0", "2.0.0-0"]);

    let ref r: VersionReq = req(">1.0.0-0,<=1.1.0-0");
    assert_prerelease_match_all(r, &["1.0.0-pre", "1.0.0", "1.0.9", "1.1.0-0"]);
    assert_prerelease_match_none(r, &["0.0.1", "1.0.0-0", "1.2.3-pre", "2.0.0"]);

    let ref r: VersionReq = req(">=1.0.0,<1.1.0-0");
    assert_prerelease_match_all(r, &["1.0.0", "1.0.0", "1.0.9"]);
    assert_prerelease_match_none(r, &["0.0.1", "1.0.0-pre", "1.1.0-0", "1.1.0", "2.0.0"]);
}
