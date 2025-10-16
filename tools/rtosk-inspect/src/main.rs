// Usage: cargo run --bin rtosk-inspect -- /path/to/KERNEL.RTOSK
// If you don't have a cargo package for this, you can also:
// rustc rtos_inspect.rs -O -o rtos_inspect && ./rtos_inspect KERNEL.RTOSK

use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::mem;

use rtoskfmt::{constants::RTOSK_MAGIC, RtoskHeader, RtoskSegment};

#[derive(Debug)]
struct InspectError(String);

impl fmt::Display for InspectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
impl Error for InspectError {}

macro_rules! bail {
    ($($t:tt)*) => { return Err(Box::<dyn Error>::from(InspectError(format!($($t)*)))) };
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn read_struct<T: Copy>(buffer: &[u8], offset: usize) -> Option<T> {
    let size = mem::size_of::<T>();
    if offset + size > buffer.len() { return None; }
    // Safety: T is repr(C) POD (u16/u32/u64 fields). Bounds checked above.
    let mut tmp = mem::MaybeUninit::<T>::uninit();
    unsafe {
        std::ptr::copy_nonoverlapping(
            buffer.as_ptr().add(offset),
            tmp.as_mut_ptr() as *mut u8,
            size,
        );
        Some(tmp.assume_init())
    }
}

fn hex_dump_prefix(bytes: &[u8]) {
    for (i, b) in bytes.iter().enumerate() {
        if i % 16 == 0 {
            if i != 0 { println!(); }
            print!("{:016x}: ", i);
        }
        print!("{:02x} ", b);
    }
    println!();
}

fn dump_bytes_at(data: &[u8], offset: usize, length: usize) {
    let end = offset.saturating_add(length).min(data.len());
    println!("--- dump file[0x{:x}..0x{:x}] ---", offset, end);
    hex_dump_prefix(&data[offset..end]);
    println!();
}

fn find_magic(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if haystack.len() < needle.len() { return None; }
    for i in 0..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle { return Some(i); }
    }
    None
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let path = args.next().ok_or_else(|| InspectError("usage: rtosk-inspect <KERNEL.RTOSK>".into()))?;

    let mut file = File::open(&path)?;
    let mut blob = Vec::new();
    file.read_to_end(&mut blob)?;

    if blob.len() < mem::size_of::<RtoskHeader>() {
        bail!("file too small for header");
    }

    let magic_offset = find_magic(&blob, &RTOSK_MAGIC);
    if magic_offset.is_none() {
        println!("RTOSK magic '{}' not found in file", String::from_utf8_lossy(&RTOSK_MAGIC));
    }
    let magic_off = magic_offset.unwrap_or(0);
    println!("RTOSK magic offset in file: 0x{:x}", magic_off);

    let header: RtoskHeader = match read_struct(&blob, magic_off) {
        Some(h) => h,
        None => bail!("failed to read header at offset 0x{:x}", magic_off),
    };

    let magic_str = String::from_utf8_lossy(&header.magic);
    println!("Header:");
    println!("  magic      = {:?} ({})", &header.magic, magic_str);
    println!("  ver        = {}.{}", header.ver_major, header.ver_minor);
    println!("  header_len = 0x{:x} ({})", header.header_len, header.header_len);
    println!("  entry64    = 0x{:016x}", header.entry64);
    println!("  page_size  = 0x{:x}", header.page_size);
    println!("  seg_count  = {}", header.seg_count);
    println!("  image_crc32= 0x{:08x}", header.image_crc32);
    println!("  flags      = 0x{:08x}", header.flags);
    println!();

    let header_len = header.header_len as usize;
    let seg_table_bytes = (header.seg_count as usize).saturating_mul(mem::size_of::<RtoskSegment>());
    println!("Expect segment table bytes = 0x{:x}", seg_table_bytes);
    if magic_off + header_len + seg_table_bytes > blob.len() {
        println!("WARNING: file too small for declared header_len+segments (moff+hdr_bytes+seg_bytes > file_len)");
    }

    let mut segments: Vec<RtoskSegment> = Vec::new();
    let mut seg_table_off = magic_off + mem::size_of::<RtoskHeader>();
    for i in 0..header.seg_count as usize {
        match read_struct::<RtoskSegment>(&blob, seg_table_off) {
            Some(s) => segments.push(s),
            None => {
                println!("Failed to read segment[{}] at 0x{:x}", i, seg_table_off);
                break;
            }
        }
        seg_table_off += mem::size_of::<RtoskSegment>();
    }

    for (i, seg) in segments.iter().enumerate() {
        println!("Segment[{}]:", i);
        println!("  file_offset  = 0x{:x}", seg.file_offset);
        println!("  file_size    = 0x{:x}", seg.file_size);
        println!("  memory_addr  = 0x{:x}", seg.memory_addr);
        println!("  memory_size  = 0x{:x}", seg.memory_size);
        println!("  flags        = 0x{:x}", seg.flags);
        if (seg.file_offset as usize) < blob.len() {
            let fo = seg.file_offset as usize;
            let want = 64.min(blob.len().saturating_sub(fo));
            println!("  -> file payload at 0x{:x} (first 0x{:x} bytes):", fo, want);
            dump_bytes_at(&blob, fo, want);
        } else {
            println!("  -> file_offset out of range (file len {})", blob.len());
        }
        println!();
    }

    if header.entry64 == 0 {
        println!("header.entry64 is zero — packer didn't set entry VA.");
    } else {
        let entry = header.entry64 as u64;
        println!("Checking header.entry64 = 0x{:x}", entry);
        let mut found = false;
        for (i, seg) in segments.iter().enumerate() {
            let mem_beg = seg.memory_addr;
            let mem_end = seg.memory_addr.saturating_add(seg.memory_size);
            if entry >= mem_beg && entry < mem_end {
                found = true;
                let delta = entry - mem_beg;
                let mapped_file_offset = seg.file_offset.saturating_add(delta);
                println!("  -> entry is inside segment[{}]", i);
                println!("     seg.mem=[0x{:x}..0x{:x}) seg.file_offset=0x{:x}", mem_beg, mem_end, seg.file_offset);
                println!("     entry delta into segment = 0x{:x}", delta);
                println!("     corresponding file offset = 0x{:x}", mapped_file_offset);
                if (mapped_file_offset as usize) < blob.len() {
                    let off = mapped_file_offset as usize;
                    let want = 128.min(blob.len().saturating_sub(off));
                    println!("     bytes at file offset ->");
                    dump_bytes_at(&blob, off, want);
                } else {
                    println!("     mapped file offset out of file range (file len {})", blob.len());
                }
            }
        }
        if !found {
            println!("  -> entry is NOT inside any declared segment memory range. That suggests header.entry64 may be an RVA/offset or incorrect.");
            let want = 128.min(blob.len());
            println!();
            println!("File head (first {} bytes):", want);
            dump_bytes_at(&blob, 0, want);
        }
    }

    let total_head_len = (header.header_len as usize).min(blob.len().saturating_sub(magic_off));
    println!(
        "Raw header region (moff..moff+header_len = 0x{:x}..0x{:x}) dump:",
        magic_off,
        magic_off + total_head_len
    );
    if total_head_len > 0 {
        dump_bytes_at(&blob, magic_off, total_head_len.min(256));
    }

    Ok(())
}
