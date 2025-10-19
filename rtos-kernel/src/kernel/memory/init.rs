use core::alloc::Layout;
use linked_list_allocator::LockedHeap;
use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        Mapper, Page, PhysFrame, Size4KiB, FrameAllocator, PageTableFlags, PageTable, OffsetPageTable,
    },
};
use rtos_types::{boot_info::BootInfo, frame_range::FrameRange};

pub const HEAP_START: u64 = 0xffff_8000_0000_0000;
pub const HEAP_SIZE: usize = 8 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(_: Layout) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

pub unsafe fn init_offset_page_table(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    fn active_l4(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
        let (frame, _) = Cr3::read();
        let phys = frame.start_address();
        let virt = phys_mem_offset + phys.as_u64();
        unsafe { &mut *virt.as_mut_ptr() }
    }
    OffsetPageTable::new(active_l4(phys_mem_offset), phys_mem_offset)
}

pub unsafe fn init_heap<M: Mapper<Size4KiB>, F: FrameAllocator<Size4KiB>>(
    mapper: &mut M,
    frame_allocator: &mut F,
) -> Result<(), ()> {
    let heap_start_va = VirtAddr::new(HEAP_START);
    let heap_end_va = heap_start_va + (HEAP_SIZE - 1) as u64;

    let start_page = Page::containing_address(heap_start_va);
    let end_page = Page::containing_address(heap_end_va);

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

    for page in Page::range_inclusive(start_page, end_page) {
        let frame: PhysFrame<Size4KiB> = frame_allocator.allocate_frame().ok_or(())?;
        let flush = mapper.map_to(page, frame, flags, frame_allocator).map_err(|_| ())?;
        flush.flush();
    }

    ALLOCATOR.lock().init((HEAP_START as usize) as *mut u8, HEAP_SIZE);
    Ok(())
}

