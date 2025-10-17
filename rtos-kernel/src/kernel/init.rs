use crate::kernel::state::KernelState;
use rtos_types::BootInfo;

pub unsafe fn kernel_init(bi: &BootInfo) -> KernelState {
    let mut state = KernelState::new(bi);

    // Example: write more after state is constructed
    state.with_console(|c| c.write_str("Hello from init!\n"));

    state
}
