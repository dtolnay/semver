use super::eval::{matches_exact, matches_greater, matches_less};
use crate::{Comparator, Op, Prerelease, Version};

pub(super) fn matches_prerelease_impl(cmp: &Comparator, ver: &Version) -> bool {
    match cmp.op {
        Op::Exact | Op::Wildcard => matches_exact_prerelease(cmp, ver),
        Op::Greater => matches_greater(cmp, ver),
        Op::GreaterEq => {
            if matches_exact_prerelease(cmp, ver) {
                return true;
            }
            matches_greater(cmp, ver)
        }
        Op::Less => matches_less(&fill_partial_req(cmp), ver),
        Op::LessEq => {
            if matches_exact_prerelease(cmp, ver) {
                return true;
            }
            matches_less(&fill_partial_req(cmp), ver)
        }
        Op::Tilde => matches_tilde_prerelease(cmp, ver),
        Op::Caret => matches_caret_prerelease(cmp, ver),
        #[cfg(no_non_exhaustive)]
        Op::__NonExhaustive => unreachable!(),
    }
}

#[cfg(not(feature = "mirror_node_matches_prerelease"))]
fn fill_partial_req(cmp: &Comparator) -> Comparator {
    let mut cmp = cmp.clone();
    if cmp.minor.is_none() {
        cmp.minor = Some(0);
        cmp.patch = Some(0);
    } else if cmp.patch.is_none() {
        cmp.patch = Some(0);
    }
    cmp
}

#[cfg(not(feature = "mirror_node_matches_prerelease"))]
fn matches_exact_prerelease(cmp: &Comparator, ver: &Version) -> bool {
    if matches_exact(cmp, ver) {
        return true;
    }

    // If the comparator has a prerelease tag like =3.0.0-alpha.24,
    // then it shoud be only exactly match 3.0.0-alpha.24.
    if !cmp.pre.is_empty() {
        return false;
    }

    if !matches_greater(&fill_partial_req(cmp), ver) {
        return false;
    }

    let mut upper = Comparator {
        op: Op::Less,
        pre: Prerelease::new("0").unwrap(),
        ..cmp.clone()
    };

    match (upper.minor.is_some(), upper.patch.is_some()) {
        (true, true) => {
            upper.patch = Some(upper.patch.unwrap() + 1);
        }
        (true, false) => {
            // Partial Exact VersionReq eg. =0.24
            upper.minor = Some(upper.minor.unwrap() + 1);
            upper.patch = Some(0);
        }
        (false, false) => {
            // Partial Exact VersionReq eg. =0
            upper.major += 1;
            upper.minor = Some(0);
            upper.patch = Some(0);
        }
        _ => {}
    }

    matches_less(&upper, ver)
}

fn matches_tilde_prerelease(cmp: &Comparator, ver: &Version) -> bool {
    if matches_exact(cmp, ver) {
        return true;
    }

    if !matches_greater(&fill_partial_req(cmp), ver) {
        return false;
    }

    let mut upper = Comparator {
        op: Op::Less,
        pre: Prerelease::new("0").unwrap(),
        ..cmp.clone()
    };

    match (upper.minor.is_some(), upper.patch.is_some()) {
        (true, _) => {
            upper.minor = Some(upper.minor.unwrap() + 1);
            upper.patch = Some(0);
        }
        (false, false) => {
            upper.major += 1;
            upper.minor = Some(0);
            upper.patch = Some(0);
        }
        _ => {}
    }

    matches_less(&upper, ver)
}

#[cfg(not(feature = "mirror_node_matches_prerelease"))]
fn matches_caret_prerelease(cmp: &Comparator, ver: &Version) -> bool {
    if matches_exact(cmp, ver) {
        return true;
    }

    if !matches_greater(&fill_partial_req(cmp), ver) {
        return false;
    }

    let mut upper = Comparator {
        op: Op::Less,
        pre: Prerelease::new("0").unwrap(),
        ..cmp.clone()
    };

    match (
        upper.major > 0,
        upper.minor.is_some(),
        upper.patch.is_some(),
    ) {
        (true, _, _) | (_, false, false) => {
            upper.major += 1;
            upper.minor = Some(0);
            upper.patch = Some(0);
        }
        (_, true, false) => {
            upper.minor = Some(upper.minor.unwrap() + 1);
            upper.patch = Some(0);
        }
        (_, true, _) if upper.minor.unwrap() > 0 => {
            upper.minor = Some(upper.minor.unwrap() + 1);
            upper.patch = Some(0);
        }
        (_, true, _) if upper.minor.unwrap() == 0 => {
            if upper.patch.is_none() {
                upper.patch = Some(1);
            } else {
                upper.patch = Some(upper.patch.unwrap() + 1);
            }
        }
        _ => {}
    }

    matches_less(&upper, ver)
}

#[cfg(feature = "mirror_node_matches_prerelease")]
fn fill_partial_req(cmp: &Comparator) -> Comparator {
    let mut cmp = cmp.clone();
    if cmp.minor.is_none() {
        cmp.minor = Some(0);
        cmp.patch = Some(0);
        cmp.pre = Prerelease::new("0").unwrap();
    } else if cmp.patch.is_none() {
        cmp.patch = Some(0);
        cmp.pre = Prerelease::new("0").unwrap();
    }
    cmp
}

#[cfg(feature = "mirror_node_matches_prerelease")]
fn matches_exact_prerelease(cmp: &Comparator, ver: &Version) -> bool {
    let lower = fill_partial_req(cmp);
    if matches_exact(&lower, ver) {
        return true;
    }

    // If the comparator has a prerelease tag like =3.0.0-alpha.24,
    // then it shoud be only exactly match 3.0.0-alpha.24.
    if !cmp.pre.is_empty() {
        return false;
    }

    if !matches_greater(&lower, ver) {
        return false;
    }

    let mut upper = Comparator {
        op: Op::Less,
        pre: Prerelease::new("0").unwrap(),
        ..cmp.clone()
    };

    match (upper.minor.is_some(), upper.patch.is_some()) {
        (true, true) => {
            upper.patch = Some(upper.patch.unwrap() + 1);
        }
        (true, false) => {
            // Partial Exact VersionReq eg. =0.24
            upper.minor = Some(upper.minor.unwrap() + 1);
            upper.patch = Some(0);
        }
        (false, false) => {
            // Partial Exact VersionReq eg. =0
            upper.major += 1;
            upper.minor = Some(0);
            upper.patch = Some(0);
        }
        _ => {}
    }

    matches_less(&upper, ver)
}

#[cfg(feature = "mirror_node_matches_prerelease")]
fn matches_caret_prerelease(cmp: &Comparator, ver: &Version) -> bool {
    let mut lower = fill_partial_req(cmp);
    if lower.major == 0 && lower.pre.is_empty() {
        lower.pre = Prerelease::new("0").unwrap();
    }

    if matches_exact(&lower, ver) {
        return true;
    }

    if !matches_greater(&lower, ver) {
        return false;
    }

    let mut upper = Comparator {
        op: Op::Less,
        pre: Prerelease::new("0").unwrap(),
        ..cmp.clone()
    };

    match (
        upper.major > 0,
        upper.minor.is_some(),
        upper.patch.is_some(),
    ) {
        (true, _, _) | (_, false, false) => {
            upper.major += 1;
            upper.minor = Some(0);
            upper.patch = Some(0);
        }
        (_, true, false) => {
            upper.minor = Some(upper.minor.unwrap() + 1);
            upper.patch = Some(0);
        }
        (_, true, _) if upper.minor.unwrap() > 0 => {
            upper.minor = Some(upper.minor.unwrap() + 1);
            upper.patch = Some(0);
        }
        (_, true, _) if upper.minor.unwrap() == 0 => {
            if upper.patch.is_none() {
                upper.patch = Some(1);
            } else {
                upper.patch = Some(upper.patch.unwrap() + 1);
            }
        }
        _ => {}
    }

    matches_less(&upper, ver)
}
