pub(crate) const END_OF_CHAIN: u32 = 0xFFFF_FFFE;
pub(crate) const FREE_SECTOR: u32 = 0xFFFF_FFFF;
pub(crate) const FAT_SECTOR: u32 = 0xFFFF_FFFD;
pub(crate) const SECTOR_SIZE: usize = 512;

const OLE_STREAM: &[u8] = &[
    0x01, 0x00, 0x00, 0x02, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

const COMP_OBJ_STREAM: &[u8] = &[
    0x01, 0x00, 0xfe, 0xff, 0x03, 0x0a, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x03, 0xce, 0x02, 0x00,
    0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46, 0x16, 0x00, 0x00, 0x00,
    b'M', b'a', b't', b'h', b'T', b'y', b'p', b'e', b' ', b'7', b'.', b'0', b' ', b'E', b'q', b'u',
    b'a', b't', b'i', b'o', b'n', 0x00, 0x0c, 0x00, 0x00, 0x00, b'M', b'a', b't', b'h', b'T', b'y',
    b'p', b'e', b' ', b'E', b'F', 0x00, 0x0f, 0x00, 0x00, 0x00, b'E', b'q', b'u', b'a', b't', b'i',
    b'o', b'n', b'.', b'D', b'S', b'M', b'T', b'4', 0x00, 0xf4, 0x39, 0xb2, 0x71, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const EQUATION_CLSID: [u8; 16] = [
    0x03, 0xce, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
];

#[derive(Debug)]
struct Stream<'a> {
    name: &'a str,
    data: &'a [u8],
    start_sector: u32,
}

/// Write a minimal regular-stream CFB file containing the MathType streams.
pub(crate) fn write_compound_file(equation_native: &[u8]) -> Result<Vec<u8>, String> {
    let mut streams = vec![
        Stream {
            name: "\u{0001}Ole",
            data: OLE_STREAM,
            start_sector: 0,
        },
        Stream {
            name: "Equation Native",
            data: equation_native,
            start_sector: 0,
        },
        Stream {
            name: "\u{0001}CompObj",
            data: COMP_OBJ_STREAM,
            start_sector: 0,
        },
    ];

    let mut next_sector = 0u32;
    for stream in &mut streams {
        stream.start_sector = next_sector;
        next_sector += sectors_needed(stream.data.len()) as u32;
    }
    let dir_sector = next_sector;
    next_sector += 1;
    let fat_sector = next_sector;
    let total_sectors = next_sector + 1;
    if total_sectors > 128 {
        return Err("minimal CFB writer currently supports at most 128 sectors".to_string());
    }

    let mut file = Vec::with_capacity((total_sectors as usize + 1) * SECTOR_SIZE);
    file.extend_from_slice(&cfb_header(dir_sector, fat_sector));
    for stream in &streams {
        append_stream_sectors(&mut file, stream.data);
    }
    append_directory_sector(&mut file, &streams);
    append_fat_sector(&mut file, &streams, dir_sector, fat_sector, total_sectors);
    Ok(file)
}

/// Return how many 512-byte sectors are needed for one regular stream.
fn sectors_needed(size: usize) -> usize {
    size.max(1).div_ceil(SECTOR_SIZE)
}

/// Build the CFB header with MiniFAT disabled by a zero mini-stream cutoff.
fn cfb_header(dir_sector: u32, fat_sector: u32) -> [u8; SECTOR_SIZE] {
    let mut header = [0xffu8; SECTOR_SIZE];
    header[..8].copy_from_slice(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
    put_u16(&mut header, 24, 0x003e);
    put_u16(&mut header, 26, 0x0003);
    put_u16(&mut header, 28, 0xfffe);
    put_u16(&mut header, 30, 9);
    put_u16(&mut header, 32, 6);
    put_u32(&mut header, 44, 1);
    put_u32(&mut header, 48, dir_sector);
    put_u32(&mut header, 56, 0);
    put_u32(&mut header, 60, END_OF_CHAIN);
    put_u32(&mut header, 68, END_OF_CHAIN);
    put_u32(&mut header, 76, fat_sector);
    header
}

/// Append one stream padded to full CFB sectors.
fn append_stream_sectors(file: &mut Vec<u8>, data: &[u8]) {
    let padded_len = sectors_needed(data.len()) * SECTOR_SIZE;
    file.extend_from_slice(data);
    file.resize(file.len() + padded_len - data.len(), 0);
}

/// Append the directory tree used by MathType's own OLE output.
fn append_directory_sector(file: &mut Vec<u8>, streams: &[Stream<'_>]) {
    let mut sector = [0u8; SECTOR_SIZE];
    write_directory_entry(
        &mut sector[0..128],
        "Root Entry",
        5,
        0,
        FREE_SECTOR,
        FREE_SECTOR,
        3,
        END_OF_CHAIN,
        0,
        Some(EQUATION_CLSID),
    );
    write_directory_entry(
        &mut sector[128..256],
        streams[0].name,
        2,
        1,
        FREE_SECTOR,
        FREE_SECTOR,
        FREE_SECTOR,
        streams[0].start_sector,
        streams[0].data.len() as u64,
        None,
    );
    write_directory_entry(
        &mut sector[256..384],
        streams[1].name,
        2,
        1,
        FREE_SECTOR,
        FREE_SECTOR,
        FREE_SECTOR,
        streams[1].start_sector,
        streams[1].data.len() as u64,
        None,
    );
    write_directory_entry(
        &mut sector[384..512],
        streams[2].name,
        2,
        1,
        1,
        2,
        FREE_SECTOR,
        streams[2].start_sector,
        streams[2].data.len() as u64,
        None,
    );
    file.extend_from_slice(&sector);
}

/// Write one 128-byte CFB directory entry.
#[allow(clippy::too_many_arguments)]
fn write_directory_entry(
    entry: &mut [u8],
    name: &str,
    object_type: u8,
    color: u8,
    left: u32,
    right: u32,
    child: u32,
    start_sector: u32,
    size: u64,
    clsid: Option<[u8; 16]>,
) {
    let mut utf16: Vec<u16> = name.encode_utf16().collect();
    utf16.push(0);
    for (idx, unit) in utf16.iter().enumerate() {
        let offset = idx * 2;
        entry[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
    }
    put_u16(entry, 64, (utf16.len() * 2) as u16);
    entry[66] = object_type;
    entry[67] = color;
    put_u32(entry, 68, left);
    put_u32(entry, 72, right);
    put_u32(entry, 76, child);
    if let Some(value) = clsid {
        entry[80..96].copy_from_slice(&value);
    }
    put_u32(entry, 116, start_sector);
    put_u64(entry, 120, size);
}

/// Append the FAT chains for all streams plus directory and FAT sectors.
fn append_fat_sector(
    file: &mut Vec<u8>,
    streams: &[Stream<'_>],
    dir_sector: u32,
    fat_sector: u32,
    total_sectors: u32,
) {
    let mut fat = [0xffu8; SECTOR_SIZE];
    for stream in streams {
        let count = sectors_needed(stream.data.len()) as u32;
        for offset in 0..count {
            let sector = stream.start_sector + offset;
            let next = if offset + 1 == count {
                END_OF_CHAIN
            } else {
                sector + 1
            };
            put_u32(&mut fat, (sector * 4) as usize, next);
        }
    }
    put_u32(&mut fat, (dir_sector * 4) as usize, END_OF_CHAIN);
    put_u32(&mut fat, (fat_sector * 4) as usize, FAT_SECTOR);
    for sector in total_sectors..128 {
        put_u32(&mut fat, (sector * 4) as usize, FREE_SECTOR);
    }
    file.extend_from_slice(&fat);
}

/// Store a little-endian u16 at an absolute byte offset.
fn put_u16(buf: &mut [u8], offset: usize, value: u16) {
    buf[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

/// Store a little-endian u32 at an absolute byte offset.
fn put_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Store a little-endian u64 at an absolute byte offset.
fn put_u64(buf: &mut [u8], offset: usize, value: u64) {
    buf[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
