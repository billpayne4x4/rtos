use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Mapper, Page, PageTableFlags as Flags, PhysFrame, Size4KiB, FrameAllocator},
};

pub unsafe fn map_framebuffer_identity<M: Mapper<Size4KiB>, A: FrameAllocator<Size4KiB>>(
    mapper: &mut M,
    frame_alloc: &mut A,
    fb_phys_base: u64,
    fb_byte_len: usize,
) {
    let phys_start = PhysAddr::new(fb_phys_base);
    let virt_start = VirtAddr::new(fb_phys_base);
    let page_count: u64 = ((fb_byte_len as u64) + 0xFFF) / 0x1000;

    let mut i: u64 = 0;
    while i < page_count {
        let pa = phys_start + (i * 0x1000);
        let va = virt_start + (i * 0x1000);
        let frame = PhysFrame::containing_address(pa);
        let page  = Page::containing_address(va);
        mapper
            .map_to(page, frame, Flags::PRESENT | Flags::WRITABLE | Flags::NO_EXECUTE, frame_alloc)
            .expect("map_framebuffer_identity")
            .flush();
        i += 1;
    }
}
