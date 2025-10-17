use uefi::data_types::CStr16;
use uefi::system;

pub fn write_line(text: &str) {
    let mut buf = [0u16; 192];
    let mut i = 0usize;
    for &b in text.as_bytes() {
        if i + 3 >= buf.len() { break; }
        buf[i] = b as u16;
        i += 1;
    }
    if i + 3 < buf.len() {
        buf[i] = b'\r' as u16; i += 1;
        buf[i] = b'\n' as u16; i += 1;
        buf[i] = 0;
        // SAFETY: buf is NUL-terminated and bounded.
        let s16: &CStr16 = unsafe { CStr16::from_u16_with_nul_unchecked(&buf[..=i]) };
        let _ = system::with_stdout(|out| out.output_string(s16));
    }
}

pub fn write_hex(label: &str, value: u64) {
    let mut tmp = [0u8; 64];
    let mut n = 0usize;
    for &b in label.as_bytes() { tmp[n] = b; n += 1; }
    tmp[n] = b' '; n += 1; tmp[n] = b'0'; n += 1; tmp[n] = b'x'; n += 1;

    let mut started = false;
    for i in (0..16).rev() {
        let nyb = ((value >> (i*4)) & 0xF) as u8;
        let ch = if nyb < 10 { b'0' + nyb } else { b'a' + (nyb - 10) };
        if !started && ch == b'0' && i != 0 { continue; }
        started = true; tmp[n] = ch; n += 1;
    }

    if let Ok(s) = core::str::from_utf8(&tmp[..n]) {
        write_line(s);
    }
}

pub fn clear_screen() {
    let _ = system::with_stdout(|out| out.clear());
}