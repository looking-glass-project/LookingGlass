fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    co_op_build_identity();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // A level change loads the destination while the viewer and outgoing
        // scene are still live. Windows' 1 MiB default exhausts the main thread
        // stack in that path; reserve 8 MiB, with pages committed on demand.
        // RUST_MIN_STACK only affects spawned threads, not the window thread.
        let arg = if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            "/STACK:8388608"
        } else {
            "-Wl,--stack,8388608"
        };
        println!("cargo:rustc-link-arg={arg}");
    }
}

/// Version strings alone cannot distinguish two development builds on `coop`.
/// Hash the actual sources and locked dependencies, without requiring Git or
/// embedding private paths. CRLF/LF checkouts produce the same identity.
fn co_op_build_identity() {
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    fn collect(dir: &Path, paths: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("read build source directory") {
            let path = entry.expect("read build source entry").path();
            if path.is_dir() {
                collect(&path, paths);
            } else {
                paths.push(path);
            }
        }
    }
    let mut paths = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.lock"),
        PathBuf::from("build.rs"),
    ];
    collect(Path::new("src"), &mut paths);
    paths.sort();
    let mut hash = Sha256::new();
    hash.update(b"Looking Glass co-op source identity 1\0");
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path.to_string_lossy().replace('\\', "/");
        let raw = fs::read(&path).expect("read build source");
        let bytes = match std::str::from_utf8(&raw) {
            Ok(text) => text.replace("\r\n", "\n").into_bytes(),
            Err(_) => raw,
        };
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    // Observe added/removed source files as well as edits to known files.
    println!("cargo:rerun-if-changed=src");
    hash.update(std::env::var("TARGET").expect("build target").as_bytes());
    println!("cargo:rustc-env=LOOKING_GLASS_BUILD={:x}", hash.finalize());
}
