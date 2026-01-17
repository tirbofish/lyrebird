use std::{ffi::OsStr, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut at_least_one = false;

    for entry in walkdir::WalkDir::new(&manifest_dir) {
        if let Ok(dir) = entry {
            if dir.path().extension() == Some(OsStr::new("slint")) {
                println!("cargo:rerun-if-changed={}", dir.path().display());
                slint_build::compile(dir.path()).unwrap();
                at_least_one = true;
            }
        }
    }

    if !at_least_one {
        panic!("Unable to locate any slint files within the dir {}", manifest_dir.display());
    } 
}