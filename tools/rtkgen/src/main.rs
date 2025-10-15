use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use rtkfmt::{constants::RTOSK_MAGIC, RtkHeader, RtkSegment, RTK_EXEC_FLAG};

fn parse_u64_auto(s: &str) -> u64 {
    let t = s.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).expect("parse hex u64")
    } else {
        t.parse::<u64>().expect("parse u64")
    }
}

fn main() {
    // Usage:
    //   rtkgen <input.bin-or-elf> <output.rtosk> [entry_va] [page_size]
    //
    // Behavior:
    //   * ALWAYS packs as a flat RTOSK (single segment) using libs/rtkfmt types.
    //   * Ignores ELF program headers entirely.
    //   * entry_va default = 0x200000 (can override via arg3).
    //   * page_size default = 4096 (can override via arg4).
    //
    // This guarantees header.entry64 != 0 and flags = RTK_EXEC_FLAG.

    let mut args = env::args().skip(1);
    let input_path = PathBuf::from(args.next().expect("input path"));
    let output_path = PathBuf::from(args.next().expect("output path"));
    let entry_va = args
        .next()
        .map(|s| parse_u64_auto(&s))
        .unwrap_or(0x200000);
    let page_size = args
        .next()
        .map(|s| parse_u64_auto(&s) as u32)
        .unwrap_or(4096);

    // Read entire input as a raw payload
    let mut in_file = File::open(&input_path).expect("open input");
    let mut payload = Vec::<u8>::new();
    in_file.read_to_end(&mut payload).expect("read input");
    let payload_len = payload.len() as u64;

    // Build header & a single executable segment
    let seg = RtkSegment {
        file_offset: 0,                 // to be backfilled after we write payload
        memory_addr: entry_va,          // VA where we want it mapped
        memory_size: payload_len,       // zero-fill beyond file_size if needed (here equal)
        file_size: payload_len,
        flags: RTK_EXEC_FLAG,           // executable
    };

    let header_len = (core::mem::size_of::<RtkHeader>() + core::mem::size_of::<RtkSegment>()) as u32;

    let mut header = RtkHeader {
        magic: RTOSK_MAGIC,
        ver_major: 1,
        ver_minor: 0,
        header_len,
        entry64: entry_va,              // <<< critical: non-zero entry VA
        page_size,
        seg_count: 1,
        image_crc32: 0,                 // to be backfilled
        flags: 0,
    };

    // Write header + seg table + payload, then backfill offsets & CRC
    let mut out = File::create(&output_path).expect("create out");

    // header
    let header_bytes = unsafe {
        core::slice::from_raw_parts(
            &header as *const RtkHeader as *const u8,
            core::mem::size_of::<RtkHeader>(),
        )
    };
    out.write_all(header_bytes).expect("write header");

    // seg table (temporary; file_offset will be updated)
    let mut seg_written = seg;
    let seg_bytes = unsafe {
        core::slice::from_raw_parts(
            &seg_written as *const RtkSegment as *const u8,
            core::mem::size_of::<RtkSegment>(),
        )
    };
    out.write_all(seg_bytes).expect("write seg");

    // payload
    let payload_off = out.stream_position().expect("pos");
    out.write_all(&payload).expect("write payload");

    // backfill segment.file_offset
    seg_written.file_offset = payload_off;
    let seg_table_start = core::mem::size_of::<RtkHeader>() as u64;
    out.seek(SeekFrom::Start(seg_table_start)).expect("seek seg");
    let seg_bytes_updated = unsafe {
        core::slice::from_raw_parts(
            &seg_written as *const RtkSegment as *const u8,
            core::mem::size_of::<RtkSegment>(),
        )
    };
    out.write_all(seg_bytes_updated).expect("rewrite seg");

    // compute CRC of image region after header_len
    let crc = compute_image_crc32(&output_path, header_len as u64);
    header.image_crc32 = crc;

    // backfill header with CRC
    out.seek(SeekFrom::Start(0)).expect("seek start");
    let header_bytes_updated = unsafe {
        core::slice::from_raw_parts(
            &header as *const RtkHeader as *const u8,
            core::mem::size_of::<RtkHeader>(),
        )
    };
    out.write_all(header_bytes_updated).expect("rewrite header");
}

fn compute_image_crc32(path: &PathBuf, offset: u64) -> u32 {
    let mut file = File::open(path).expect("open image");
    file.seek(SeekFrom::Start(offset)).expect("seek payload");
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).expect("read");
        if n == 0 { break; }
        for &b in &buf[..n] {
            let mut x = (crc ^ (b as u32)) & 0xFF;
            for _ in 0..8 {
                let m = (x & 1).wrapping_neg();
                x = (x >> 1) ^ (0xEDB88320 & m);
            }
            crc = (crc >> 8) ^ x;
        }
    }
    !crc
}
