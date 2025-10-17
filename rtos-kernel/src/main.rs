#![no_std]
#![no_main]

mod panic;
mod kernel;
mod framebuffer;
mod types;
mod console;
mod utils;

use kernel::kernel_init;
use rtos_types::{BootInfo, FramebufferInfo, FramebufferFormat};
use crate::utils::SerialWriter;


#[no_mangle]
pub extern "C" fn kmain(bi: *const BootInfo) -> ! {
    SerialWriter::init();
    serial_log!("Kernel initializing...");

    let bi = unsafe { &*bi };
    let _state = unsafe { kernel_init(bi) };
    serial_log!("Kernel initialized.");


    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

