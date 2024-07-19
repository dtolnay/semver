use crate::{Comparator, Op, Prerelease, Version, VersionReq};

pub(crate) fn matches_req(req: &VersionReq, ver: &Version, prerelease_matches: bool) -> bool {
    for cmp in &req.comparators {
        if prerelease_matches {
            if !matches_prerelease_impl(cmp, ver) {
                return false;
            }
        } else if !matches_impl(cmp, ver) {
            return false;
        }
    }

    if ver.pre.is_empty() || prerelease_matches {
        return true;
    }

    // If a version has a prerelease tag (for example, 1.2.3-alpha.3) then it
    // will only be allowed to satisfy req if at least one comparator with the
    // same major.minor.patch also has a prerelease tag.
    for cmp in &req.comparators {
        if pre_is_compatible(cmp, ver) {
            return true;
        }
    }

    false
}

pub(crate) fn matches_comparator(cmp: &Comparator, ver: &Version) -> bool {
    matches_impl(cmp, ver) && (ver.pre.is_empty() || pre_is_compatible(cmp, ver))
}
// If VersionReq missing Minor, Patch, then filling them with 0
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

fn matches_prerelease_impl(cmp: &Comparator, ver: &Version) -> bool {
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

fn matches_impl(cmp: &Comparator, ver: &Version) -> bool {
    match cmp.op {
        Op::Exact | Op::Wildcard => matches_exact(cmp, ver),
        Op::Greater => matches_greater(cmp, ver),
        Op::GreaterEq => matches_exact(cmp, ver) || matches_greater(cmp, ver),
        Op::Less => matches_less(cmp, ver),
        Op::LessEq => matches_exact(cmp, ver) || matches_less(cmp, ver),
        Op::Tilde => matches_tilde(cmp, ver),
        Op::Caret => matches_caret(cmp, ver),
        #[cfg(no_non_exhaustive)]
        Op::__NonExhaustive => unreachable!(),
    }
}

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

fn matches_exact(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    if let Some(minor) = cmp.minor {
        if ver.minor != minor {
            return false;
        }
    }

    if let Some(patch) = cmp.patch {
        if ver.patch != patch {
            return false;
        }
    }

    ver.pre == cmp.pre
}

fn matches_greater(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return ver.major > cmp.major;
    }

    match cmp.minor {
        None => return false,
        Some(minor) => {
            if ver.minor != minor {
                return ver.minor > minor;
            }
        }
    }

    match cmp.patch {
        None => return false,
        Some(patch) => {
            if ver.patch != patch {
                return ver.patch > patch;
            }
        }
    }

    ver.pre > cmp.pre
}

fn matches_less(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return ver.major < cmp.major;
    }

    match cmp.minor {
        None => return false,
        Some(minor) => {
            if ver.minor != minor {
                return ver.minor < minor;
            }
        }
    }

    match cmp.patch {
        None => return false,
        Some(patch) => {
            if ver.patch != patch {
                return ver.patch < patch;
            }
        }
    }

    ver.pre < cmp.pre
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

fn matches_tilde(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    if let Some(minor) = cmp.minor {
        if ver.minor != minor {
            return false;
        }
    }

    if let Some(patch) = cmp.patch {
        if ver.patch != patch {
            return ver.patch > patch;
        }
    }

    ver.pre >= cmp.pre
}

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

fn matches_caret(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    let minor = match cmp.minor {
        None => return true,
        Some(minor) => minor,
    };

    let patch = match cmp.patch {
        None => {
            if cmp.major > 0 {
                return ver.minor >= minor;
            } else {
                return ver.minor == minor;
            }
        }
        Some(patch) => patch,
    };

    if cmp.major > 0 {
        if ver.minor != minor {
            return ver.minor > minor;
        } else if ver.patch != patch {
            return ver.patch > patch;
        }
    } else if minor > 0 {
        if ver.minor != minor {
            return false;
        } else if ver.patch != patch {
            return ver.patch > patch;
        }
    } else if ver.minor != minor || ver.patch != patch {
        return false;
    }

    ver.pre >= cmp.pre
}

fn pre_is_compatible(cmp: &Comparator, ver: &Version) -> bool {
    cmp.major == ver.major
        && cmp.minor == Some(ver.minor)
        && cmp.patch == Some(ver.patch)
        && !cmp.pre.is_empty()
}
