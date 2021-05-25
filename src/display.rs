use crate::{BuildMetadata, Comparator, Op, Prerelease, Version, VersionReq};
use core::fmt::{self, Debug, Display};

impl Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.major, formatter)?;
        formatter.write_str(".")?;
        Display::fmt(&self.minor, formatter)?;
        formatter.write_str(".")?;
        Display::fmt(&self.patch, formatter)?;
        if !self.pre.is_empty() {
            formatter.write_str("-")?;
            Display::fmt(&self.pre, formatter)?;
        }
        if !self.build.is_empty() {
            formatter.write_str("+")?;
            Display::fmt(&self.build, formatter)?;
        }
        Ok(())
    }
}

impl Display for VersionReq {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.comparators.is_empty() {
            return formatter.write_str("*");
        }
        for (i, comparator) in self.comparators.iter().enumerate() {
            if i > 0 {
                formatter.write_str(", ")?;
            }
            Display::fmt(comparator, formatter)?;
        }
        Ok(())
    }
}

impl Display for Comparator {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let op = match self.op {
            Op::Exact => "=",
            Op::Greater => ">",
            Op::GreaterEq => ">=",
            Op::Less => "<",
            Op::LessEq => "<=",
            Op::Tilde => "~",
            Op::Caret => "^",
            Op::Wildcard => "",
            #[cfg(no_non_exhaustive)]
            Op::__NonExhaustive => unreachable!(),
        };
        formatter.write_str(op)?;
        Display::fmt(&self.major, formatter)?;
        if let Some(minor) = &self.minor {
            formatter.write_str(".")?;
            Display::fmt(minor, formatter)?;
            if let Some(patch) = &self.patch {
                formatter.write_str(".")?;
                Display::fmt(patch, formatter)?;
                if !self.pre.is_empty() {
                    formatter.write_str("-")?;
                    Display::fmt(&self.pre, formatter)?;
                }
            } else if self.op == Op::Wildcard {
                formatter.write_str(".*")?;
            }
        } else if self.op == Op::Wildcard {
            formatter.write_str(".*")?;
        }
        Ok(())
    }
}

impl Display for Prerelease {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Display for BuildMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Debug for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut debug = formatter.debug_struct("Version");
        debug
            .field("major", &self.major)
            .field("minor", &self.minor)
            .field("patch", &self.patch);
        if !self.pre.is_empty() {
            debug.field("pre", &self.pre);
        }
        if !self.build.is_empty() {
            debug.field("build", &self.build);
        }
        debug.finish()
    }
}

impl Debug for Prerelease {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Prerelease(\"")?;
        Display::fmt(self, formatter)?;
        formatter.write_str("\")")?;
        Ok(())
    }
}

impl Debug for BuildMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("BuildMetadata(\"")?;
        Display::fmt(self, formatter)?;
        formatter.write_str("\")")?;
        Ok(())
    }
}
