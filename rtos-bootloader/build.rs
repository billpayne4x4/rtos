use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let logo_path = manifest_dir.join("../images/rtos-logo-transparent.raw");
    println!("cargo:rerun-if-changed={}", logo_path.display());
}
