use crate::{BuildMetadata, Prerelease};
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
