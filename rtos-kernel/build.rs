fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_script_path = format!("{}/linker.ld", manifest_dir);
    println!("cargo:rustc-link-arg=-T{}", linker_script_path);
    println!("cargo:warning=Using linker script: {}", linker_script_path);
    println!("cargo:rerun-if-changed=linker.ld");
}