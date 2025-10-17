use core::fmt::{self, Write};

pub struct SerialWriter;

impl SerialWriter {
    const COM1: u16 = 0x3F8;

    #[inline(always)]
    unsafe fn outb(port: u16, val: u8) {
        core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
        );
    }

    #[inline(always)]
    unsafe fn inb(port: u16) -> u8 {
        let mut v: u8;
        core::arch::asm!(
        "in al, dx",
        out("al") v,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
        );
        v
    }

    #[inline(always)]
    fn serial_can_tx() -> bool {
        unsafe { (Self::inb(Self::COM1 + 5) & 0x20) != 0 }
    }

    #[inline(always)]
    fn serial_put(b: u8) {
        while !Self::serial_can_tx() {}
        unsafe { Self::outb(Self::COM1, b) }
    }

    pub fn init() {
        unsafe {
            Self::outb(Self::COM1 + 1, 0x00);
            Self::outb(Self::COM1 + 3, 0x80);
            Self::outb(Self::COM1 + 0, 0x01);
            Self::outb(Self::COM1 + 1, 0x00);
            Self::outb(Self::COM1 + 3, 0x03);
            Self::outb(Self::COM1 + 2, 0xC7);
            Self::outb(Self::COM1 + 4, 0x0B);
        }
    }

    pub fn write(s: &str) {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b == b'\n' {
                Self::serial_put(b'\r');
            }
            Self::serial_put(b);
            i += 1;
        }
    }

    pub fn write_u32(mut n: u32) {
        if n == 0 {
            Self::serial_put(b'0');
            return;
        }

        let mut buf = [0u8; 10];
        let mut i = 0;
        while n > 0 {
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            Self::serial_put(buf[i]);
        }
    }

    pub fn write_usize(n: usize) {
        Self::write_u64(n as u64);
    }

    pub fn write_u64(mut n: u64) {
        if n == 0 {
            Self::serial_put(b'0');
            return;
        }

        let mut buf = [0u8; 20];
        let mut i = 0;
        while n > 0 {
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            Self::serial_put(buf[i]);
        }
    }

    pub fn write_hex(mut n: usize) {
        Self::write("0x");
        if n == 0 {
            Self::serial_put(b'0');
            return;
        }

        let mut buf = [0u8; 16];
        let mut i = 0;
        while n > 0 {
            let digit = (n & 0xF) as u8;
            buf[i] = if digit < 10 { b'0' + digit } else { b'a' + digit - 10 };
            n >>= 4;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            Self::serial_put(buf[i]);
        }
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::write(s);
        Ok(())
    }


}

#[macro_export]
macro_rules! serial_println {
    () => {{
        $crate::utils::log_serial::SerialWriter::write("\n");
    }};
    ($s:literal) => {{
        $crate::utils::log_serial::SerialWriter::write(concat!($s, "\n"));
    }};
}

#[macro_export]
macro_rules! serial_log {
    ($s:literal) => {{
        $crate::utils::log_serial::SerialWriter::write(concat!("K: ", $s, "\n"));
    }};
}

