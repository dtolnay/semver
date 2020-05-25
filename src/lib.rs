use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[error("Could not parse properly.")]
pub enum ParseError {
    #[error("Numeric identifiers MUST NOT include leading zeroes.")]
    LeadingZero,
    #[error("Parsing error.")]
    Incorrect,
}

pub fn satisfies(version: &str, range: &str) -> bool {
    let mut range_iter = range.char_indices().peekable();

    // empty string is equivalent to *
    if range.is_empty() {
        return true;
    }

    match range_iter.next() {
        Some((_, '<')) => match range_iter.peek() {
            Some((_, '=')) => satisfies_lte(version, range),
            Some(_) => satisfies_lt(version, range),
            None => false,
        },
        Some((_, '>')) => match range_iter.peek() {
            Some((_, '=')) => satisfies_gte(version, range),
            Some(_) => satisfies_gt(version, range),
            None => false,
        },
        Some((_, '=')) => satisfies_eq(version, range),
        Some((_, '^')) => satisfies_caret(version, &range[1..]),
        Some((_, '~')) => satisfies_tilde(version, range),
        Some((_, 'X')) => true,
        Some((_, 'x')) => true,
        Some((_, '*')) => true,
        Some(_) => satisfies_caret(version, range),
        _ => false,
    }
}

fn satisfies_tilde(version: &str, range: &str) -> bool {
    let mut version_iter = version.char_indices().peekable();
    // we chop off the ~
    let mut range_iter = range[1..].char_indices().peekable();

    (|| {
        // first we check the major version
        let range_major = parse_major(&mut range_iter).ok()?;
        let version_major = parse_major(&mut version_iter).ok()?;

        // if it is greater than, fail
        if version_major > range_major {
            return None;
        }

        // now we need to check if we have a minor version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a minor version, we need to make sure the major wasn't less than
            if version_major < range_major {
                return None;
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_minor = parse_minor(&mut range_iter).ok().unwrap_or(0);
        let version_minor = parse_minor(&mut version_iter).ok()?;

        // if it is less than, fail
        if version_minor < range_minor {
            return None;
        }

        // now we need to check if we have a patch version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a patch version, we need to make sure the minor wasn't greater than
            if version_minor > range_minor {
                return None;
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_patch = parse_patch(&mut range_iter).ok().unwrap_or(0);
        let version_patch = parse_patch(&mut version_iter).ok()?;

        // if it is less than, fail
        if version_patch < range_patch {
            return None;
        }

        // now, for prerelease versions

        let mut version_rest = version;

        if let Some((idx, c)) = version_iter.peek() {
            if matches!(c, '-' | '+') {
                version_rest = &version[*idx..];
            }
        }

        let mut range_rest = range;

        if let Some((idx, c)) = range_iter.peek() {
            if matches!(c, '-' | '+') {
                // we need to add 1 because we removed the ~ before
                range_rest = &range[(*idx + 1)..];
            }
        }

        // now we need to check if we have a prerelease version or not.
        match (parse_pre(range_rest), parse_pre(version_rest)) {
            (Some(mut range_pre), Some(mut version_pre)) => {
                if (version_major != range_major)
                    || (version_minor != range_minor)
                    || (version_patch != range_patch)
                {
                    return None;
                }

                while let (Some(range), Some(version)) = (range_pre.next(), version_pre.next()) {
                    if version != range {
                        return None;
                    }
                }
            }

            (None, Some(_)) => {
                return None;
            }
            (Some(_), None) => {
                return Some(());
            }
            (None, None) => {
                return Some(());
            }
        }

        Some(())
    })()
    .is_some()
}

fn satisfies_caret(version: &str, range: &str) -> bool {
    let mut version_iter = version.char_indices().peekable();
    let mut range_iter = range[..].char_indices().peekable();

    (|| {
        // first we check the major version
        let range_major = parse_major(&mut range_iter).ok()?;
        let version_major = parse_major(&mut version_iter).ok()?;

        // if it is greater than, fail
        if version_major > range_major {
            return None;
        }

        // now we need to check if we have a minor version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a minor version, we need to make sure the major wasn't less than
            if version_major < range_major {
                return None;
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_minor = parse_minor(&mut range_iter).ok().unwrap_or(0);
        let version_minor = parse_minor(&mut version_iter).ok()?;

        // if it is less than, fail
        if version_minor < range_minor {
            return None;
        }

        // now we need to check if we have a patch version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a patch version, we need to make sure the minor wasn't less than
            if version_minor < range_minor {
                return None;
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_patch = parse_patch(&mut range_iter).ok().unwrap_or(0);
        let version_patch = parse_patch(&mut version_iter).ok()?;

        // if it is less than, fail
        if version_patch < range_patch {
            return None;
        }

        // now, for prerelease versions

        let mut version_rest = version;

        if let Some((idx, c)) = version_iter.peek() {
            if matches!(c, '-' | '+') {
                version_rest = &version[*idx..];
            }
        }

        let mut range_rest = range;

        if let Some((idx, c)) = range_iter.peek() {
            if matches!(c, '-' | '+') {
                range_rest = &range[*idx..];
            }
        }

        // now we need to check if we have a prerelease version or not.
        match (parse_pre(range_rest), parse_pre(version_rest)) {
            (Some(mut range_pre), Some(mut version_pre)) => {
                if (version_major != range_major)
                    || (version_minor != range_minor)
                    || (version_patch != range_patch)
                {
                    return None;
                }

                while let (Some(range), Some(version)) = (range_pre.next(), version_pre.next()) {
                    if version != range {
                        return None;
                    }
                }
            }

            (None, Some(_)) => {
                return None;
            }
            (Some(_), None) => {
                return Some(());
            }
            (None, None) => {
                return Some(());
            }
        }

        Some(())
    })()
    .is_some()
}

fn satisfies_eq(version: &str, range: &str) -> bool {
    let mut version_iter = version.char_indices().peekable();
    // we chop off the =
    let mut range_iter = range[1..].char_indices().peekable();

    (|| {
        // first we check the major version
        let range_major = parse_major(&mut range_iter).ok()?;
        let version_major = parse_major(&mut version_iter).ok()?;

        // if it is not equal, fail
        if version_major != range_major {
            return None;
        }

        // now we need to check if we have a minor version or not.
        if parse_dot(&mut range_iter).is_none() {
            return Some(());
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_minor = parse_minor(&mut range_iter).ok()?;
        let version_minor = parse_minor(&mut version_iter).ok()?;

        // if it is not equal, fail
        if version_minor != range_minor {
            return None;
        }

        // now we need to check if we have a patch version or not.
        if parse_dot(&mut range_iter).is_none() {
            return Some(());
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_patch = parse_patch(&mut range_iter).ok()?;
        let version_patch = parse_patch(&mut version_iter).ok()?;

        // if it is not equal, fail
        if version_patch != range_patch {
            return None;
        }

        // now, for prerelease versions

        let mut version_rest = version;

        if let Some((idx, c)) = version_iter.peek() {
            if matches!(c, '-' | '+') {
                version_rest = &version[*idx..];
            }
        }

        let mut range_rest = range;

        if let Some((idx, c)) = range_iter.peek() {
            if matches!(c, '-' | '+') {
                // we need to add 1 because we removed the = before
                range_rest = &range[(*idx + 1)..];
            }
        }

        // now we need to check if we have a prerelease version or not.
        match (parse_pre(range_rest), parse_pre(version_rest)) {
            (Some(mut range_pre), Some(mut version_pre)) => {
                if (version_major != range_major)
                    || (version_minor != range_minor)
                    || (version_patch != range_patch)
                {
                    return None;
                }

                while let (Some(range), Some(version)) = (range_pre.next(), version_pre.next()) {
                    if version != range {
                        return None;
                    }
                }
            }

            (None, Some(_)) => {
                return None;
            }
            (Some(_), None) => {
                return None;
            }
            (None, None) => {
                return Some(());
            }
        }

        Some(())
    })()
    .is_some()
}

fn satisfies_gt(version: &str, range: &str) -> bool {
    let mut version_iter = version.char_indices().peekable();
    // we chop off the >
    let mut range_iter = range[1..].char_indices().peekable();

    (|| {
        // first we check the major version
        let range_major = parse_major(&mut range_iter).ok()?;
        let version_major = parse_major(&mut version_iter).ok()?;

        // if it is not greater than, fail
        if version_major < range_major {
            return None;
        }

        // if it is greater than, pass
        if version_major > range_major {
            return Some(());
        }

        // now we need to check if we have a minor version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a minor version, we need to make sure the major wasn't equal
            if version_major == range_major {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_minor = parse_minor(&mut range_iter).ok()?;
        let version_minor = parse_minor(&mut version_iter).ok()?;

        // if it is not greater than, fail
        if version_minor < range_minor {
            return None;
        }

        // now we need to check if we have a patch version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a patch version, we need to make sure the minor wasn't equal
            if version_minor == range_minor {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_patch = parse_patch(&mut range_iter).ok()?;
        let version_patch = parse_patch(&mut version_iter).ok()?;

        // if it is not greater than, fail
        if version_patch < range_patch {
            return None;
        }

        // now, for prerelease versions

        let mut version_rest = version;

        if let Some((idx, c)) = version_iter.peek() {
            if matches!(c, '-' | '+') {
                version_rest = &version[*idx..];
            }
        }

        let mut range_rest = range;

        if let Some((idx, c)) = range_iter.peek() {
            if matches!(c, '-' | '+') {
                // we need to add 1 because we removed the > before
                range_rest = &range[(*idx + 1)..];
            }
        }

        // now we need to check if we have a prerelease version or not.
        match (parse_pre(range_rest), parse_pre(version_rest)) {
            (Some(mut range_pre), Some(mut version_pre)) => {
                if (version_major != range_major)
                    || (version_minor != range_minor)
                    || (version_patch != range_patch)
                {
                    return None;
                }

                let mut all_eq = true;

                while let (Some(range), Some(version)) = (range_pre.next(), version_pre.next()) {
                    if version < range {
                        return None;
                    }

                    if version != range {
                        all_eq = false;
                    }
                }

                if all_eq {
                    return None;
                }
            }

            (None, Some(_)) => {
                return None;
            }
            (Some(_), None) => {
                return Some(());
            }
            (None, None) => {
                // if we don't have a pre-release version, we need to make sure the patch wasn't equal
                if version_patch == range_patch {
                    return None;
                } else {
                    return Some(());
                }
            }
        }

        Some(())
    })()
    .is_some()
}

fn satisfies_gte(version: &str, range: &str) -> bool {
    let mut version_iter = version.char_indices().peekable();
    // we chop off the >=
    let mut range_iter = range[2..].char_indices().peekable();

    (|| {
        // first we check the major version
        let range_major = parse_major(&mut range_iter).ok()?;
        let version_major = parse_major(&mut version_iter).ok()?;

        // if it is not greater than, fail
        if version_major < range_major {
            return None;
        }

        // if it is greater than, pass
        if version_major > range_major {
            return Some(());
        }

        // now we need to check if we have a minor version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a minor version, we need to make sure the major wasn't equal
            if version_major == range_major {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_minor = parse_minor(&mut range_iter).ok()?;
        let version_minor = parse_minor(&mut version_iter).ok()?;

        // if it is not greater than, fail
        if version_minor < range_minor {
            return None;
        }

        // now we need to check if we have a patch version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a patch version, we need to make sure the minor wasn't equal
            if version_minor == range_minor {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_patch = parse_patch(&mut range_iter).ok()?;
        let version_patch = parse_patch(&mut version_iter).ok()?;

        // if it is not greater than, fail
        if version_patch < range_patch {
            return None;
        }

        // now, for prerelease versions

        let mut version_rest = version;

        if let Some((idx, c)) = version_iter.peek() {
            if matches!(c, '-' | '+') {
                version_rest = &version[*idx..];
            }
        }

        let mut range_rest = range;

        if let Some((idx, c)) = range_iter.peek() {
            if matches!(c, '-' | '+') {
                // we need to add 2 because we removed the >= before
                range_rest = &range[(*idx + 2)..];
            }
        }

        // now we need to check if we have a prerelease version or not.
        match (parse_pre(range_rest), parse_pre(version_rest)) {
            (Some(mut range_pre), Some(mut version_pre)) => {
                if (version_major != range_major)
                    || (version_minor != range_minor)
                    || (version_patch != range_patch)
                {
                    return None;
                }

                while let (Some(range), Some(version)) = (range_pre.next(), version_pre.next()) {
                    if version < range {
                        return None;
                    }
                }
            }
            (None, Some(_)) => {
                return None;
            }
            _ => {
                return Some(());
            }
        }

        Some(())
    })()
    .is_some()
}

fn satisfies_lt(version: &str, range: &str) -> bool {
    let mut version_iter = version.char_indices().peekable();
    // we chop off the <
    let mut range_iter = range[1..].char_indices().peekable();

    (|| {
        // first we check the major version
        let range_major = parse_major(&mut range_iter).ok()?;
        let version_major = parse_major(&mut version_iter).ok()?;

        // if it is not less than, fail
        if version_major > range_major {
            return None;
        }

        // if it is less than, pass
        if version_major < range_major {
            return Some(());
        }

        // now we need to check if we have a minor version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a minor version, we need to make sure the major wasn't equal
            if version_major == range_major {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_minor = parse_minor(&mut range_iter).ok()?;
        let version_minor = parse_minor(&mut version_iter).ok()?;

        // if it is not less than, fail
        if version_minor > range_minor {
            return None;
        }

        // now we need to check if we have a patch version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a patch version, we need to make sure the minor wasn't equal
            if version_minor == range_minor {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_patch = parse_patch(&mut range_iter).ok()?;
        let version_patch = parse_patch(&mut version_iter).ok()?;

        // if it is not less than, fail
        if version_patch > range_patch {
            return None;
        }

        // now, for prerelease versions

        let mut version_rest = version;

        if let Some((idx, c)) = version_iter.peek() {
            if matches!(c, '-' | '+') {
                version_rest = &version[*idx..];
            }
        }

        let mut range_rest = range;

        if let Some((idx, c)) = range_iter.peek() {
            if matches!(c, '-' | '+') {
                // we need to add 1 because we removed the < before
                range_rest = &range[(*idx + 1)..];
            }
        }

        // now we need to check if we have a prerelease version or not.
        match (parse_pre(range_rest), parse_pre(version_rest)) {
            (Some(mut range_pre), Some(mut version_pre)) => {
                if (version_major != range_major)
                    || (version_minor != range_minor)
                    || (version_patch != range_patch)
                {
                    return None;
                }

                let mut all_eq = true;

                while let (Some(range), Some(version)) = (range_pre.next(), version_pre.next()) {
                    if version > range {
                        return None;
                    }

                    if version != range {
                        all_eq = false;
                    }
                }

                if all_eq {
                    return None;
                }
            }

            (None, Some(_)) => {
                return None;
            }
            (Some(_), None) => {
                // if we don't have a pre-release version, we need to make sure the patch wasn't equal
                if version_patch == range_patch {
                    return None;
                } else {
                    return Some(());
                }
            }
            (None, None) => {
                // if we don't have a pre-release version, we need to make sure the patch wasn't equal
                if version_patch == range_patch {
                    return None;
                } else {
                    return Some(());
                }
            }
        }

        Some(())
    })()
    .is_some()
}

fn satisfies_lte(version: &str, range: &str) -> bool {
    let mut version_iter = version.char_indices().peekable();
    // we chop off the <=
    let mut range_iter = range[2..].char_indices().peekable();

    (|| {
        // first we check the major version
        let range_major = parse_major(&mut range_iter).ok()?;
        let version_major = parse_major(&mut version_iter).ok()?;

        // if it is not less than, fail
        if version_major > range_major {
            return None;
        }

        // if it is less than, pass
        if version_major < range_major {
            return Some(());
        }

        // now we need to check if we have a minor version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a minor version, we need to make sure the major wasn't equal
            if version_major == range_major {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_minor = parse_minor(&mut range_iter).ok()?;
        let version_minor = parse_minor(&mut version_iter).ok()?;

        // if it is not less than, fail
        if version_minor > range_minor {
            return None;
        }

        // now we need to check if we have a patch version or not.
        if parse_dot(&mut range_iter).is_none() {
            // if we don't have a patch version, we need to make sure the minor wasn't equal
            if version_minor == range_minor {
                return None;
            } else {
                return Some(());
            }
        }

        if parse_dot(&mut version_iter).is_none() {
            return Some(());
        }

        let range_patch = parse_patch(&mut range_iter).ok()?;
        let version_patch = parse_patch(&mut version_iter).ok()?;

        // if it is not less than, fail
        if version_patch > range_patch {
            return None;
        }

        // now, for prerelease versions

        let mut version_rest = version;

        if let Some((idx, c)) = version_iter.peek() {
            if matches!(c, '-' | '+') {
                version_rest = &version[*idx..];
            }
        }

        let mut range_rest = range;

        if let Some((idx, c)) = range_iter.peek() {
            if matches!(c, '-' | '+') {
                // we need to add 2 because we removed the <= before
                range_rest = &range[(*idx + 2)..];
            }
        }

        // now we need to check if we have a prerelease version or not.
        match (parse_pre(range_rest), parse_pre(version_rest)) {
            (Some(mut range_pre), Some(mut version_pre)) => {
                if (version_major != range_major)
                    || (version_minor != range_minor)
                    || (version_patch != range_patch)
                {
                    return None;
                }

                while let (Some(range), Some(version)) = (range_pre.next(), version_pre.next()) {
                    if version > range {
                        return None;
                    }
                }
            }

            (None, Some(_)) => {
                return None;
            }
            _ => {
                return Some(());
            }
        }

        Some(())
    })()
    .is_some()
}

pub fn valid(version: &str) -> bool {
    let mut iter = version.char_indices().peekable();

    (|| {
        parse_major(&mut iter).ok()?;
        parse_dot(&mut iter)?;
        parse_minor(&mut iter).ok()?;
        parse_dot(&mut iter)?;
        parse_patch(&mut iter).ok()?;

        let mut rest = version;

        if let Some((idx, c)) = iter.peek() {
            if matches!(c, '=' | '+') {
                rest = &version[*idx..];
            }
        }

        parse_pre(rest);

        parse_build(rest);

        Some(())
    })()
    .is_some()
}

// not const yet see https://github.com/rust-lang/rust/issues/49146
pub fn major(version: &str) -> Result<u64, ParseError> {
    parse_major(&mut version.char_indices().peekable())
}

// not const yet see https://github.com/rust-lang/rust/issues/49146
pub fn minor(version: &str) -> Result<u64, ParseError> {
    let mut iter = version.char_indices().peekable();

    parse_major(&mut iter)?;

    parse_dot(&mut iter).map_or(Err(ParseError::Incorrect), Ok)?;

    parse_minor(&mut iter)
}

// not const yet see https://github.com/rust-lang/rust/issues/49146
pub fn patch(version: &str) -> Result<u64, ParseError> {
    let mut iter = version.char_indices().peekable();

    parse_major(&mut iter)?;

    parse_dot(&mut iter).map_or(Err(ParseError::Incorrect), Ok)?;

    parse_minor(&mut iter)?;

    parse_dot(&mut iter).map_or(Err(ParseError::Incorrect), Ok)?;

    parse_patch(&mut iter)
}

// not const yet see https://github.com/rust-lang/rust/issues/49146
pub fn pre(version: &str) -> Option<impl Iterator<Item = &str>> {
    let mut iter = version.char_indices().peekable();

    parse_major(&mut iter).ok()?;

    parse_dot(&mut iter)?;

    parse_minor(&mut iter).ok()?;

    parse_dot(&mut iter)?;

    parse_patch(&mut iter).ok()?;

    let mut rest = version;

    if let Some((idx, _)) = iter.next() {
        rest = &version[idx..];
    }

    parse_pre(rest)
}

pub fn build(version: &str) -> Option<impl Iterator<Item = Result<&str, ParseError>>> {
    let mut iter = version.char_indices().peekable();

    parse_major(&mut iter).ok()?;

    parse_dot(&mut iter)?;

    parse_minor(&mut iter).ok()?;

    parse_dot(&mut iter)?;

    parse_patch(&mut iter).ok()?;

    let mut rest = version;

    if let Some((idx, c)) = iter.peek() {
        if matches!(c, '=' | '+') {
            rest = &version[*idx..];
        }
    }

    parse_pre(rest);

    parse_build(rest)
}

fn parse_major(
    iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<u64, ParseError> {
    parse_number(iter)
}

fn parse_minor(
    iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<u64, ParseError> {
    parse_number(iter)
}

fn parse_patch(
    iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<u64, ParseError> {
    parse_number(iter)
}

fn parse_pre(rest: &str) -> Option<impl Iterator<Item = &str>> {
    let mut iter = rest.char_indices().peekable();

    match iter.next() {
        Some((_, '-')) => {}
        _ => {
            return None;
        }
    }

    let mut rest = &rest[1..];

    if rest.is_empty() {
        return None;
    }

    let o = iter.find(|&(_, c)| !matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '.'));

    if let Some((idx, _)) = o {
        rest = &rest[..idx];
    }

    for s in rest.split('.') {
        // check for leading zeros. Plain old 0 is fine, but starting with 0 is not
        if s.starts_with('0') && (s.len() != 1) {
            return None;
        }
    }

    Some(rest.split('.'))
}

fn parse_build(rest: &str) -> Option<impl Iterator<Item = Result<&str, ParseError>>> {
    let mut iter = rest.char_indices().peekable();

    match iter.next() {
        Some((_, '+')) => {}
        _ => {
            return None;
        }
    }

    let mut rest = &rest[1..];

    let o = iter.find(|&(_, c)| !matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '.'));

    if let Some((idx, _)) = o {
        rest = &rest[..idx];
    }

    Some(rest.split('.').map(|s| match s.len() {
        0 => Err(ParseError::Incorrect),
        _ => Ok(s),
    }))
}

fn parse_dot(iter: &mut impl Iterator<Item = (usize, char)>) -> Option<()> {
    // sigh https://github.com/rust-lang/rfcs/issues/2616
    match iter.next() {
        Some((_, '.')) => {}
        _ => {
            return None;
        }
    }

    Some(())
}

// thanks <3 https://adriann.github.io/rust_parser.html
//
// I don't like the body of this function, but I can always clean it up later.
fn parse_number(
    iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<u64, ParseError> {
    // we gotta have something left to return a number
    let peek = iter.peek();

    if peek.is_none() {
        return Err(ParseError::Incorrect);
    }

    if let Some(&(_, c)) = peek {
        if c == 'x' || c == 'X' || c == '*' {
            return Ok(0);
        }

        if !c.is_numeric() {
            return Err(ParseError::Incorrect);
        }
    }

    let mut number = 0;
    let mut possible_leading_zero = false;

    // we have to check if the first digit is a zero, because that's not allowed.
    if let Some(Some(digit)) = iter.peek().map(|&(_, c)| c.to_digit(10)) {
        if digit == 0 {
            possible_leading_zero = true;
        } else {
            number = number * 10 + digit as u64;
            iter.next();
        }
    }

    while let Some(Some(digit)) = iter.peek().map(|&(_, c)| c.to_digit(10)) {
        number = number * 10 + digit as u64;
        iter.next();
    }

    if number != 0 && possible_leading_zero {
        return Err(ParseError::LeadingZero);
    }

    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::ParseError;

    #[test]
    fn parse_number() {
        // testing weird cases

        for c in b'A'..=b'z' {
            let s = (c as char).to_string();
            let mut iter = s.char_indices().peekable();

            if c == b'x' {
                assert_eq!(0, crate::parse_number(&mut iter).unwrap());
            } else if c == b'X' {
                assert_eq!(0, crate::parse_number(&mut iter).unwrap());
            } else if c == b'*' {
                assert_eq!(0, crate::parse_number(&mut iter).unwrap());
            } else {
                assert!(crate::parse_number(&mut iter).is_err());
            }
        }
    }

    #[test]
    fn valid() {
        // examples of valid numbers
        assert!(super::valid("1.2.3"));
        assert!(super::valid("10.20.30"));
        assert!(super::valid("100.200.300"));
        assert!(super::valid("100.200.300-alpha"));
        assert!(super::valid("100.200.300-alpha.2"));
        assert!(super::valid(
            "100.200.300+b4039641946cc7ea87357204c7b14c6090703857"
        ));
        assert!(super::valid("100.200.300-pre.and+build"));

        // examples of invalid numbers
        assert!(!super::valid("1"));
        assert!(!super::valid("1."));
        assert!(!super::valid("1.2."));
    }

    #[test]
    fn major() {
        // "A normal version number MUST take the form X.Y.Z where X, Y, and Z
        // are non-negative integers... X is the major version."
        assert_eq!(Ok(1), super::major("1.2.3"));
        assert_eq!(Ok(10), super::major("10.20.30"));
        assert_eq!(Ok(100), super::major("100.200.300"));

        // ... and MUST NOT contain leading zeroes.
        assert_eq!(Err(ParseError::LeadingZero), super::major("01.2.3"));
        // 0 is not a leading zero
        assert_eq!(Ok(0), super::major("0.2.3"));
    }

    #[test]
    fn minor() {
        // "A normal version number MUST take the form X.Y.Z where X, Y, and Z
        // are non-negative integers... Y is the minor version."
        assert_eq!(Ok(2), super::minor("1.2.3"));
        assert_eq!(Ok(20), super::minor("10.20.30"));
        assert_eq!(Ok(200), super::minor("100.200.300"));

        // ... and MUST NOT contain leading zeroes.
        assert_eq!(Err(ParseError::LeadingZero), super::minor("1.02.3"));
        // 0 is not a leading zero
        assert_eq!(Ok(0), super::minor("1.0.3"));
    }

    #[test]
    fn patch() {
        // "A normal version number MUST take the form X.Y.Z where X, Y, and Z
        // are non-negative integers... Z is the patch version."
        assert_eq!(Ok(3), super::patch("1.2.3"));
        assert_eq!(Ok(30), super::patch("10.20.30"));
        assert_eq!(Ok(300), super::patch("100.200.300"));

        // ... and MUST NOT contain leading zeroes.
        assert_eq!(Err(ParseError::LeadingZero), super::patch("1.2.03"));
        // 0 is not a leading zero
        assert_eq!(Ok(0), super::patch("1.2.0"));
    }

    #[test]
    fn pre() {
        // pre-release version MAY be denoted by appending a hyphen and a series
        // of dot separated identifiers immediately following the patch version.
        let mut pre = super::pre("1.2.3-alpha.2").unwrap();
        assert_eq!(Some("alpha"), pre.next());
        assert_eq!(Some("2"), pre.next());
        assert!(pre.next().is_none());

        // Identifiers MUST comprise only ASCII alphanumerics and hyphen
        // [0-9A-Za-z-].
        let mut pre = super::pre("1.2.3-alp-ha").unwrap();
        assert_eq!(Some("alp-ha"), pre.next());
        assert!(pre.next().is_none());

        // Identifiers MUST NOT be empty.
        assert!(super::pre("1.2.3-").is_none());

        // Numeric identifiers MUST NOT include leading zeroes.
        let pre = super::pre("1.2.3-02");
        assert!(pre.is_none());

        // 0 on its own is not a leading zero!
        let mut pre = super::pre("1.2.3-0").unwrap();
        assert_eq!(Some("0"), pre.next());
        assert!(pre.next().is_none());

        // none means no pre-release
        assert!(super::pre("1.2.3").is_none());
    }

    #[test]
    fn build() {
        // Build metadata MAY be denoted by appending a plus sign and a series
        // of dot separated identifiers immediately following the patch or
        // pre-release version.
        let mut build = super::build("1.2.3+alpha.2").unwrap();
        assert_eq!(Some(Ok("alpha")), build.next());
        assert_eq!(Some(Ok("2")), build.next());
        assert!(build.next().is_none());

        // Identifiers MUST comprise only ASCII alphanumerics and hyphen
        // [0-9A-Za-z-].
        let mut build = super::build("1.2.3+alp-ha").unwrap();
        assert_eq!(Some(Ok("alp-ha")), build.next());
        assert!(build.next().is_none());

        // Identifiers MUST NOT be empty.
        let mut build = super::build("1.2.3+").unwrap();
        assert!(build.next().unwrap().is_err());
    }

    mod satisfies {
        mod primitive {
            #[test]
            fn less_than() {
                // major only
                assert!(crate::satisfies("1.2.3", "<2"));
                assert!(!crate::satisfies("1.2.3", "<1"));
                assert!(!crate::satisfies("1.2.3", "<0"));

                // major and minor
                assert!(crate::satisfies("1.2.3", "<1.3"));
                assert!(!crate::satisfies("1.2.3", "<1.2"));
                assert!(!crate::satisfies("1.2.3", "<1.1"));

                // major, minor, and patch
                assert!(crate::satisfies("1.2.3", "<1.2.4"));
                assert!(!crate::satisfies("1.2.3", "<1.2.3"));
                assert!(!crate::satisfies("1.2.3", "<1.2.2"));

                // prerelease
                assert!(crate::satisfies("1.2.3-pre.1", "<1.2.3-pre.2"));
                assert!(!crate::satisfies("1.2.3-pre.1", "<1.2.3-pre.1"));
                assert!(!crate::satisfies("1.2.3-pre.1", "<1.2.3-pre.0"));

                // prerelease in range, but not in version
                assert!(crate::satisfies("0.1.0", "<1.0.0-pre.1"));
                assert!(!crate::satisfies("1.0.0", "<1.0.0-pre.1"));

                // prerelease in version, but not in range
                assert!(!crate::satisfies("1.0.0-beta.1", "<1.0.0"));

                // build metadata
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    "<1.2.3+20130313144700"
                ));
                assert!(crate::satisfies(
                    "1.2.3+20130313144700",
                    "<2.3.4+20130313144700"
                ));
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    "<1.2.2+20130313144700"
                ));

                // metadata in range, but not in version
                assert!(!crate::satisfies("0.1.0", "<0.1.0+lol"));
                assert!(!crate::satisfies("1.0.0", "<1.0.0+hunter2"));

                // metadata in version, but not in range
                assert!(!crate::satisfies("1.0.0+20200509", "<1.0.0"));
            }

            #[test]
            fn less_than_or_equal_to() {
                // major only
                assert!(crate::satisfies("1.2.3", "<=2"));
                assert!(!crate::satisfies("1.2.3", "<=1"));
                assert!(!crate::satisfies("1.2.3", "<=0"));

                // major and minor
                assert!(crate::satisfies("1.2.3", "<=1.3"));
                assert!(!crate::satisfies("1.2.3", "<=1.2"));
                assert!(!crate::satisfies("1.2.3", "<=1.1"));

                // major, minor, and patch
                assert!(crate::satisfies("1.2.3", "<=1.2.4"));
                assert!(crate::satisfies("1.2.3", "<=1.2.3"));
                assert!(!crate::satisfies("1.2.3", "<=1.2.2"));

                // prerelease
                assert!(crate::satisfies("1.2.3-pre.1", "<=1.2.3-pre.2"));
                assert!(crate::satisfies("1.2.3-pre.1", "<=1.2.3-pre.1"));
                assert!(!crate::satisfies("1.2.3-pre.1", "<=1.2.3-pre.0"));

                // prerelease in range, but not in version
                assert!(crate::satisfies("0.1.0", "<=1.0.0-pre.1"));
                assert!(crate::satisfies("1.0.0", "<=1.0.0-pre.1"));

                // prerelease in version, but not in range
                assert!(!crate::satisfies("1.0.0-beta.1", "<=1.0.0"));

                // build metadata
                assert!(crate::satisfies(
                    "1.2.3+20130313144700",
                    "<=1.2.3+20130313144700"
                ));
                assert!(crate::satisfies(
                    "1.2.3+20130313144700",
                    "<=2.3.4+20130313144700"
                ));
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    "<=1.2.2+20130313144700"
                ));

                // metadata in range, but not in version
                assert!(crate::satisfies("0.1.0", "<=0.1.0+lol"));
                assert!(crate::satisfies("1.0.0", "<=1.0.0+hunter2"));

                // metadata in version, but not in range
                assert!(crate::satisfies("1.0.0+20200509", "<=1.0.0"));
            }

            #[test]
            fn greater_than() {
                // major only
                assert!(!crate::satisfies("1.2.3", ">2"));
                assert!(!crate::satisfies("1.2.3", ">1"));
                assert!(crate::satisfies("1.2.3", ">0"));

                // major and minor
                assert!(!crate::satisfies("1.2.3", ">1.3"));
                assert!(!crate::satisfies("1.2.3", ">1.2"));
                assert!(crate::satisfies("1.2.3", ">1.1"));

                // major, minor, and patch
                assert!(!crate::satisfies("1.2.3", ">1.2.4"));
                assert!(!crate::satisfies("1.2.3", ">1.2.3"));
                assert!(crate::satisfies("1.2.3", ">1.2.2"));

                // prerelease
                assert!(!crate::satisfies("1.2.3-pre.1", ">1.2.3-pre.2"));
                assert!(!crate::satisfies("1.2.3-pre.1", ">1.2.3-pre.1"));
                assert!(crate::satisfies("1.2.3-pre.1", ">1.2.3-pre.0"));

                // prerelease in range, but not in version
                assert!(!crate::satisfies("0.1.0", ">1.0.0-pre.1"));
                assert!(crate::satisfies("1.0.0", ">1.0.0-pre.1"));

                // prerelease in version, but not in range
                assert!(!crate::satisfies("1.0.0-beta.1", ">1.0.0"));

                // build metadata
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    ">1.2.3+20130313144700"
                ));
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    ">2.3.4+20130313144700"
                ));
                assert!(crate::satisfies(
                    "1.2.3+20130313144700",
                    ">1.2.2+20130313144700"
                ));

                // metadata in range, but not in version
                assert!(!crate::satisfies("0.1.0", ">0.1.0+lol"));
                assert!(!crate::satisfies("1.0.0", ">1.0.0+hunter2"));

                // metadata in version, but not in range
                assert!(!crate::satisfies("1.0.0+20200509", ">1.0.0"));
            }

            #[test]
            fn greater_than_or_equal_to() {
                // major only
                assert!(!crate::satisfies("1.2.3", ">=2"));
                assert!(!crate::satisfies("1.2.3", ">=1"));
                assert!(crate::satisfies("1.2.3", ">=0"));

                // major and minor
                assert!(!crate::satisfies("1.2.3", ">=1.3"));
                assert!(!crate::satisfies("1.2.3", ">=1.2"));
                assert!(crate::satisfies("1.2.3", ">=1.1"));

                // major, minor, and patch
                assert!(!crate::satisfies("1.2.3", ">=1.2.4"));
                assert!(crate::satisfies("1.2.3", ">=1.2.3"));
                assert!(crate::satisfies("1.2.3", ">=1.2.2"));

                // prerelease
                assert!(!crate::satisfies("1.2.3-pre.1", ">=1.2.3-pre.2"));
                assert!(crate::satisfies("1.2.3-pre.1", ">=1.2.3-pre.1"));
                assert!(crate::satisfies("1.2.3-pre.1", ">=1.2.3-pre.0"));

                // prerelease in range, but not in version
                assert!(!crate::satisfies("0.1.0", ">=1.0.0-pre.1"));
                assert!(crate::satisfies("1.0.0", ">=1.0.0-pre.1"));

                // prerelease in version, but not in range
                assert!(!crate::satisfies("1.0.0-beta.1", ">=1.0.0"));

                // build metadata
                assert!(crate::satisfies(
                    "1.2.3+20130313144700",
                    ">=1.2.3+20130313144700"
                ));
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    ">=2.3.4+20130313144700"
                ));
                assert!(crate::satisfies(
                    "1.2.3+20130313144700",
                    ">=1.2.2+20130313144700"
                ));

                // metadata in range, but not in version
                assert!(crate::satisfies("0.1.0", ">=0.1.0+lol"));
                assert!(crate::satisfies("1.0.0", ">=1.0.0+hunter2"));

                // metadata in version, but not in range
                assert!(crate::satisfies("1.0.0+20200509", ">=1.0.0"));
            }

            #[test]
            fn equal() {
                // major only
                assert!(!crate::satisfies("1.2.3", "=2"));
                assert!(crate::satisfies("1.2.3", "=1"));
                assert!(!crate::satisfies("1.2.3", "=0"));

                // major and minor
                assert!(!crate::satisfies("1.2.3", "=1.3"));
                assert!(crate::satisfies("1.2.3", "=1.2"));
                assert!(!crate::satisfies("1.2.3", "=1.1"));

                // major, minor, and patch
                assert!(!crate::satisfies("1.2.3", "=1.2.4"));
                assert!(crate::satisfies("1.2.3", "=1.2.3"));
                assert!(!crate::satisfies("1.2.3", "=1.2.2"));

                // prerelease
                assert!(!crate::satisfies("1.2.3-pre.1", "=1.2.3-pre.2"));
                assert!(crate::satisfies("1.2.3-pre.1", "=1.2.3-pre.1"));
                assert!(!crate::satisfies("1.2.3-pre.1", "=1.2.3-pre.0"));

                // prerelease in range, but not in version
                assert!(!crate::satisfies("0.1.0", "=1.0.0-pre.1"));
                assert!(!crate::satisfies("1.0.0", "=1.0.0-pre.1"));

                // prerelease in version, but not in range
                assert!(!crate::satisfies("1.0.0-beta.1", "=1.0.0"));

                // build metadata
                assert!(crate::satisfies(
                    "1.2.3+20130313144700",
                    "=1.2.3+20130313144700"
                ));
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    "=2.3.4+20130313144700"
                ));
                assert!(!crate::satisfies(
                    "1.2.3+20130313144700",
                    "=1.2.2+20130313144700"
                ));

                // metadata in range, but not in version
                assert!(crate::satisfies("0.1.0", "=0.1.0+lol"));
                assert!(crate::satisfies("1.0.0", "=1.0.0+hunter2"));

                // metadata in version, but not in range
                assert!(crate::satisfies("1.0.0+20200509", "=1.0.0"));
            }

            #[test]
            fn tweet() {
                // https://twitter.com/izs/status/1259489276523196417

                // 2.0.1-pre.1 is not in >2.0.0
                assert!(!crate::satisfies("2.0.1-pre.1", ">2.0.0"));

                // 2.0.1-pre.1 is not in >=2.0.0-pre.0
                assert!(!crate::satisfies("2.0.1-pre.1", ">=2.0.0-pre.0"));

                // 2.0.1-pre.1 is in >=2.0.1-pre.0
                assert!(crate::satisfies("2.0.1-pre.1", ">=2.0.1-pre.0"));
            }
        }
    }

    mod advanced_ranges {
        #[test]
        fn x() {
            // major
            assert!(crate::satisfies("1.2.3", "X"));
            assert!(crate::satisfies("1.2.3", "x"));
            assert!(crate::satisfies("1.2.3", "*"));

            // empty string is special case for *
            assert!(crate::satisfies("1.2.3", ""));

            // minor
            assert!(crate::satisfies("1.2.3", "1"));
            assert!(crate::satisfies("1.2.3", "1.X"));
            assert!(crate::satisfies("1.2.3", "1.x"));
            assert!(crate::satisfies("1.2.3", "1.*"));

            // patch
            assert!(crate::satisfies("1.2.3", "1.2"));
            assert!(crate::satisfies("1.2.3", "1.2.X"));
            assert!(crate::satisfies("1.2.3", "1.2.x"));
            assert!(crate::satisfies("1.2.3", "1.2.*"));
        }

        #[test]
        fn tilde() {
            // major only
            assert!(crate::satisfies("1.0.0", "~1"));

            assert!(!crate::satisfies("0.1.0", "~1"));
            assert!(!crate::satisfies("2.0.0", "~1"));

            // major and minor
            assert!(crate::satisfies("1.2.0", "~1.2"));
            assert!(crate::satisfies("1.2.1", "~1.2"));

            assert!(!crate::satisfies("1.3.0", "~1.2"));

            // major, minor, and patch
            assert!(crate::satisfies("1.2.3", "~1.2.3"));

            assert!(!crate::satisfies("1.2.2", "~1.2.3"));
            assert!(!crate::satisfies("1.3.0", "~1.2.3"));

            // prerelease
            assert!(crate::satisfies("1.2.3-pre.1", "~1.2.3-pre.1"));

            assert!(!crate::satisfies("1.2.3-pre.1", "~1.2.3-pre.2"));

            // prerelease in range, but not in version
            assert!(crate::satisfies("1.2.3", "~1.2.3-pre.1"));

            // prerelease in version, but not in range
            assert!(!crate::satisfies("1.2.3-pre.1", "~1.2.3"));

            // build metadata
            assert!(crate::satisfies(
                "1.2.3+20130313144700",
                "~1.2.3+20130313144700"
            ));

            // metadata in range, but not in version
            assert!(crate::satisfies("0.1.0", "~0.1.0+lol"));

            // metadata in version, but not in range
            assert!(crate::satisfies("1.0.0+20200509", "~1.0.0"));
        }

        #[test]
        fn caret() {
            // bare versions are karet versions
            assert!(crate::satisfies("1.2.3", "1.0.0"));

            // major only
            assert!(crate::satisfies("1.0.0", "^1"));
            assert!(crate::satisfies("1.2.0", "^1"));
            assert!(crate::satisfies("1.2.3", "^1"));

            assert!(!crate::satisfies("1.2.3-rc.0", "^1"));

            assert!(!crate::satisfies("0.1.2", "^1"));
            assert!(!crate::satisfies("2.0.0", "^1"));

            // major and minor
            assert!(crate::satisfies("1.2.0", "^1.2"));
            assert!(crate::satisfies("1.2.3", "^1.2"));

            assert!(!crate::satisfies("1.2.3-rc.0", "^1.2"));

            assert!(!crate::satisfies("1.1.0", "^1.2"));
            assert!(!crate::satisfies("2.0.0", "^1.2"));

            // major, minor, and patch
            assert!(crate::satisfies("1.2.3", "^1.2.3"));

            assert!(!crate::satisfies("1.2.3-rc.0", "^1.2.3"));

            assert!(!crate::satisfies("1.2.2", "^1.2.3"));
            assert!(!crate::satisfies("2.0.0", "^1.2.3"));

            // prerelease
            assert!(crate::satisfies("1.2.3-pre.1", "^1.2.3-pre.1"));
            assert!(!crate::satisfies("1.2.3-pre.0", "^1.2.3-pre.1"));

            // prerelease in range, but not in version
            assert!(crate::satisfies("1.2.3", "^1.2.3-pre.1"));

            // prerelease in version, but not in range
            assert!(!crate::satisfies("1.2.3-pre.1", "^1.2.3"));

            // build metadata
            assert!(crate::satisfies(
                "1.2.3+20130313144700",
                "^1.2.3+20130313144700"
            ));

            // metadata in range, but not in version
            assert!(crate::satisfies("0.1.0", "^0.1.0+lol"));

            // metadata in version, but not in range
            assert!(crate::satisfies("1.0.0+20200509", "^1.0.0"));
        }
    }
}
