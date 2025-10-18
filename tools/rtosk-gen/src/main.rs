use std::{
    env,
    fs::File,
    io::{Read, Write},
    mem,
    path::PathBuf,
};

use goblin::elf::{program_header, section_header, Elf};
use rtos_types::{RTOSK_MAGIC, RtoskHeader, RtoskSegment, RTOSK_EXEC_FLAG};

fn align_up(x: usize, a: usize) -> usize { (x + (a - 1)) & !(a - 1) }

fn parse_u64(s: &str) -> u64 {
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(h, 16).unwrap()
    } else { s.parse::<u64>().unwrap() }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        let mut x = (crc ^ (b as u32)) & 0xFF;
        for _ in 0..8 {
            let m = (x & 1).wrapping_neg();
            x = (x >> 1) ^ (0xEDB88320 & m);
        }
        crc = (crc >> 8) ^ x;
    }
    !crc
}

#[derive(Clone, Copy)]
struct PayloadSpec { src_off: usize, file_sz: usize }

fn main() {
    // Usage: rtosk-gen <input> <output.rtosk> [entry_va] [page_size]
    let mut args = env::args().skip(1);
    let in_path  = PathBuf::from(args.next().expect("usage: rtosk-gen <input> <output.rtosk> [entry_va] [page_size]"));
    let out_path = PathBuf::from(args.next().expect("usage: rtosk-gen <input> <output.rtosk> [entry_va] [page_size]"));
    let entry_override = args.next();
    let page_size: u32 = args.next().map(|s| parse_u64(&s) as u32).unwrap_or(4096);

    // Read whole input
    let mut f = File::open(&in_path).expect("open input");
    let mut blob = Vec::new();
    f.read_to_end(&mut blob).expect("read input");

    let mut segments: Vec<RtoskSegment> = Vec::new();
    let mut payloads: Vec<PayloadSpec>  = Vec::new();
    let mut entry64: u64;

    // Try ELF first
    if let Ok(elf) = Elf::parse(&blob) {
        // Prefer ELF64 little-endian; else treat as flat bin
        if elf.is_64 && elf.little_endian {
            // 1) Try PT_LOAD-based packing
            entry64 = entry_override.as_deref().map(parse_u64).unwrap_or(elf.header.e_entry);

            let mut total_ph_filesz: usize = 0;
            for ph in &elf.program_headers {
                if ph.p_type != program_header::PT_LOAD || ph.p_memsz == 0 { continue; }

                let src_off = ph.p_offset as usize;
                let file_sz = ph.p_filesz as usize;
                // Ensure we don't read past file
                if src_off.checked_add(file_sz).unwrap_or(usize::MAX) > blob.len() { continue; }

                let flags = if (ph.p_flags & program_header::PF_X) != 0 { RTOSK_EXEC_FLAG } else { 0 };

                segments.push(RtoskSegment {
                    file_offset: 0,
                    file_size: ph.p_filesz,
                    memory_addr: ph.p_vaddr,
                    memory_size: ph.p_memsz,
                    flags,
                });
                payloads.push(PayloadSpec { src_off, file_sz });
                total_ph_filesz = total_ph_filesz.saturating_add(file_sz);
            }

            // 2) If PHDRs gave us nothing useful (your tiny test case), fall back to SHDR packing.
            let ph_pack_too_small = total_ph_filesz < 64; // heuristic: basically empty
            if segments.is_empty() || ph_pack_too_small {
                segments.clear();
                payloads.clear();

                // Collect allocatable PROGBITS sections (.text/.rodata/.data etc.)
                // Keep original VAs (sh_addr) so they map where you linked them (0x200000…).
                let mut total_sh_filesz = 0usize;
                for sh in &elf.section_headers {
                    let sh_flags = sh.sh_flags;
                    let sh_type  = sh.sh_type;

                    let is_alloc   = (sh_flags & section_header::SHF_ALLOC as u64) != 0;
                    let is_progbit = sh_type == section_header::SHT_PROGBITS;

                    if !is_alloc || !is_progbit || sh.sh_size == 0 { continue; }

                    let src_off = sh.sh_offset as usize;
                    let file_sz = sh.sh_size as usize;
                    if src_off.checked_add(file_sz).unwrap_or(usize::MAX) > blob.len() { continue; }

                    // Mark exec if this looks like .text (has SHF_EXECINSTR)
                    let exec = (sh_flags & section_header::SHF_EXECINSTR as u64) != 0;
                    let flags = if exec { RTOSK_EXEC_FLAG } else { 0 };

                    segments.push(RtoskSegment {
                        file_offset: 0,
                        file_size: file_sz as u64,
                        memory_addr: sh.sh_addr,
                        memory_size: sh.sh_size,
                        flags,
                    });
                    payloads.push(PayloadSpec { src_off, file_sz });
                    total_sh_filesz = total_sh_filesz.saturating_add(file_sz);
                }

                // Entry selection: override > ELF e_entry > first alloc section > 0x200000
                if entry64 == 0 {
                    if let Some(first_alloc) = elf.section_headers.iter()
                        .find(|sh| (sh.sh_flags & section_header::SHF_ALLOC as u64) != 0 && sh.sh_addr != 0)
                    {
                        entry64 = first_alloc.sh_addr;
                    } else {
                        entry64 = 0x200000;
                    }
                }

                // Last resort: if even sections yielded nothing (stripped test?), pack flat at entry
                if segments.is_empty() {
                    let sz = blob.len() as u64;
                    let e  = if entry64 != 0 { entry64 } else { 0x200000 };
                    segments.push(RtoskSegment {
                        file_offset: 0,
                        file_size: sz,
                        memory_addr: e,
                        memory_size: sz,
                        flags: RTOSK_EXEC_FLAG,
                    });
                    payloads.push(PayloadSpec { src_off: 0, file_sz: sz as usize });
                }
            }

            // If e_entry is still zero, default to first executable segment or first segment VA.
            if entry64 == 0 {
                entry64 = segments.iter()
                    .find(|s| s.flags & RTOSK_EXEC_FLAG != 0)
                    .map(|s| s.memory_addr)
                    .or_else(|| segments.first().map(|s| s.memory_addr))
                    .unwrap_or(0x200000);
            }
        } else {
            // Non-ELF64 → flat bin
            let e = entry_override.as_deref().map(parse_u64).unwrap_or(0x200000);
            entry64 = e;
            let sz = blob.len() as u64;
            segments.push(RtoskSegment {
                file_offset: 0, file_size: sz, memory_addr: e, memory_size: sz, flags: RTOSK_EXEC_FLAG,
            });
            payloads.push(PayloadSpec { src_off: 0, file_sz: sz as usize });
        }
    } else {
        // Not ELF → flat bin
        let e = entry_override.as_deref().map(parse_u64).unwrap_or(0x200000);
        entry64 = e;
        let sz = blob.len() as u64;
        segments.push(RtoskSegment {
            file_offset: 0, file_size: sz, memory_addr: e, memory_size: sz, flags: RTOSK_EXEC_FLAG,
        });
        payloads.push(PayloadSpec { src_off: 0, file_sz: sz as usize });
    }

    // ---- Layout RTOSK image ----
    let header_len = mem::size_of::<RtoskHeader>() + segments.len() * mem::size_of::<RtoskSegment>();
    let mut cur = header_len;
    for (seg, p) in segments.iter_mut().zip(payloads.iter()) {
        cur = align_up(cur, 16);
        seg.file_offset = cur as u64;
        cur += p.file_sz;
    }

    let mut out_buf = vec![0u8; cur];

    // Header + segment table
    let mut hdr = RtoskHeader {
        magic: RTOSK_MAGIC,
        ver_major: 1,
        ver_minor: 0,
        header_len: header_len as u32,
        entry64,
        page_size,
        seg_count: segments.len() as u32,
        image_crc32: 0,
        flags: 0,
    };

    let mut cursor = 0usize;
    unsafe {
        let hptr = &hdr as *const _ as *const u8;
        std::ptr::copy_nonoverlapping(hptr, out_buf.as_mut_ptr().add(cursor), mem::size_of::<RtoskHeader>());
        cursor += mem::size_of::<RtoskHeader>();
        for seg in &segments {
            let sptr = seg as *const _ as *const u8;
            std::ptr::copy_nonoverlapping(sptr, out_buf.as_mut_ptr().add(cursor), mem::size_of::<RtoskSegment>());
            cursor += mem::size_of::<RtoskSegment>();
        }
    }

    // Copy payload bytes
    for (i, p) in payloads.iter().enumerate() {
        if p.file_sz == 0 { continue; }
        let dst = segments[i].file_offset as usize;
        out_buf[dst..dst + p.file_sz].copy_from_slice(&blob[p.src_off..p.src_off + p.file_sz]);
    }

    // Finalize CRC
    hdr.image_crc32 = crc32(&out_buf);
    unsafe {
        let hptr = &hdr as *const _ as *const u8;
        std::ptr::copy_nonoverlapping(hptr, out_buf.as_mut_ptr(), mem::size_of::<RtoskHeader>());
    }

    let mut out = File::create(&out_path).expect("create output");
    out.write_all(&out_buf).expect("write output");
    let _ = out.flush();
}
