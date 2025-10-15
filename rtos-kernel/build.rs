fn main() {
    // Keep the linker script wired up
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_script = format!("{}/linker.ld", manifest_dir);
    println!("cargo:rustc-link-search=native={manifest_dir}");
    println!("cargo:rustc-link-arg=-T{linker_script}");
    println!("cargo:warning=Using linker script: {linker_script}");
    println!("cargo:rerun-if-changed=linker.ld");

    // Assemble the kernel entry for **ELF64**, not win64/COFF.
    println!("cargo:rerun-if-changed=src/entry.asm");
    let mut b = nasm_rs::Build::new();
    b.file("src/entry.asm").flag("-felf64");
    let _ = b.compile("kernel_entry"); // silence unused_must_use warning
}
