#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

//extern crate alloc;

mod panic;
mod kernel;

#[path = "../../libs/serial-writer/src/lib.rs"]
mod serial_writer;

use rtos_types::boot_info::BootInfo;
use serial_writer::SerialWriter;
use x86_64::VirtAddr;

#[no_mangle]
pub extern "C" fn rtos_entry(boot_info: *mut rtos_types::boot_info::BootInfo) -> ! {
    unsafe { kmain(&*boot_info) }
}

#[no_mangle]
pub extern "C" fn kmain(bi: *const BootInfo) -> ! {
    SerialWriter::init();
    serial_logk!("Kernel initializing");

/*    let boot_info = unsafe { &*bi };
    let phys_mem_offset = VirtAddr::new(boot_info.phys_mem_offset);

    let mut mapper = unsafe { kernel::memory::init_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        kernel::memory::BootInfoFrameAllocator::new_from_bootinfo(boot_info)
    };

    serial_logk!("Setting up virtual heap and page mappings");
    unsafe {
        kernel::memory::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap init failed");
    }*/

    serial_logk!("Kernel initialized");

    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}