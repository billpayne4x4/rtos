use core::mem::size_of;
use rtos_types::{boot_info::BootInfo, frame_range::FrameRange};
use uefi::{
    Status,
    boot::{self, AllocateType},
    mem::memory_map::{MemoryType, MemoryMap},
};

/// Populate `bi` in-place from UEFI: coalesce usable RAM into a FrameRange table,
/// stash its physical pointer/len, and set `phys_mem_offset`.
/// Also excludes the framebuffer memory from usable ranges.
/// Returns `Ok(())` on success.
pub fn collect_bootinfo_from_uefi_inplace(
    bi: &mut BootInfo,
    phys_mem_offset: u64,
) -> Result<(), Status> {
    // --- Get memory map (UEFI 0.35 API) ---
    let mmap = boot::memory_map(MemoryType::LOADER_DATA)
        .map_err(|e| e.status())?;

    // Count conventional memory regions
    let conv_count = mmap.entries()
        .filter(|d| d.ty == MemoryType::CONVENTIONAL)
        .count();

    // --- Allocate page-backed storage for coalesced FrameRange[] ---
    let needed_bytes = conv_count.saturating_mul(size_of::<FrameRange>()).max(size_of::<FrameRange>());
    let needed_pages = ((needed_bytes + 4095) / 4096) as usize;

    let ranges_pa = unsafe {
        boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, needed_pages)
            .map_err(|e| e.status())?
    };
    let ranges_ptr = ranges_pa.as_ptr() as *mut FrameRange;
    let mut out_len: usize = 0;

    // --- Coalesce directly into the allocated table ---
    let mut cur: Option<FrameRange> = None;
    for d in mmap.entries() {
        if d.ty != MemoryType::CONVENTIONAL { continue; }
        let first = (d.phys_start / 4096) as u64;
        let count = d.page_count as u64;
        let seg = FrameRange { start_frame: first, frame_count: count };

        cur = Some(match cur {
            None => seg,
            Some(prev) => {
                let prev_end = prev.start_frame + prev.frame_count;
                if prev_end == seg.start_frame {
                    FrameRange {
                        start_frame: prev.start_frame,
                        frame_count: prev.frame_count + seg.frame_count
                    }
                } else {
                    unsafe { ranges_ptr.add(out_len).write(prev); }
                    out_len += 1;
                    seg
                }
            }
        });
    }
    if let Some(last) = cur {
        unsafe { ranges_ptr.add(out_len).write(last); }
        out_len += 1;
    }

    // --- Carve out the table's own pages from the usable set ---
    let table_first = (ranges_pa.as_ptr() as u64 / 4096) as u64;
    let table_count = needed_pages as u64;

    out_len = carve_out_range(ranges_ptr, out_len, table_first, table_count);

    // --- Carve out framebuffer from usable ranges ---
    if bi.framebuffer.info.base != 0 && bi.framebuffer.info.size > 0 {
        let fb_first = bi.framebuffer.info.base / 4096;
        let fb_count = ((bi.framebuffer.info.size + 4095) / 4096) as u64;
        out_len = carve_out_range(ranges_ptr, out_len, fb_first, fb_count);
    }

    // --- Update BootInfo in-place ---
    bi.phys_mem_offset = phys_mem_offset;
    bi.usable_frame_ranges_ptr = ranges_pa.as_ptr() as u64; // physical address
    bi.usable_frame_ranges_len = out_len as u64;

    Ok(())
}

/// Helper function to carve out a frame range from the usable set.
/// Returns the new length of the array.
fn carve_out_range(
    ranges_ptr: *mut FrameRange,
    mut out_len: usize,
    exclude_first: u64,
    exclude_count: u64,
) -> usize {
    let exclude_end = exclude_first + exclude_count;

    let mut i = 0usize;
    while i < out_len {
        let r = unsafe { ranges_ptr.add(i).read() };
        let r_end = r.start_frame + r.frame_count;

        let overlap = !(r_end <= exclude_first || exclude_end <= r.start_frame);
        if !overlap {
            i += 1;
            continue;
        }

        // Full overlap: remove this range
        if exclude_first <= r.start_frame && exclude_end >= r_end {
            out_len -= 1;
            if i < out_len {
                let last = unsafe { ranges_ptr.add(out_len).read() };
                unsafe { ranges_ptr.add(i).write(last); }
            }
            continue;
        }

        // Trim start
        if exclude_first <= r.start_frame && exclude_end < r_end {
            let new_r = FrameRange {
                start_frame: exclude_end,
                frame_count: r_end - exclude_end
            };
            unsafe { ranges_ptr.add(i).write(new_r); }
            i += 1;
            continue;
        }

        // Trim end
        if exclude_first > r.start_frame && exclude_end >= r_end {
            let new_r = FrameRange {
                start_frame: r.start_frame,
                frame_count: exclude_first - r.start_frame
            };
            unsafe { ranges_ptr.add(i).write(new_r); }
            i += 1;
            continue;
        }

        // Middle split
        let left = FrameRange {
            start_frame: r.start_frame,
            frame_count: exclude_first - r.start_frame
        };
        let right = FrameRange {
            start_frame: exclude_end,
            frame_count: r_end - exclude_end
        };
        unsafe { ranges_ptr.add(i).write(left); }
        unsafe { ranges_ptr.add(out_len).write(right); }
        out_len += 1;
        i += 1;
    }

    out_len
}