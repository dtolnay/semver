use std::env;
use std::process::Command;
use std::str;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let compiler = match rustc_minor_version() {
        Some(compiler) => compiler,
        None => return,
    };

    if compiler >= 80 {
        println!("cargo:rustc-check-cfg=cfg(no_nonzero_bitscan)");
        println!("cargo:rustc-check-cfg=cfg(no_track_caller)");
        println!("cargo:rustc-check-cfg=cfg(no_unsafe_op_in_unsafe_fn_lint)");
        println!("cargo:rustc-check-cfg=cfg(test_node_semver)");
    }

    if compiler < 46 {
        // #[track_caller].
        // https://blog.rust-lang.org/2020/08/27/Rust-1.46.0.html#track_caller
        println!("cargo:rustc-cfg=no_track_caller");
    }

    if compiler < 52 {
        // #![deny(unsafe_op_in_unsafe_fn)].
        // https://github.com/rust-lang/rust/issues/71668
        println!("cargo:rustc-cfg=no_unsafe_op_in_unsafe_fn_lint");
    }

    if compiler < 53 {
        // Efficient intrinsics for count-leading-zeros and count-trailing-zeros
        // on NonZero integers stabilized in 1.53.0. On many architectures these
        // are more efficient than counting zeros on ordinary zeroable integers.
        // https://doc.rust-lang.org/std/num/struct.NonZeroU64.html#method.leading_zeros
        // https://doc.rust-lang.org/std/num/struct.NonZeroU64.html#method.trailing_zeros
        println!("cargo:rustc-cfg=no_nonzero_bitscan");
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    pieces.next()?.parse().ok()
}
