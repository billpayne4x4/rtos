fn main() {
    // Rebuild if the assembly file changes
    println!("cargo:rerun-if-changed=src/jump.asm");

    // Assemble jump.asm -> libjump_asm.a into OUT_DIR
    let mut b = nasm_rs::Build::new();
    b.file("src/jump.asm").flag("-fwin64");

    let _ = b.compile("jump_asm").expect("nasm failed");

    // Make sure the linker can find and link the static lib
    let out = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    println!("cargo:rustc-link-search=native={out}");
    println!("cargo:rustc-link-lib=static=jump_asm");
}
