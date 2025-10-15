#![no_std]
#![no_main]

mod boot;
mod rtk;

#[uefi::entry]
fn efi_main() -> uefi::Status {
    boot::entry::boot_entry()
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
