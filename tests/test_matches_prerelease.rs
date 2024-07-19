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
    let ref r = req("0.24");
    // Match >= 0.24.0, < 0.25.0-0
    assert_prerelease_match_all(r, &["0.24.0", "0.24.1-0", "0.24.1-pre", "0.24.1"]);
    // Not Match < 0.24.0
    assert_prerelease_match_none(r, &["0.1.0", "0.8.0-pre"]);
    // Not Match >= 0.25.0-0
    assert_prerelease_match_none(r, &["0.25.0-0", "0.25.0", "0.25.8", "2.0.0-0"]);

    let ref r = req("0.8.0");
    // Match >= 0.8.0, < 0.9.0-0
    assert_prerelease_match_all(r, &["0.8.0", "0.8.1-0", "0.8.1-pre", "0.8.1"]);
    // Not Match <0.8.0
    assert_prerelease_match_none(r, &["0.1.0", "0.8.0-pre"]);
    // Not Match >=0.9.0-0
    assert_prerelease_match_none(r, &["0.9.0-0", "1.0.0-pre", "2.0.0-0"]);

    let ref r = req("0.8.0-0");
    // Match >= 0.8.0-0, < 0.9.0-0
    assert_prerelease_match_all(r, &["0.8.0-0", "0.8.0", "0.8.1-pre", "0.8.1"]);
    // Not Match < 0.8.0-0
    assert_prerelease_match_none(r, &["0.1.0", "0.7.7-pre", "0.7.7"]);
    // Not Match >= 0.9.0-0
    assert_prerelease_match_none(r, &["0.9.0-0", "0.9.0", "1.0.0-pre", "2.0.0-0"]);
}

#[test]
fn major_caret() {
    let ref r = req("0");
    // Match >= 0.0.0, < 1.0.0-0
    assert_prerelease_match_all(r, &["0.0.0", "0.0.1-0", "0.0.1-pre", "0.1.1"]);
    // Not Match < 0.0.0
    assert_prerelease_match_none(r, &["0.0.0-pre"]);
    // Not Match >= 1.0.0-0
    assert_prerelease_match_none(r, &["1.0.0-0", "1.0.0", "1.0.1-pre", "2.0.0-0"]);

    let ref r = req("0.0");
    // Match >= 0.0.0, < 0.1.0-0
    assert_prerelease_match_all(r, &["0.0.1-z0", "0.0.9"]);
    // Not Match >= 0.1.0-0
    assert_prerelease_match_none(r, &["0.1.0-0", "0.1.0", "0.1.1-pre", "0.1.1-z0", "1.1.0"]);

    let ref r = req("0.0.0");
    // Match >= 0.0.0, < 0.0.1-0
    assert_prerelease_match_all(r, &["0.0.0"]);
    // Not Match >= 0.0.1-0
    assert_prerelease_match_none(r, &["0.0.1-0", "0.0.1", "1.0.1-pre"]);

    let ref r = req("1");
    // Match >= 1.0.0, < 2.0.0-0
    assert_prerelease_match_all(r, &["1.2.3", "1.2.4-0", "1.2.4", "1.8.8"]);
    // Not Match < 1.0.0
    assert_prerelease_match_none(r, &["0.0.8", "0.9.0", "1.0.0-pre"]);
    // Not Match >= 2.0.0-0
    assert_prerelease_match_none(r, &["2.0.0-0", "2.0.0", "2.0.1"]);

    let ref r = req("1.2");
    // Match >= 1.2.0, < 1.3.0-0
    assert_prerelease_match_all(r, &["1.2.3", "1.2.4-0", "1.2.4", "1.8.8"]);
    // Not Match < 1.2.0
    assert_prerelease_match_none(r, &["0.0.8", "0.9.0", "1.0.0-pre"]);
    // Not Match >= 1.3.0-0
    assert_prerelease_match_none(r, &["2.0.0-0", "2.0.0", "2.0.1"]);

    let ref r = req("1.2.3");
    // Match >=1.2.3, < 2.0.0-0
    assert_prerelease_match_all(r, &["1.2.3", "1.2.4-0", "1.2.4", "1.8.8"]);
    // Not Match < 1.2.3
    assert_prerelease_match_none(r, &["0.8.8", "1.2.0", "1.2.3-pre"]);
    // Not Match >= 2.0.0-0
    assert_prerelease_match_none(r, &["2.0.0-0", "2.0.0", "2.0.1"]);

    let ref r = req("1.2.3-0");
    // Match >= 1.2.3-0, < 2.0.0-0
    assert_prerelease_match_all(r, &["1.2.3-pre", "1.2.3", "1.2.4-0", "1.2.4"]);
    // Not Match < 1.2.3-0, >= 2.0.0-0
    assert_prerelease_match_none(r, &["1.2.0", "2.0.0-0", "2.0.0"]);
}

#[test]
fn test_exact() {
    // =I.J.K equivalent to >=I.J.K, <I.J.(K+1)-0
    let ref r = req("=4.2.1");
    assert_prerelease_match_all(r, &["4.2.1"]);
    assert_prerelease_match_none(r, &["1.2.3", "4.2.1-pre", "4.2.2", "5.0.0"]);

    // =I.J equivalent to >=I.J.0, <I.(J+1).0-0
    let ref r = req("=4.2");
    // Match >= 4.2.0, < 4.3.0-0
    assert_prerelease_match_all(r, &["4.2.0", "4.2.1", "4.2.4-pre", "4.2.9"]);
    // Not Match < 4.2.0
    assert_prerelease_match_none(r, &["0.0.1", "2.1.2-pre", "4.0.0-pre"]);
    // Not Match >= 4.3.0-0
    assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "5.0.0-0", "5.0.0"]);

    // =I equivalent to >=I.0.0, <(I+1).0.0-0
    let ref r = req("=4");
    // Match >= 4.0.0, < 5.0.0-0
    assert_prerelease_match_all(r, &["4.0.0", "4.2.1", "4.2.4-pre", "4.9.9"]);
    // Not Match < 4.0.0
    assert_prerelease_match_none(r, &["0.0.1", "2.1.2-pre", "4.0.0-pre"]);
    // Not Match >= 5.0.0-0
    assert_prerelease_match_none(r, &["5.0.0-0", "5.0.0", "5.0.1"]);

    // =I.J.K-pre only match I.J.K-pre
    let ref r = req("=4.2.1-0");
    // Only exactly match 4.2.1-0
    assert_prerelease_match_all(r, &["4.2.1-0"]);
    // Not match others
    assert_prerelease_match_none(r, &["1.2.3", "4.2.0", "4.2.1-1", "4.2.2", "4.3.5"]);
}

#[test]
fn test_greater() {
    // >I.J.K
    let ref r = req(">4.2.1");
    assert_prerelease_match_all(r, &["4.2.2", "5.0.0-0", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1-pre", "4.2.1"]);
    // >I.J equivalent to >=I.(J+1).0-0
    let ref r = req(">4.2");
    assert_prerelease_match_all(r, &["4.3.0-pre", "4.3.0", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1"]);
    // >I equivalent to >=(I+1).0.0-0
    let ref r = req(">4");
    assert_prerelease_match_all(r, &["5.0.0-0", "5.0.0-1", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1"]);
}

#[test]
fn test_greater_eq() {
    // >=I.J.K
    let ref r = req(">=4.2.1");
    assert_prerelease_match_all(r, &["4.2.1", "5.0.0-0", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1-pre"]);
    // >=I.J equivalent to >=I.J.0
    let ref r = req(">=4.2");
    assert_prerelease_match_all(r, &["4.2.0", "4.2.1-pre", "4.3.0", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.1.1", "4.2.0-0"]);
    // >=I equivalent to >=I.0.0
    let ref r = req(">=4");
    assert_prerelease_match_all(r, &["4.0.0", "4.1.0-1", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.0.0-pre"]);
}

#[test]
fn test_less() {
    // <I.J.K
    let ref r = req("<4.2.1");
    assert_prerelease_match_all(r, &["0.0.0", "4.0.0", "4.2.1-pre"]);
    assert_prerelease_match_none(r, &["4.2.2", "5.0.0-0", "5.0.0"]);
    // <I.J equivalent to <I.J.0
    let ref r = req("<4.2");
    assert_prerelease_match_all(r, &["0.0.0", "4.2.0-pre"]);
    assert_prerelease_match_none(r, &["4.2.0", "4.3.0-pre", "4.3.0"]);
    // <I equivalent to <I.0.0
    let ref r = req("<4");
    assert_prerelease_match_all(r, &["0.0.0", "4.0.0-pre"]);
    assert_prerelease_match_none(r, &["4.0.0", "5.0.0-1", "5.0.0"]);
}

#[test]
fn test_less_eq() {
    // <=I.J.K
    let ref r = req("<=4.2.1");
    assert_prerelease_match_all(r, &["0.0.0", "4.2.1-pre", "4.2.1"]);
    assert_prerelease_match_none(r, &["4.2.2", "5.0.0-0", "5.0.0"]);
    // <=I.J equivalent to <I.(J+1).0-0
    let ref r = req("<=4.2");
    assert_prerelease_match_all(r, &["0.0.0", "4.2.0-pre"]);
    assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "4.4.0"]);
    // <=I equivalent to <(I+1).0.0-0
    let ref r = req("<=4");
    assert_prerelease_match_all(r, &["0.0.0", "4.0.0-pre", "4.0.0"]);
    assert_prerelease_match_none(r, &["5.0.0-1", "5.0.0"]);
}

#[test]
fn test_tilde() {
    // ~I.J.K — equivalent to >=I.J.K, <I.(J+1).0-0
    let ref r = req("~1.2.3");
    // Match >= 1.2.3, < 1.3.0-0
    assert_prerelease_match_all(r, &["1.2.3", "1.2.4-pre", "1.2.4"]);
    // Not Match < 1.2.3
    assert_prerelease_match_none(r, &["0.0.1", "1.0.0-pre", "1.1.0-0", "1.1.0"]);
    // Not Match >= 1.3.0-0
    assert_prerelease_match_none(r, &["1.3.0-0", "1.3.0", "1.3.1-0", "1.3.1", "2.0.0"]);

    // ~I.J — equivalent to >=I.J.0, <I.(J+1).0-0
    let ref r = req("~0.24");
    // Match >= 0.24.0, < 0.25.0-0
    assert_prerelease_match_all(r, &["0.24.0", "0.24.1-pre", "0.24.1", "0.24.9"]);
    // Not Match < 0.24.0
    assert_prerelease_match_none(r, &["0.0.1", "0.9.9", "0.24.0-pre"]);
    // Not Match >= 0.25.0-0
    assert_prerelease_match_none(r, &["0.25.0-0", "1.1.0", "1.2.3", "2.0.0"]);

    // ~I — >=I.0.0, <(I+1).0.0-0
    let ref r = req("~1");
    // Match >= 1.0.0, < 2.0.0-0
    assert_prerelease_match_all(r, &["1.0.0", "1.1.0-0", "1.1.0", "1.2.3"]);
    // Not Match < 1.0.0
    assert_prerelease_match_none(r, &["0.0.1", "0.9.9", "1.0.0-pre"]);
    // Not Match >= 2.0.0-0
    assert_prerelease_match_none(r, &["2.0.0-0", "2.0.0", "2.0.1"]);

    // ~I.J.K-[pre] — equivalent to >=I.J.K-[pre], <I.J.(K+1).0
    let ref r = req("~1.2.3-0");
    // Match >= 1.2.3-0, < 1.3.0-0
    assert_prerelease_match_all(r, &["1.2.3-0", "1.2.3", "1.2.4-pre", "1.2.4"]);
    // Not Match < 1.2.3-0
    assert_prerelease_match_none(r, &["0.0.1", "1.0.0-pre", "1.1.0-0", "1.1.0"]);
    // Not Match >= 1.3.0-0
    assert_prerelease_match_none(r, &["1.3.0-0", "1.3.0", "1.3.1-0", "1.3.1", "2.0.0"]);
}

#[test]
fn test_caret() {
    // ^I.J.K (for I>0) — equivalent to >=I.J.K, <(I+1).0.0-0
    let ref r = req("^1.2.3");
    assert_prerelease_match_all(r, &["1.2.3", "1.2.4-0", "1.8.9"]);
    assert_prerelease_match_none(r, &["0.0.9", "1.1.1-0", "1.2.3-0", "2.0.0-pre", "2.1.1"]);

    // ^0.J.K (for J>0) — equivalent to >=0.J.K, <0.(J+1).0-0
    let ref r = req("^0.2.3");
    assert_prerelease_match_all(r, &["0.2.3", "0.2.9-0", "0.2.9"]);
    assert_prerelease_match_none(r, &["0.0.9", "0.2.3-0", "0.3.0-0", "0.3.11", "1.1.1"]);

    // ^0.0.K — equivalent to >=0.0.K, <0.0.(K+1)-0
    let ref r = req("^0.0.3");
    assert_prerelease_match_all(r, &["0.0.3"]);
    assert_prerelease_match_none(
        r,
        &["0.0.1", "0.0.4-0", "0.0.9", "0.3.0-0", "0.4.0-0", "1.1.1"],
    );

    // ^I.J (for I>0 or J>0) — equivalent to >=I.J.0, <(I+1).0.0-0)
    let ref r = req("^1.2");
    assert_prerelease_match_all(r, &["1.2.0", "1.9.0-0", "1.9.9"]);
    assert_prerelease_match_none(r, &["0.0.1", "0.0.4-0", "1.2.0-0", "2.0.0-0", "4.0.1"]);

    // ^0.0 — equivalent to >=0.0.0, <0.1.0-0
    let ref r = req("^0.0");
    assert_prerelease_match_all(r, &["0.0.0", "0.0.1", "0.0.4-0"]);
    assert_prerelease_match_none(r, &["0.1.0-0", "0.1.0", "1.1.1"]);

    // ^I — equivalent to >=I.0.0, <(I+1).0.0-0
    let ref r = req("^1");
    assert_prerelease_match_all(r, &["1.0.0", "1.0.1", "1.0.4-0"]);
    assert_prerelease_match_none(r, &["0.1.0-0", "0.1.0", "2.0.0-0", "3.1.2"]);
}

#[test]
fn test_wildcard() {
    // I.J.* — equivalent to =I.J
    //
    // =I.J equivalent to >=I.J.0, <I.(J+1).0-0
    let ref r = req("4.2.*");
    // Match >= 4.2.0, < 4.3.0-0
    assert_prerelease_match_all(r, &["4.2.0", "4.2.1", "4.2.4-pre", "4.2.9"]);
    // Not Match < 4.2.0
    assert_prerelease_match_none(r, &["0.0.1", "2.1.2-pre", "4.0.0-pre"]);
    // Not Match >= 4.3.0-0
    assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "5.0.0-0", "5.0.0", "5.0.1"]);

    // I.* or I.*.* — equivalent to =I
    //
    // =I equivalent to >=I.0.0, <(I+1).0.0-0
    for r in &[req("4.*"), req("4.*.*")] {
        // Match >= 4.0.0, < 5.0.0-0
        assert_prerelease_match_all(r, &["4.0.0", "4.2.1", "4.2.4-pre", "4.9.9"]);
        // Not Match < 4.0.0
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-pre", "4.0.0-pre"]);
        // Not Match >= 5.0.0-0
        assert_prerelease_match_none(r, &["5.0.0-0", "5.0.0", "5.0.1"]);
    }
}