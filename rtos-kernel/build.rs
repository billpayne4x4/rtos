fn main() {
    // Linker script
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_script = format!("{}/linker.ld", manifest_dir);
    println!("cargo:rustc-link-arg=-T{}", linker_script);
    println!("cargo:warning=Using linker script: {}", linker_script);
    println!("cargo:rerun-if-changed=linker.ld");

    // Assemble the kernel entry (ELF64)
    println!("cargo:rerun-if-changed=src/entry.asm");
    let mut b = nasm_rs::Build::new();
    b.file("src/entry.asm").flag("-felf64");
    b.compile("kernel_entry");

    // Tell rustc to link the generated static lib from OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=kernel_entry");
}
