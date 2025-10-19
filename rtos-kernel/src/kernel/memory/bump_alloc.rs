use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// A simple bump allocator for the kernel heap.
/// This is not efficient for general use but works fine for basic needs.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: AtomicUsize::new(0),
        }
    }

    // was: pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next.store(heap_start, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // Align the next pointer
        let mut current = self.next.load(Ordering::Relaxed);
        loop {
            let aligned = (current + align - 1) & !(align - 1);
            let new_next = aligned + size;

            // Check if we have enough space
            if new_next > self.heap_end {
                return ptr::null_mut();
            }

            // Try to update next atomically
            match self.next.compare_exchange(
                current,
                new_next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return aligned as *mut u8,
                Err(actual) => current = actual,
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocators don't free individual allocations
        // Memory is only reclaimed when the entire allocator is reset
    }
}

// For use in memory.rs
use spin::Mutex;

pub struct LockedBumpAllocator {
    inner: Mutex<BumpAllocator>,
}

impl LockedBumpAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(BumpAllocator::new()),
        }
    }

    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        self.inner.lock().init(heap_start, heap_size);
    }
}

unsafe impl GlobalAlloc for LockedBumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }
}