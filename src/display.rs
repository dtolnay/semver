use crate::{BuildMetadata, Prerelease, Version};
use std::fmt::{self, Debug, Display};

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
