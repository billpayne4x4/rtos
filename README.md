# RTOS — A Rust Operating System

<p align="center">
  <img src="images/rtos-logo-transparent.png" alt="gDocker interface screenshot" width="512">
</p>

**⚙️ RTOS** is a minimal, work-in-progress Rust-based operating system project with its own bootloader, kernel, and custom file formats.  
It currently boots under UEFI, uses an assembly trampoline to enter 64-bit Rust kernel mode, and successfully prints a message on screen.

> **ℹ️ Info:**
> I’m developing this project as a hands-on learning experience to deepen my understanding of operating systems, UEFI, and low-level Rust development.

---

## 🧭 Project Overview

### 🚀 Bootloader
- Written in Rust with an asm trampoline (`jump.asm`)
- Loads a custom `.RTOSK` kernel image from the EFI system partition
- Parses the image header, maps the kernel into memory, and jumps to its entry point
- Calls `jump_to_kernel`, transitioning to the kernel's `entry.asm` then kernel `main.rs`

### 🧩 Kernel
- Written in Rust with an assembly entry point (`entry.asm`) that bridges back into Rust code
- Successfully executes Rust kernel initialization and prints to the screen
- Uses a custom linker script (`linker.ld`)
- Output format: **RTOSK**, a custom kernel image format defined by `rtoskfmt`
- The kernel image is packed with `rtosk-gen`

### 📚 Libraries
- `libs/rtoskfmt`: defines RTOSK file format parsing and packing
- Future libraries will include hardware abstractions, drivers, and runtime components

### 🧰 Tools
- `tools/uefi-run.sh` – builds and runs the OS under QEMU with UEFI
- `tools/rtosk-gen` – packs kernel ELF into `.RTOSK`
- `tools/rtosk-inspect` – inspects `.RTOSK` images
- `libs/rtoskfmt` – shared library implementing the RTOSK format spec

---

## 🏗️ Build Instructions

Ensure you have a nightly Rust toolchain:
```bash
rustup default nightly
```

Build and run with the included helper:
```bash
./tools/uefi-run.sh dev
```

The build process:
1. Compiles bootloader and kernel (with NASM assembly)
2. Generates the `KERNEL.RTOSK` binary
3. Packs and copies required EFI files into a QEMU bootable image

---

## 🖥️ Example Output
```
bill@pop-os:/Repository/Projects/rtos-workspace$ ./tools/uefi-run.sh dev
   Compiling compiler_builtins v0.1.160 (/home/bill/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/compiler-builtins/compiler-builtins)
   Compiling core v0.0.0 (/home/bill/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core)
   Compiling proc-macro2 v1.0.101
   Compiling unicode-ident v1.0.19
   Compiling quote v1.0.41
   Compiling log v0.4.28
   Compiling nasm-rs v0.3.1
   Compiling rtos-bootloader v0.1.0 (/Repository/Projects/rtos-workspace/rtos-bootloader)
   Compiling syn v2.0.106
   Compiling ptr_meta_derive v0.3.1
   Compiling uefi-macros v0.18.1
   Compiling bitflags v2.9.4
   Compiling uguid v2.2.0
   Compiling bit_field v0.10.3
   Compiling ptr_meta v0.3.1
   Compiling cfg-if v1.0.3
   Compiling rtoskfmt v0.1.0 (/Repository/Projects/rtos-workspace/libs/rtoskfmt)
   Compiling panic-abort v0.3.2
   Compiling ucs2 v0.3.3
   Compiling uefi-raw v0.11.0
   Compiling uefi v0.35.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.70s
   Compiling compiler_builtins v0.1.160 (/home/bill/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/compiler-builtins/compiler-builtins)
   Compiling core v0.0.0 (/home/bill/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core)
   Compiling rustversion v1.0.22
   Compiling log v0.4.28
   Compiling nasm-rs v0.3.1
   Compiling rtos-kernel v0.1.0 (/Repository/Projects/rtos-workspace/rtos-kernel)
warning: rtos-kernel@0.1.0: Using linker script: /Repository/Projects/rtos-workspace/rtos-kernel/linker.ld
   Compiling bitflags v2.9.4
   Compiling bit_field v0.10.3
   Compiling volatile v0.4.6
   Compiling spin v0.9.8
   Compiling lazy_static v1.5.0
   Compiling x86_64 v0.15.2
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.56s
Kernel ELF: build/rtos-kernel/target/x86_64-unknown-none/debug/rtos-kernel
Kernel ELF size: 2604192 bytes
Kernel BIN: build/rtos-kernel/kernel.bin (76968 bytes)
   Compiling proc-macro2 v1.0.101
   Compiling unicode-ident v1.0.19
   Compiling quote v1.0.41
   Compiling log v0.4.28
   Compiling plain v0.2.3
   Compiling rtoskfmt v0.1.0 (/Repository/Projects/rtos-workspace/libs/rtoskfmt)
   Compiling syn v2.0.106
   Compiling scroll_derive v0.13.1
   Compiling scroll v0.13.0
   Compiling goblin v0.10.2
   Compiling rtosk-gen v0.1.0 (/Repository/Projects/rtos-workspace/tools/rtosk-gen)
    Finished `release` profile [optimized] target(s) in 3.17s
Packed: build/rtos-kernel/KERNEL.RTOSK (size 77048 bytes)
ESP ready:
  BOOTX64.EFI (130048 bytes)
  KERNEL.RTOSK (77048 bytes)
   Compiling rtoskfmt v0.1.0 (/Repository/Projects/rtos-workspace/libs/rtoskfmt)
   Compiling rtosk-inspect v0.1.0 (/Repository/Projects/rtos-workspace/tools/rtosk-inspect)
    Finished `release` profile [optimized] target(s) in 0.17s
Inspector (if built): build/rtosk-inspect/target/release/rtosk-inspect
=
=
=
BdsDxe: loading Boot0001 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)
BdsDxe: starting Boot0001 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)
BL: boot_entry start
BL: opened loaded_image
BL: opened SimpleFileSystem
BL: opened root dir
BL: opened KERNEL.RTOSK
BL: kernel_size 0x12cf8
BL: kernel blob loaded
BL: RTOSK off 0x0
BL: entry64 0x200000
BL: seg_count 0x1
BL: page_size 0x1000
BL: hdr.len 0x28
BL: segments_bytes 0x28
BL: seg[i] 0x0
  file_offset 0x50
  file_size 0x12ca8
  memory_addr 0x200000
  memory_size 0x12ca8
  flags 0x1
BL: stack_top 0x5d88000
BL: boot_info 0x5d7f000
BL: map seg 0x0
  start_page 0x200000
  end_page 0x213000
BL: map copied 0x12ca8
BL: entry (header.entry64) 0x200000
BL: calling trampoline (jump_to_kernel)
TRAMPOLINE: Entered
TRAMPOLINE: Entry 0000000000200000
TRAMPOLINE: Stack Top 0000000005D88000
TRAMPOLINE: Boot Info 0000000005D7F000
TRAMPOLINE: Memory at entry point:
48 83 EC 08 E8 F7 01 00 00 F4 EB FD CC CC CC CC 
50 48 89 3C 24 48 8D 3D E4 F3 00 00 BE 0E 00 00 
00 E8 CA 00 00 00 F4 EB FD CC CC CC CC CC CC CC 
50 40 88 F8 88 44 24 02 88 44 24 03 E8 8F 01 00 

TRAMPOLINE: Jumping to Kernel Entry Point...
IN KERNEL (asm→rust) ✅
```

🎉 This confirms that the trampoline correctly transfers execution from the bootloader to the kernel and into Rust code.

---

## 🧰 Tools

| Tool | Description |
|------|-------------|
| `uefi-run.sh` | Launches QEMU with UEFI firmware, builds and boots the OS. |
| `rtosk-gen` | Packs kernel ELF into `.RTOSK` custom format. |
| `rtosk-inspect` | Utility to inspect `.RTOSK` images. |

---

## 🧭 TODO / Roadmap

- [ ] Add memory management (parse boot info structure)
- [ ] Set up interrupt handling (IDT)
- [ ] Implement a proper console / VGA text mode
- [ ] Add keyboard input
- [ ] Build more RTOS features
- [ ] Support multiple concurrent processes (scheduling, isolation, etc.)
- [ ] Implement file system and device drivers
- [ ] Add network stack and IPC mechanisms
- [ ] Write tests for bootloader and kernel components

---

## 🧠 Design Notes

- **Language:** Rust (nightly, `no_std`)
- **Assembly:** x86_64 NASM
- **Boot target:** UEFI x86_64
- **Custom format:** RTOSK (kernel image), RTOSF (planned file format)
- **Transition:** bootloader → `jump.asm` → `entry.asm` → `rust::kmain()`

---

## ⚖️ License

You’re free to modify, rename, or redistribute this script **as long as you credit the original author**.  
Please include acknowledgment such as:

> *Originally developed by Bill Payne (payne.xyz).*

That’s all I ask — keep the credit visible if you redistribute or modify it.

---

## ✨ Summary

This project demonstrates:
- A custom Rust-based UEFI bootloader
- Cross-language execution path (ASM → Rust)
- Custom binary and file formats
- Real working boot flow ending in a printed kernel message

**It's alive** (well, barely blinking on the serial output—but alive nonetheless). 🚀