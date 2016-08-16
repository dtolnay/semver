extern crate semver;
extern crate crates_index;
extern crate tempdir;

// This test checks to see if every existing crate parses successfully. Important to not break the
// Rust universe!

#[cfg(feature = "ci")]
#[test]
fn test_regressions() {
    use tempdir::TempDir;
    use crates_index::Index;
    use semver::Version;
    use semver::VersionReq;

    let dir = TempDir::new("semver").unwrap();
    let index = Index::new(dir.into_path());
    index.clone().unwrap();

    for krate in index.crates() {
        println!("testing crate: {}", krate.name());
        for version in krate.versions() {
            let v = version.version();
            println!("testing version: {}", v);
            assert!(Version::parse(v).is_ok(), "failed: {} ({})", version.name(), v);

            for dependency in version.dependencies() {
                let r = dependency.requirement();
                println!("testing dependency: {}", r);
                assert!(VersionReq::parse(r).is_ok(), "failed: {} ({})", dependency.name(), r);
            }
        }
    }
}
