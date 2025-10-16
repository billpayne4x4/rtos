#![no_std]
#![no_main]

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