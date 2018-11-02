extern crate semver;

use semver::{Version,VersionReq};

fn main() -> Result<(),Box<::std::error::Error>> {
    let mut a = std::env::args();
    if a.len() != 2 && a.len() != 3 {
        eprintln!("Usage: semver_check [-q] <requirement> <version>");
        eprintln!("Examples:");
        eprintln!("    semver_check '^0.1.5' 0.1.8");
        eprintln!("    semver_check -q '^0.1.5' 0.1.8 && echo yes || echo no");
        Err("Invalid usage")?;
    }
    let quiet = a.len() == 3 && a.next() == Some("-q".to_string());
    let req = a.next().unwrap();
    let ver = a.next().unwrap();

    let r = VersionReq::parse(&req)?;
    let v = Version::parse(&ver)?;

    if r.matches(&v) {
        if !quiet {
            println!("Yes");
        }
        return Ok(())
    } else {
        if !quiet {
            println!("No");
        }
        ::std::process::exit(1);
    }
}
