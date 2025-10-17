use crate::framebuffer::Framebuffer;
use crate::console::Console;
use crate::serial_log;
use rtos_types::BootInfo;

pub struct KernelState {
    pub fb: Framebuffer,
}

impl KernelState {
    /// Build state from BootInfo and do a minimal visual bring-up.
    pub unsafe fn new(bi: &BootInfo) -> Self {
        serial_log!("Getting kernel state from boot info...");
        let mut state = Self { fb: Framebuffer::from_bootinfo(bi) };

        serial_log!("Clearing console...");
        // RTOS blue background
        state.fb.clear(0x00, 0x2D, 0x61);
        serial_log!("Done.");

        // One-shot console just to print a banner
        state.with_console(|c| {
            c.write_str("Kernel setup complete.\n");
        });

        state
    }

    pub fn with_console<F>(&mut self, mut f: F)
    where
        F: FnOnce(&mut Console<'_>),
    {
        let mut console = Console::new(&mut self.fb, (255, 255, 255), Some((0, 0, 0)));
        f(&mut console);
        // `console` drops here; `&mut fb` borrow ends cleanly.
    }
}
