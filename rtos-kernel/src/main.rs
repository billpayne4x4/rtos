#![no_std]
#![no_main]

/* ============== Minimal Rust kernel ============== */

#[repr(C)]
pub struct BootInfo { pub _reserved: u64 }

/* --- Tiny serial (COM1) --- */

const COM1: u16 = 0x3F8;

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val,
    options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let mut v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") port,
    options(nomem, nostack, preserves_flags));
    v
}

fn serial_can_tx() -> bool { unsafe { (inb(COM1 + 5) & 0x20) != 0 } }

fn serial_put(b: u8) {
    while !serial_can_tx() {}
    unsafe { outb(COM1, b) }
}

fn serial_write(s: &str) {
    for &b in s.as_bytes() {
        if b == b'\n' { serial_put(b'\r'); }
        serial_put(b);
    }
}

fn serial_init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
    }
}

/* --- Rust entry called from assembly --- */

#[no_mangle]
pub extern "C" fn kmain(_bi: *const BootInfo) -> ! {
    serial_init();
    serial_write("IN KERNEL (asm→rust) ✅\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

/* --- Panic handler --- */

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_write("kernel: PANIC\n");
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); } }
}
