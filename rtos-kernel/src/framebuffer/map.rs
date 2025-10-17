use crate::framebuffer::Framebuffer;
impl Framebuffer {
    pub unsafe fn remap_with_offset(&mut self, phys_mem_offset: u64) {
        let pa = PhysAddr::new(self.ptr as u64); // self.ptr was phys from bootloader
        let va = VirtAddr::new(phys_mem_offset + pa.as_u64());
        self.ptr = va.as_mut_ptr(); // now a valid VA you can write to
    }
}