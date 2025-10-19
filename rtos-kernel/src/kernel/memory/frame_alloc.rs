use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use rtos_types::{boot_info::BootInfo, frame_range::FrameRange};

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    ranges: &'static [FrameRange],
    current_range_idx: usize,
    next_frame_in_range: u64,
}

impl BootInfoFrameAllocator {
    /// Creates a frame allocator from the boot info.
    ///
    /// # Safety
    /// The caller must guarantee that the frame ranges in BootInfo are valid
    /// and that no frames are used elsewhere.
    pub unsafe fn new_from_bootinfo(boot_info: &BootInfo) -> Self {
        let ranges_ptr = (boot_info.phys_mem_offset + boot_info.usable_frame_ranges_ptr) as *const FrameRange;
        let ranges_len = boot_info.usable_frame_ranges_len as usize;
        let ranges = core::slice::from_raw_parts(ranges_ptr, ranges_len);

        let mut allocator = Self {
            ranges,
            current_range_idx: 0,
            next_frame_in_range: 0,
        };

        // Initialize to the first frame in the first range
        if let Some(first_range) = ranges.get(0) {
            allocator.next_frame_in_range = first_range.start_frame;
        }

        allocator
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        while self.current_range_idx < self.ranges.len() {
            let range = &self.ranges[self.current_range_idx];
            let range_end = range.start_frame + range.frame_count;

            if self.next_frame_in_range < range_end {
                let frame_num = self.next_frame_in_range;
                self.next_frame_in_range += 1;
                let addr = frame_num * 4096;
                return Some(PhysFrame::containing_address(x86_64::PhysAddr::new(addr)));
            }

            // Move to next range
            self.current_range_idx += 1;
            if self.current_range_idx < self.ranges.len() {
                self.next_frame_in_range = self.ranges[self.current_range_idx].start_frame;
            }
        }

        None
    }
}