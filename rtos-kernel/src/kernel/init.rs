use rtos_types::BootInfo;
use crate::kernel::state::KernelState;
use crate::framebuffer::validate_framebuffer_soft;
use crate::serial_log;

pub unsafe fn kernel_init(bi: &BootInfo) -> KernelState {
    let mut state = KernelState::new(bi);

    match validate_framebuffer_soft(&mut state.fb) {
        Ok(()) => serial_log!("Framebuffer validated OK"),
        Err(e) => {
            serial_log!("Framebuffer soft-validation failed");
            serial_log!("Width: ", state.fb.width as usize);
            serial_log!("Height: ", state.fb.height as usize);
            serial_log!("Stride: ", state.fb.stride as usize);
            serial_log!("Ptr: "); crate::utils::SerialWriter::write_hex(state.fb.ptr as usize); crate::utils::SerialWriter::write("\n");
            let _ = e; // keep it simple for now; you can pretty-print later
        }
    }

    state
}
