#![no_std]


use core::fmt::{self, Write};

pub struct SerialWriter;

impl SerialWriter {
    const COM1: u16 = 0x3F8;

    #[inline(never)]
    unsafe fn outb(port: u16, val: u8) {
        core::arch::asm!(
        "outb %al, %dx",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags, att_syntax)
        );
    }

    #[inline(never)]
    unsafe fn inb(port: u16) -> u8 {
        let v: u8;
        core::arch::asm!(
        "inb %dx, %al",
        out("al") v,
        in("dx") port,
        options(nomem, nostack, preserves_flags, att_syntax)
        );
        v
    }

    #[inline(never)]
    fn serial_can_tx() -> bool {
        unsafe { (Self::inb(Self::COM1 + 5) & 0x20) != 0 }
    }

    #[inline(never)]
    fn serial_put(b: u8) {
        while !Self::serial_can_tx() {}
        unsafe { Self::outb(Self::COM1, b) }
    }

    #[inline(never)]
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
        $crate::serial_writer::SerialWriter::write("\n");
    }};
    ($s:literal) => {{
        $crate::serial_writer::SerialWriter::write(concat!($s, "\n"));
    }};
    ($s:literal, $val:expr) => {{
        $crate::serial_writer::SerialWriter::write($s);
        $crate::serial_writer::SerialWriter::write(" ");
        $crate::serial_writer::SerialWriter::write_usize($val as usize);
        $crate::serial_writer::SerialWriter::write("\n");
    }};
    ($s:literal, hex $val:expr) => {{
        $crate::serial_writer::SerialWriter::write($s);
        $crate::serial_writer::SerialWriter::write(" ");
        $crate::serial_writer::SerialWriter::write_hex($val as usize);
        $crate::serial_writer::SerialWriter::write("\n");
    }};
    ($s:literal, $val1:expr, $val2:expr) => {{
        $crate::serial_writer::SerialWriter::write($s);
        $crate::serial_writer::SerialWriter::write(" ");
        $crate::serial_writer::SerialWriter::write_usize($val1 as usize);
        $crate::serial_writer::SerialWriter::write(" ");
        $crate::serial_writer::SerialWriter::write_usize($val2 as usize);
        $crate::serial_writer::SerialWriter::write("\n");
    }};
}

#[macro_export]
macro_rules! serial_logb {
    ($s:literal) => {{
        $crate::serial_writer::SerialWriter::write(concat!("BL: ", $s, "\n"));
    }};
    ($s:literal, $val:expr) => {{
        $crate::serial_writer::SerialWriter::write(concat!("BL: ", $s, " "));
        $crate::serial_writer::SerialWriter::write_usize($val as usize);
        $crate::serial_writer::SerialWriter::write("\n");
    }};
    ($s:literal, $val1:expr, $val2:expr) => {{
        $crate::serial_writer::SerialWriter::write(concat!("BL: ", $s, " "));
        $crate::serial_writer::SerialWriter::write_usize($val1 as usize);
        $crate::serial_writer::SerialWriter::write(" ");
        $crate::serial_writer::SerialWriter::write_usize($val2 as usize);
        $crate::serial_writer::SerialWriter::write("\n");
    }};
}

#[macro_export]
macro_rules! serial_logk {
    ($s:literal) => {{
        $crate::serial_writer::SerialWriter::write(concat!("K: ", $s, "\n"));
    }};
    ($s:literal, $val:expr) => {{
        $crate::serial_writer::SerialWriter::write(concat!("K: ", $s, " "));
        $crate::serial_writer::SerialWriter::write_usize($val as usize);
        $crate::serial_writer::SerialWriter::write("\n");
    }};
    ($s:literal, $val1:expr, $val2:expr) => {{
        $crate::serial_writer::SerialWriter::write(concat!("K: ", $s, " "));
        $crate::serial_writer::SerialWriter::write_usize($val1 as usize);
        $crate::serial_writer::SerialWriter::write(" ");
        $crate::serial_writer::SerialWriter::write_usize($val2 as usize);
        $crate::serial_writer::SerialWriter::write("\n");
    }};
}