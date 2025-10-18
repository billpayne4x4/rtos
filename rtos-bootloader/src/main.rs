#![no_std]
#![no_main]

#[path = "../../libs/serial-writer/src/lib.rs"]
mod serial_writer;

use uefi::Status;

mod boot;
mod rtosk;

#[uefi::entry]
fn efi_main() -> Status {
    boot::entry::boot_entry()
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}