#![allow(
    clippy::missing_panics_doc,
    clippy::shadow_unrelated,
    clippy::toplevel_ref_arg,
    clippy::wildcard_imports
)]

mod node;
mod util;
use crate::util::*;
#[cfg(test_node_semver)]
use node::{req, VersionReq};
#[cfg(not(test_node_semver))]
use semver::VersionReq;
#[cfg_attr(not(no_track_caller), track_caller)]
fn assert_prerelease_match_all(req: &VersionReq, versions: &[&str]) {
    for string in versions {
        let parsed = version(string);
        assert!(
            req.matches_prerelease(&parsed),
            "{} did not match {}",
            req,
            string,
        );
    }
}

#[cfg_attr(not(no_track_caller), track_caller)]
fn assert_prerelease_match_none(req: &VersionReq, versions: &[&str]) {
    for string in versions {
        let parsed = version(string);
        assert!(
            !req.matches_prerelease(&parsed),
            "{} matched {}",
            req,
            string
        );
    }
}

#[test]
#[cfg(not(any(feature = "mirror_node_matches_prerelease", test_node_semver)))]
fn test_exact() {
    // =I.J.K-pre only match I.J.K-pre
    let ref r = req("=4.2.1-0");
    // Only exactly match 4.2.1-0
    assert_prerelease_match_all(r, &["4.2.1-0"]);
    // Not match others
    assert_prerelease_match_none(r, &["1.2.3", "4.2.0", "4.2.1-1", "4.2.2"]);

    // =I.J.K equivalent to >=I.J.K, <I.J.(K+1)-0
    for r in &[req("=4.2.1"), req(">=4.2.1, <4.2.2-0")] {
        assert_prerelease_match_all(r, &["4.2.1"]);
        assert_prerelease_match_none(r, &["1.2.3", "4.2.1-0", "4.2.2-0", "4.2.2"]);
    }

    // =I.J equivalent to >=I.J.0, <I.(J+1).0-0
    for r in &[req("=4.2"), req(">=4.2.0, <4.3.0-0")] {
        assert_prerelease_match_all(r, &["4.2.0", "4.2.1", "4.2.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0", "4.2.0-0"]);
        assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "5.0.0-0", "5.0.0"]);
    }

    // =I equivalent to >=I.0.0, <(I+1).0.0-0
    for r in &[req("=4"), req(">=4.0.0, <5.0.0-0")] {
        assert_prerelease_match_all(r, &["4.0.0", "4.2.1", "4.2.4-0", "4.9.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0", "4.0.0-0"]);
        assert_prerelease_match_none(r, &["5.0.0-0", "5.0.0", "5.0.1"]);
    }
}

#[test]
#[cfg(not(any(feature = "mirror_node_matches_prerelease", test_node_semver)))]
fn test_greater_eq() {
    // >=I.J.K-0
    let ref r = req(">=4.2.1-0");
    assert_prerelease_match_all(r, &["4.2.1-0", "4.2.1", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "1.2.3"]);

    // >=I.J.K
    let ref r = req(">=4.2.1");
    assert_prerelease_match_all(r, &["4.2.1", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1-0"]);

    // >=I.J equivalent to >=I.J.0
    for r in &[req(">=4.2"), req(">=4.2.0")] {
        assert_prerelease_match_all(r, &["4.2.1-0", "4.2.0", "4.3.0"]);
        assert_prerelease_match_none(r, &["0.0.0", "4.1.1", "4.2.0-0"]);
    }

    // >=I equivalent to >=I.0.0
    for r in &[req(">=4"), req(">=4.0.0")] {
        assert_prerelease_match_all(r, &["4.0.0", "4.1.0-1", "5.0.0"]);
        assert_prerelease_match_none(r, &["0.0.0", "1.2.3", "4.0.0-0"]);
    }
}

#[test]
#[cfg(not(any(feature = "mirror_node_matches_prerelease", test_node_semver)))]
fn test_less() {
    // <I.J.K-0
    let ref r = req("<4.2.1-0");
    assert_prerelease_match_all(r, &["0.0.0", "4.0.0"]);
    assert_prerelease_match_none(r, &["4.2.1-0", "4.2.2", "5.0.0-0", "5.0.0"]);

    // <I.J.K
    let ref r = req("<4.2.1");
    assert_prerelease_match_all(r, &["0.0.0", "4.0.0", "4.2.1-0"]);
    assert_prerelease_match_none(r, &["4.2.2", "5.0.0-0", "5.0.0"]);

    // <I.J equivalent to <I.J.0
    for r in &[req("<4.2"), req("<4.2.0")] {
        assert_prerelease_match_all(r, &["0.0.0", "4.1.0", "4.2.0-0"]);
        assert_prerelease_match_none(r, &["4.2.0", "4.3.0-0", "4.3.0"]);
    }

    // <I equivalent to <I.0.0
    for r in &[req("<4"), req("<4.0.0")] {
        assert_prerelease_match_all(r, &["0.0.0", "4.0.0-0"]);
        assert_prerelease_match_none(r, &["4.0.0", "5.0.0-1", "5.0.0"]);
    }
}

#[test]
#[cfg(not(any(feature = "mirror_node_matches_prerelease", test_node_semver)))]
fn test_caret() {
    // ^I.J.K.0 (for I>0) — equivalent to >=I.J.K-0, <(I+1).0.0-0
    for r in &[req("^1.2.3-0"), req(">=1.2.3-0, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.2.3-0", "1.2.3-1", "1.2.3", "1.9.9"]);
        assert_prerelease_match_none(r, &["0.0.9", "1.1.1-0", "2.0.0-0", "2.1.1"]);
    }

    // ^I.J.K (for I>0) — equivalent to >=I.J.K, <(I+1).0.0-0
    for r in &[req("^1.2.3"), req(">=1.2.3, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.2.3", "1.9.9"]);
        assert_prerelease_match_none(
            r,
            &["0.0.9", "1.1.1-0", "1.2.3-0", "1.2.3-1", "2.0.0-0", "2.1.1"],
        );
    }

    // ^0.J.K-0 (for J>0) — equivalent to >=0.J.K-0, <0.(J+1).0-0
    for r in &[req("^0.2.3-0"), req(">=0.2.3-0, <0.3.0-0")] {
        assert_prerelease_match_all(r, &["0.2.3-0", "0.2.3", "0.2.9-0", "0.2.9"]);
        assert_prerelease_match_none(r, &["0.0.9", "0.3.0-0", "0.3.11", "1.1.1"]);
    }

    // ^0.J.K (for J>0) — equivalent to >=0.J.K-0, <0.(J+1).0-0
    for r in &[req("^0.2.3"), req(">=0.2.3, <0.3.0-0")] {
        assert_prerelease_match_all(r, &["0.2.3", "0.2.9-0", "0.2.9"]);
        assert_prerelease_match_none(r, &["0.0.9", "0.2.3-0", "0.3.0-0", "0.3.11", "1.1.1"]);
    }

    // ^0.0.K-0 — equivalent to >=0.0.K-0, <0.0.(K+1)-0
    for r in &[req("^0.0.3-0"), req(">=0.0.3-0, <0.1.0-0")] {
        assert_prerelease_match_all(r, &["0.0.3-0", "0.0.3-1", "0.0.3"]);
        assert_prerelease_match_none(r, &["0.0.1", "0.3.0-0", "0.4.0-0", "1.1.1"]);
    }

    // ^0.0.K — equivalent to >=0.0.K, <0.0.(K+1)-0
    for r in &[req("^0.0.3"), req(">=0.0.3, <0.1.0-0")] {
        assert_prerelease_match_all(r, &["0.0.3"]);
        assert_prerelease_match_none(
            r,
            &["0.0.1", "0.0.3-0", "0.3.0-0", "0.0.3-1", "0.4.0-0", "1.1.1"],
        );
    }

    // ^I.J (for I>0 or J>0) — equivalent to >=I.J.0, <(I+1).0.0-0)
    for r in &[req("^1.2"), req(">=1.2.0, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.2.0", "1.9.0-0", "1.9.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "0.0.4-0", "1.2.0-0", "2.0.0-0", "4.0.1"]);
    }

    // ^0.0 — equivalent to >=0.0.0, <0.1.0-0
    for r in &[req("^0.0"), req(">=0.0.0, <0.1.0-0")] {
        assert_prerelease_match_all(r, &["0.0.0", "0.0.1", "0.0.4-0"]);
        assert_prerelease_match_none(r, &["0.0.0-0", "0.1.0-0", "0.1.0", "1.1.1"]);
    }

    // ^I — equivalent to >=I.0.0, <(I+1).0.0-0
    for r in &[req("^1"), req(">=1.0.0, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.0.0", "1.0.1"]);
        assert_prerelease_match_none(r, &["0.1.0-0", "0.1.0", "1.0.0-0", "2.0.0-0", "3.1.2"]);
    }
}

#[test]
#[cfg(not(any(feature = "mirror_node_matches_prerelease", test_node_semver)))]
fn test_wildcard() {
    // I.J.* — equivalent to =I.J
    //
    // =I.J equivalent to >=I.J.0, <I.(J+1).0-0
    for r in &[req("4.2.*"), req("=4.2")] {
        // Match >= 4.2.0, < 4.3.0-0
        assert_prerelease_match_all(r, &["4.2.0", "4.2.1", "4.2.9"]);
        // Not Match < 4.2.0
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0", "4.2.0-0"]);
        // Not Match >= 4.3.0-0
        assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "5.0.0", "5.0.1"]);
    }

    // I.* or I.*.* — equivalent to =I
    //
    // =I equivalent to >=I.0.0, <(I+1).0.0-0
    for r in &[req("4.*"), req("4.*.*"), req("=4")] {
        // Match >= 4.0.0, < 5.0.0-0
        assert_prerelease_match_all(r, &["4.0.0", "4.2.1", "4.9.9"]);
        // Not Match < 4.0.0
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0", "4.0.0-0"]);
        // Not Match >= 5.0.0-0
        assert_prerelease_match_none(r, &["5.0.0-0", "5.0.0", "5.0.1"]);
    }
}

//
// These tests below can pass in both implementations
//

#[test]
fn test_greater() {
    // >I.J.K-0
    let ref r = req(">4.2.1-0");
    assert_prerelease_match_all(r, &["4.2.1", "4.2.2", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1-0"]);

    // >I.J.K
    let ref r = req(">4.2.1");
    assert_prerelease_match_all(r, &["4.2.2", "5.0.0-0", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1-0", "4.2.1"]);

    // >I.J equivalent to >=I.(J+1).0-0
    for r in &[req(">4.2"), req(">=4.3.0-0")] {
        assert_prerelease_match_all(r, &["4.3.0-0", "4.3.0", "5.0.0"]);
        assert_prerelease_match_none(r, &["0.0.0", "4.2.1"]);
    }

    // >I equivalent to >=(I+1).0.0-0
    for r in &[req(">4"), req(">=5.0.0-0")] {
        assert_prerelease_match_all(r, &["5.0.0-0", "5.0.0"]);
        assert_prerelease_match_none(r, &["0.0.0", "4.2.1"]);
    }
}

#[test]
fn test_less_eq() {
    // <=I.J.K
    let ref r = req("<=4.2.1");
    assert_prerelease_match_all(r, &["0.0.0", "4.2.1-0", "4.2.1"]);
    assert_prerelease_match_none(r, &["4.2.2", "5.0.0-0", "5.0.0"]);
    // <=I.J.K-0
    let ref r = req("<=4.2.1-0");
    assert_prerelease_match_all(r, &["0.0.0", "4.2.1-0"]);
    assert_prerelease_match_none(r, &["4.2.1", "4.2.2", "5.0.0-0", "5.0.0"]);

    // <=I.J equivalent to <I.(J+1).0-0
    for r in &[req("<=4.2"), req("<4.3.0-0")] {
        assert_prerelease_match_all(r, &["0.0.0", "4.2.0-0"]);
        assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "4.4.0"]);
    }

    // <=I equivalent to <(I+1).0.0-0
    for r in &[req("<=4"), req("<5.0.0-0")] {
        assert_prerelease_match_all(r, &["0.0.0", "4.0.0-0", "4.0.0"]);
        assert_prerelease_match_none(r, &["5.0.0-1", "5.0.0"]);
    }
}

#[test]
fn test_tilde() {
    // ~I.J.K-0 — equivalent to >=I.J.K-0, <I.(J+1).0-0
    for r in &[req("~1.2.3-0"), req(">= 1.2.3-0, < 1.3.0-0")] {
        assert_prerelease_match_all(r, &["1.2.3-0", "1.2.3", "1.2.4-0", "1.2.4"]);
        assert_prerelease_match_none(r, &["0.0.1", "1.1.0-0"]);
        assert_prerelease_match_none(r, &["1.3.0-0", "1.3.0", "1.3.1", "2.0.0"]);
    }

    // ~I.J.K — equivalent to >=I.J.K, <I.(J+1).0-0
    for r in &[req("~1.2.3"), req(">= 1.2.3, < 1.3.0-0")] {
        assert_prerelease_match_all(r, &["1.2.3", "1.2.4-0", "1.2.4"]);
        assert_prerelease_match_none(r, &["0.0.1", "1.1.0-0", "1.2.3-0"]);
        assert_prerelease_match_none(r, &["1.3.0-0", "1.3.0", "1.3.1", "2.0.0"]);
    }

    // ~I.J — equivalent to >=I.J.0, <I.(J+1).0-0
    for r in &[req("~0.24"), req(">=0.24.0, <0.25.0-0")] {
        assert_prerelease_match_all(r, &["0.24.0", "0.24.1-0", "0.24.1", "0.24.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "0.9.9", "0.24.0-0"]);
        assert_prerelease_match_none(r, &["0.25.0-0", "1.1.0", "1.2.3", "2.0.0"]);
    }

    // ~I — >=I.0.0, <(I+1).0.0-0
    for r in &[req("~1"), req(">=1.0.0, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.0.0", "1.1.0-0", "1.1.0"]);
        assert_prerelease_match_none(r, &["0.0.1", "0.9.9", "1.0.0-0"]);
        assert_prerelease_match_none(r, &["2.0.0-0", "2.0.0", "2.0.1"]);
    }
}

//
// These tests below are for node semver compatibility test. (with includePrerelease=true, see https://github.com/npm/node-semver?tab=readme-ov-file#functions)
//

#[test]
#[cfg(any(feature = "mirror_node_matches_prerelease", test_node_semver))]
fn test_exact() {
    // =I.J.K-pre only match I.J.K-pre
    let ref r = req("=4.2.1-0");
    // Only exactly match 4.2.1-0
    assert_prerelease_match_all(r, &["4.2.1-0"]);
    // Not match others
    assert_prerelease_match_none(r, &["1.2.3", "4.2.0", "4.2.1-1", "4.2.2"]);

    // =I.J.K equivalent to >=I.J.K, <I.J.(K+1)-0
    for r in &[req("=4.2.1"), req(">=4.2.1, <4.2.2-0")] {
        assert_prerelease_match_all(r, &["4.2.1"]);
        assert_prerelease_match_none(r, &["1.2.3", "4.2.1-0", "4.2.2-0", "4.2.2"]);
    }

    // =I.J equivalent to >=I.J.0-0, <I.(J+1).0-0
    for r in &[req("=4.2"), req(">=4.2.0-0, <4.3.0-0")] {
        assert_prerelease_match_all(r, &["4.2.0-0", "4.2.0", "4.2.1", "4.2.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0"]);
        assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "5.0.0-0", "5.0.0"]);
    }

    // =I equivalent to >=I.0.0-0, <(I+1).0.0-0
    for r in &[req("=4"), req(">=4.0.0-0, <5.0.0-0")] {
        assert_prerelease_match_all(r, &["4.0.0-0", "4.0.0", "4.2.1", "4.2.4-0", "4.9.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0"]);
        assert_prerelease_match_none(r, &["5.0.0-0", "5.0.0", "5.0.1"]);
    }
}

#[test]
#[cfg(any(feature = "mirror_node_matches_prerelease", test_node_semver))]
fn test_greater_eq() {
    // >=I.J.K-0
    let ref r = req(">=4.2.1-0");
    assert_prerelease_match_all(r, &["4.2.1-0", "4.2.1", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "1.2.3"]);

    // >=I.J.K
    let ref r = req(">=4.2.1");
    assert_prerelease_match_all(r, &["4.2.1", "5.0.0"]);
    assert_prerelease_match_none(r, &["0.0.0", "4.2.1-0"]);

    // >=I.J equivalent to >=I.J.0-0
    for r in &[req(">=4.2"), req(">=4.2.0-0")] {
        assert_prerelease_match_all(r, &["4.2.0-0", "4.2.0", "4.2.1-0", "4.3.0"]);
        assert_prerelease_match_none(r, &["0.0.0", "4.1.1"]);
    }

    // >=I equivalent to >=I.0.0-0
    for r in &[req(">=4"), req(">=4.0.0-0")] {
        assert_prerelease_match_all(r, &["4.0.0-0", "4.0.0", "4.1.0-1", "5.0.0"]);
        assert_prerelease_match_none(r, &["0.0.0", "1.2.3"]);
    }
}

#[test]
#[cfg(any(feature = "mirror_node_matches_prerelease", test_node_semver))]
fn test_less() {
    // <I.J.K-0
    let ref r = req("<4.2.1-0");
    assert_prerelease_match_all(r, &["0.0.0", "4.0.0"]);
    assert_prerelease_match_none(r, &["4.2.1-0", "4.2.2", "5.0.0-0", "5.0.0"]);

    // <I.J.K
    let ref r = req("<4.2.1");
    assert_prerelease_match_all(r, &["0.0.0", "4.0.0", "4.2.1-0"]);
    assert_prerelease_match_none(r, &["4.2.2", "5.0.0-0", "5.0.0"]);

    // <I.J equivalent to <I.J.0-0
    for r in &[req("<4.2"), req("<4.2.0-0")] {
        assert_prerelease_match_all(r, &["0.0.0", "4.1.0"]);
        assert_prerelease_match_none(r, &["4.2.0-0", "4.2.0", "4.3.0-0", "4.3.0"]);
    }

    // <I equivalent to <I.0.0-0
    for r in &[req("<4"), req("<4.0.0-0")] {
        assert_prerelease_match_all(r, &["0.0.0"]);
        assert_prerelease_match_none(r, &["4.0.0-0", "4.0.0", "5.0.0-1", "5.0.0"]);
    }
}

#[test]
#[cfg(any(feature = "mirror_node_matches_prerelease", test_node_semver))]
fn test_caret() {
    // ^I.J.K.0 (for I>0) — equivalent to >=I.J.K-0, <(I+1).0.0-0
    for r in &[req("^1.2.3-0"), req(">=1.2.3-0, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.2.3-0", "1.2.3-1", "1.2.3", "1.9.9"]);
        assert_prerelease_match_none(r, &["0.0.9", "1.1.1-0", "2.0.0-0", "2.1.1"]);
    }

    // ^I.J.K (for I>0) — equivalent to >=I.J.K, <(I+1).0.0-0
    for r in &[req("^1.2.3"), req(">=1.2.3, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.2.3", "1.9.9"]);
        assert_prerelease_match_none(
            r,
            &["0.0.9", "1.1.1-0", "1.2.3-0", "1.2.3-1", "2.0.0-0", "2.1.1"],
        );
    }

    // ^0.J.K-0 (for J>0) — equivalent to >=0.J.K-0, <0.(J+1).0-0
    // ^0.J.K (for J>0) — equivalent to >=0.J.K-0, <0.(J+1).0-0
    for r in &[req("^0.2.3-0"), req("^0.2.3"), req(">=0.2.3-0, <0.3.0-0")] {
        assert_prerelease_match_all(r, &["0.2.3-0", "0.2.3", "0.2.9-0", "0.2.9"]);
        assert_prerelease_match_none(r, &["0.0.9", "0.3.0-0", "0.3.11", "1.1.1"]);
    }

    // ^0.0.K-0 — equivalent to >=0.0.K-0, <0.0.(K+1)-0
    // ^0.0.K — equivalent to >=0.0.K-0, <0.0.(K+1)-0
    for r in &[req("^0.0.3-0"), req("^0.0.3"), req(">=0.0.3-0, <0.1.0-0")] {
        assert_prerelease_match_all(r, &["0.0.3-0", "0.0.3-1", "0.0.3"]);
        assert_prerelease_match_none(r, &["0.0.1", "0.3.0-0", "0.4.0-0", "1.1.1"]);
    }

    // ^I.J (for I>0 or J>0) — equivalent to >=I.J.0-0, <(I+1).0.0-0)
    for r in &[req("^1.2"), req(">=1.2.0-0, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.2.0-0", "1.2.0", "1.9.0-0", "1.9.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "0.0.4-0", "2.0.0-0", "4.0.1"]);
    }

    // ^0.0 — equivalent to >=0.0.0-0, <0.1.0-0
    for r in &[req("^0.0"), req(">=0.0.0-0, <0.1.0-0")] {
        assert_prerelease_match_all(r, &["0.0.0-0", "0.0.0", "0.0.1", "0.0.4-0"]);
        assert_prerelease_match_none(r, &["0.1.0-0", "0.1.0", "1.1.1"]);
    }

    // ^I — equivalent to >=I.0.0-0, <(I+1).0.0-0
    for r in &[req("^1"), req(">=1.0.0-0, <2.0.0-0")] {
        assert_prerelease_match_all(r, &["1.0.0-0", "1.0.0", "1.0.1"]);
        assert_prerelease_match_none(r, &["0.1.0-0", "0.1.0", "2.0.0-0", "3.1.2"]);
    }
}

#[test]
#[cfg(any(feature = "mirror_node_matches_prerelease", test_node_semver))]
fn test_wildcard() {
    // I.J.* — equivalent to =I.J
    //
    // =I.J equivalent to >=I.J.0-0, <I.(J+1).0-0
    for r in &[req("4.2.*"), req("=4.2")] {
        assert_prerelease_match_all(r, &["4.2.0-0", "4.2.0", "4.2.1", "4.2.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0"]);
        assert_prerelease_match_none(r, &["4.3.0-0", "4.3.0", "5.0.0", "5.0.1"]);
    }

    // I.* or I.*.* — equivalent to =I
    //
    // =I equivalent to >=I.0.0-0, <(I+1).0.0-0
    for r in &[req("4.*"), req("4.*.*"), req("=4")] {
        assert_prerelease_match_all(r, &["4.0.0-0", "4.0.0", "4.2.1", "4.9.9"]);
        assert_prerelease_match_none(r, &["0.0.1", "2.1.2-0"]);
        assert_prerelease_match_none(r, &["5.0.0-0", "5.0.0", "5.0.1"]);
    }
}
