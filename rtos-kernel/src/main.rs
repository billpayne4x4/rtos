#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use uefi::prelude::*;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().ok();

    uefi::system::with_stdout(|stdout| {
        let _ = writeln!(stdout, "Kernel: hello from UEFI kernel (0.35)!");
    });

    loop {
        let _ = uefi::boot::stall(200_000);
    }
}
