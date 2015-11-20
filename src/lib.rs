#[macro_use]
extern crate nom;

pub mod parser;

use std::result;

/// A SemVer Version
#[derive(PartialEq,Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    pre: Option<String>,
    build: Option<String>,
}

/// An error type for this crate
///
/// Currently, just a generic error. Will make this nicer later.
#[derive(PartialEq,Debug)]
enum SemVerError {
    GenericError,
}

/// A Result type for errors
pub type Result<T> = result::Result<T, SemVerError>;

impl From<()> for SemVerError {
    fn from(_: ()) -> SemVerError {
        SemVerError::GenericError
    }
}

impl Version {
    /// Create a Version from a string
    ///
    /// Currently supported: x, x.y, and x.y.z versions.
    pub fn parse(version: &str) -> Result<Version> {
        Ok(try!(parser::try_parse(version.trim().as_bytes())))
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use super::SemVerError;

    #[test]
    fn test_parse() {
        assert_eq!(Version::parse(""), Err(SemVerError::GenericError));
        assert_eq!(Version::parse("  "), Err(SemVerError::GenericError));
        //assert_eq!(Version::parse("1"), Err(SemVerError::GenericError));
        //assert_eq!(Version::parse("1.2"), Err(SemVerError::GenericError));
        //assert_eq!(Version::parse("1.2.3-"), Err(SemVerError::GenericError));
        assert_eq!(Version::parse("a.b.c"), Err(SemVerError::GenericError));

        //assert_eq!(Version::parse("1.2.3 abc"), Err(SemVerError::GenericError));

        assert!(Version::parse("1.2.3") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: None,
        }));
        assert!(Version::parse("  1.2.3  ") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: None,
        }));
        assert!(Version::parse("1.2.3-alpha1") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(String::from("alpha1")),
            build: None,
        }));
        assert!(Version::parse("  1.2.3-alpha1  ") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(String::from("alpha1")),
            build: None
        }));
        assert!(Version::parse("1.2.3+build5") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: Some(String::from("build5")),
        }));
        assert!(Version::parse("  1.2.3+build5  ") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: Some(String::from("build5")),
        }));
        assert!(Version::parse("1.2.3-alpha1+build5") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(String::from("alpha1")),
            build: Some(String::from("build5")),
        }));
        assert!(Version::parse("  1.2.3-alpha1+build5  ") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(String::from("alpha1")),
            build: Some(String::from("build5")),
        }));
        assert!(Version::parse("1.2.3-1.alpha1.9+build5.7.3aedf  ") == Ok(Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(String::from("1.alpha1.9")),
            build: Some(String::from("build5.7.3aedf")),
        }));
        assert_eq!(Version::parse("0.4.0-beta.1+0851523"), Ok(Version {
            major: 0,
            minor: 4,
            patch: 0,
            pre: Some(String::from("beta.1")),
            build: Some(String::from("0851523")),
        }));

    }

    /*
    #[test]
    fn test_increment_patch() {
        let mut buggy_release = Version::parse("0.1.0").unwrap();
        buggy_release.increment_patch();
        assert_eq!(buggy_release, Version::parse("0.1.1").unwrap());
    }

    #[test]
    fn test_increment_minor() {
        let mut feature_release = Version::parse("1.4.6").unwrap();
        feature_release.increment_minor();
        assert_eq!(feature_release, Version::parse("1.5.0").unwrap());
    }

    #[test]
    fn test_increment_major() {
        let mut chrome_release = Version::parse("46.1.246773").unwrap();
        chrome_release.increment_major();
        assert_eq!(chrome_release, Version::parse("47.0.0").unwrap());
    }

    #[test]
    fn test_increment_keep_prerelease() {
        let mut release = Version::parse("1.0.0-alpha").unwrap();
        release.increment_patch();

        assert_eq!(release, Version::parse("1.0.1").unwrap());

        release.increment_minor();

        assert_eq!(release, Version::parse("1.1.0").unwrap());

        release.increment_major();

        assert_eq!(release, Version::parse("2.0.0").unwrap());
    }


    #[test]
    fn test_increment_clear_metadata() {
        let mut release = Version::parse("1.0.0+4442").unwrap();
        release.increment_patch();

        assert_eq!(release, Version::parse("1.0.1").unwrap());
        release = Version::parse("1.0.1+hello").unwrap();

        release.increment_minor();

        assert_eq!(release, Version::parse("1.1.0").unwrap());
        release = Version::parse("1.1.3747+hello").unwrap();

        release.increment_major();

        assert_eq!(release, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_eq() {
        assert_eq!(Version::parse("1.2.3"), Version::parse("1.2.3"));
        assert_eq!(Version::parse("1.2.3-alpha1"), Version::parse("1.2.3-alpha1"));
        assert_eq!(Version::parse("1.2.3+build.42"), Version::parse("1.2.3+build.42"));
        assert_eq!(Version::parse("1.2.3-alpha1+42"), Version::parse("1.2.3-alpha1+42"));
        assert_eq!(Version::parse("1.2.3+23"), Version::parse("1.2.3+42"));
    }

    #[test]
    fn test_ne() {
        assert!(Version::parse("0.0.0")       != Version::parse("0.0.1"));
        assert!(Version::parse("0.0.0")       != Version::parse("0.1.0"));
        assert!(Version::parse("0.0.0")       != Version::parse("1.0.0"));
        assert!(Version::parse("1.2.3-alpha") != Version::parse("1.2.3-beta"));
    }

    #[test]
    fn test_show() {
        assert_eq!(format!("{}", Version::parse("1.2.3").unwrap()),
                   "1.2.3".to_string());
        assert_eq!(format!("{}", Version::parse("1.2.3-alpha1").unwrap()),
                   "1.2.3-alpha1".to_string());
        assert_eq!(format!("{}", Version::parse("1.2.3+build.42").unwrap()),
                   "1.2.3+build.42".to_string());
        assert_eq!(format!("{}", Version::parse("1.2.3-alpha1+42").unwrap()),
                   "1.2.3-alpha1+42".to_string());
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Version::parse("1.2.3").unwrap().to_string(), "1.2.3".to_string());
        assert_eq!(Version::parse("1.2.3-alpha1").unwrap().to_string(), "1.2.3-alpha1".to_string());
        assert_eq!(Version::parse("1.2.3+build.42").unwrap().to_string(), "1.2.3+build.42".to_string());
        assert_eq!(Version::parse("1.2.3-alpha1+42").unwrap().to_string(), "1.2.3-alpha1+42".to_string());
    }

    #[test]
    fn test_lt() {
        assert!(Version::parse("0.0.0")          < Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.0.0")          < Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.0")          < Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3-alpha1")   < Version::parse("1.2.3"));
        assert!(Version::parse("1.2.3-alpha1")   < Version::parse("1.2.3-alpha2"));
        assert!(!(Version::parse("1.2.3-alpha2") < Version::parse("1.2.3-alpha2")));
        assert!(!(Version::parse("1.2.3+23")     < Version::parse("1.2.3+42")));
    }

    #[test]
    fn test_le() {
        assert!(Version::parse("0.0.0")        <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.0.0")        <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.0")        <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3-alpha1") <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3-alpha2") <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3+23")     <= Version::parse("1.2.3+42"));
    }

    #[test]
    fn test_gt() {
        assert!(Version::parse("1.2.3-alpha2")   > Version::parse("0.0.0"));
        assert!(Version::parse("1.2.3-alpha2")   > Version::parse("1.0.0"));
        assert!(Version::parse("1.2.3-alpha2")   > Version::parse("1.2.0"));
        assert!(Version::parse("1.2.3-alpha2")   > Version::parse("1.2.3-alpha1"));
        assert!(Version::parse("1.2.3")          > Version::parse("1.2.3-alpha2"));
        assert!(!(Version::parse("1.2.3-alpha2") > Version::parse("1.2.3-alpha2")));
        assert!(!(Version::parse("1.2.3+23")     > Version::parse("1.2.3+42")));
    }

    #[test]
    fn test_ge() {
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("0.0.0"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.0.0"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.2.0"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.2.3-alpha1"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3+23")     >= Version::parse("1.2.3+42"));
    }

    #[test]
    fn test_prerelease_check() {
        assert!(Version::parse("1.0.0").unwrap().is_prerelease() == false);
        assert!(Version::parse("0.0.1").unwrap().is_prerelease() == false);
        assert!(Version::parse("4.1.4-alpha").unwrap().is_prerelease());
        assert!(Version::parse("1.0.0-beta294296").unwrap().is_prerelease());
    }

    #[test]
    fn test_spec_order() {
        let vs = ["1.0.0-alpha",
                  "1.0.0-alpha.1",
                  "1.0.0-alpha.beta",
                  "1.0.0-beta",
                  "1.0.0-beta.2",
                  "1.0.0-beta.11",
                  "1.0.0-rc.1",
                  "1.0.0"];
        let mut i = 1;
        while i < vs.len() {
            let a = Version::parse(vs[i-1]).unwrap();
            let b = Version::parse(vs[i]).unwrap();
            assert!(a < b);
            i += 1;
        }
    }
    */
}
