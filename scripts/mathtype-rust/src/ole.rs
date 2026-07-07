pub(crate) const END_OF_CHAIN: u32 = 0xFFFF_FFFE;
pub(crate) const FREE_SECTOR: u32 = 0xFFFF_FFFF;
pub(crate) const FAT_SECTOR: u32 = 0xFFFF_FFFD;
pub(crate) const SECTOR_SIZE: usize = 512;
const MINI_STREAM_CUTOFF: u32 = 4096;
const MINI_SECTOR_SIZE: usize = 64;
const FAT_SECTOR_ID: u32 = 0;
const FIRST_DIRECTORY_SECTOR: u32 = 1;

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

/// Write a CFB file containing MathType streams in Word-compatible MiniFAT layout.
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

    let mut mini_stream = Vec::new();
    let mut mini_fat = Vec::new();
    for stream in &mut streams {
        if use_mini_stream(stream.data) {
            stream.start_sector = append_mini_stream(&mut mini_stream, &mut mini_fat, stream.data)?;
        }
    }
    let root_mini_stream = padded_sector_bytes(&mini_stream);
    let mini_fat_sector_count = sectors_needed_bytes(mini_fat.len() * 4);
    let first_mini_fat_sector = if mini_fat_sector_count == 0 {
        END_OF_CHAIN
    } else {
        FIRST_DIRECTORY_SECTOR + 1
    };
    let root_mini_stream_sector = if root_mini_stream.is_empty() {
        END_OF_CHAIN
    } else {
        first_mini_fat_sector + mini_fat_sector_count as u32
    };
    let root_mini_stream_sector_count = sectors_needed_bytes(root_mini_stream.len());
    let second_directory_sector = root_mini_stream_sector + root_mini_stream_sector_count as u32;
    let mut next_regular_sector = second_directory_sector + 1;

    for stream in &mut streams {
        if !use_mini_stream(stream.data) {
            stream.start_sector = next_regular_sector;
            next_regular_sector += sectors_needed_bytes(stream.data.len()) as u32;
        }
    }

    let total_sectors = next_regular_sector;
    if total_sectors > 128 {
        return Err("minimal CFB writer currently supports at most 128 sectors".to_string());
    }

    let mut sectors = vec![[0u8; SECTOR_SIZE]; total_sectors as usize];
    write_directory_sectors(
        &mut sectors,
        &streams,
        root_mini_stream_sector,
        root_mini_stream.len() as u64,
        second_directory_sector,
    );
    write_minifat_sectors(
        &mut sectors,
        first_mini_fat_sector,
        mini_fat_sector_count,
        &mini_fat,
    );
    write_regular_sector_chain(&mut sectors, root_mini_stream_sector, &root_mini_stream);
    for stream in &streams {
        if !use_mini_stream(stream.data) {
            write_regular_sector_chain(&mut sectors, stream.start_sector, stream.data);
        }
    }
    write_fat_sector(
        &mut sectors,
        &streams,
        first_mini_fat_sector,
        mini_fat_sector_count,
        root_mini_stream_sector,
        root_mini_stream_sector_count,
        second_directory_sector,
        total_sectors,
    );

    let mut file = Vec::with_capacity((total_sectors as usize + 1) * SECTOR_SIZE);
    file.extend_from_slice(&cfb_header(
        first_mini_fat_sector,
        mini_fat_sector_count as u32,
    ));
    for sector in sectors {
        file.extend_from_slice(&sector);
    }
    Ok(file)
}

/// Return true when a MathType stream should live in the root mini stream.
fn use_mini_stream(data: &[u8]) -> bool {
    data.len() < MINI_STREAM_CUTOFF as usize
}

/// Return how many 512-byte sectors are needed for byte data.
fn sectors_needed_bytes(size: usize) -> usize {
    if size == 0 {
        0
    } else {
        size.div_ceil(SECTOR_SIZE)
    }
}

/// Return how many 64-byte mini sectors are needed for byte data.
fn mini_sectors_needed(size: usize) -> usize {
    if size == 0 {
        0
    } else {
        size.div_ceil(MINI_SECTOR_SIZE)
    }
}

/// Append a small stream to the root mini stream and return its mini sector.
fn append_mini_stream(
    mini_stream: &mut Vec<u8>,
    mini_fat: &mut Vec<u32>,
    data: &[u8],
) -> Result<u32, String> {
    let start_sector = u32::try_from(mini_fat.len())
        .map_err(|_| "mini stream contains too many sectors".to_string())?;
    let sector_count = mini_sectors_needed(data.len());
    if sector_count == 0 {
        return Ok(END_OF_CHAIN);
    }

    let first_new_sector = mini_fat.len();
    for offset in 0..sector_count {
        let next = if offset + 1 == sector_count {
            END_OF_CHAIN
        } else {
            u32::try_from(first_new_sector + offset + 1)
                .map_err(|_| "mini stream contains too many sectors".to_string())?
        };
        mini_fat.push(next);
    }

    mini_stream.extend_from_slice(data);
    mini_stream.resize(
        mini_stream.len() + sector_count * MINI_SECTOR_SIZE - data.len(),
        0,
    );
    Ok(start_sector)
}

/// Pad data to whole regular sectors, as required for root mini stream storage.
fn padded_sector_bytes(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut padded = data.to_vec();
    padded.resize(sectors_needed_bytes(padded.len()) * SECTOR_SIZE, 0);
    padded
}

/// Build the CFB header with the standard 4096-byte mini-stream cutoff.
fn cfb_header(first_mini_fat_sector: u32, mini_fat_sector_count: u32) -> [u8; SECTOR_SIZE] {
    let mut header = [0u8; SECTOR_SIZE];
    header[..8].copy_from_slice(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
    put_u16(&mut header, 24, 0x003e);
    put_u16(&mut header, 26, 0x0003);
    put_u16(&mut header, 28, 0xfffe);
    put_u16(&mut header, 30, 9);
    put_u16(&mut header, 32, 6);
    put_u32(&mut header, 44, 1);
    put_u32(&mut header, 48, FIRST_DIRECTORY_SECTOR);
    put_u32(&mut header, 56, MINI_STREAM_CUTOFF);
    put_u32(&mut header, 60, first_mini_fat_sector);
    put_u32(&mut header, 64, mini_fat_sector_count);
    put_u32(&mut header, 68, END_OF_CHAIN);
    for index in 0..109 {
        put_u32(&mut header, 76 + index * 4, FREE_SECTOR);
    }
    put_u32(&mut header, 76, FAT_SECTOR_ID);
    header
}

/// Copy data into a regular FAT sector chain.
fn write_regular_sector_chain(sectors: &mut [[u8; SECTOR_SIZE]], start_sector: u32, data: &[u8]) {
    if start_sector == END_OF_CHAIN || data.is_empty() {
        return;
    }
    for (offset, chunk) in data.chunks(SECTOR_SIZE).enumerate() {
        let sector = start_sector as usize + offset;
        sectors[sector][..chunk.len()].copy_from_slice(chunk);
    }
}

/// Write the two-sector directory chain used by MathType's OLE output.
fn write_directory_sectors(
    sectors: &mut [[u8; SECTOR_SIZE]],
    streams: &[Stream<'_>],
    root_mini_stream_sector: u32,
    root_mini_stream_size: u64,
    second_directory_sector: u32,
) {
    write_directory_entry(
        &mut sectors[FIRST_DIRECTORY_SECTOR as usize][0..128],
        "Root Entry",
        5,
        0,
        FREE_SECTOR,
        FREE_SECTOR,
        3,
        root_mini_stream_sector,
        root_mini_stream_size,
        Some(EQUATION_CLSID),
    );
    write_directory_entry(
        &mut sectors[FIRST_DIRECTORY_SECTOR as usize][128..256],
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
        &mut sectors[FIRST_DIRECTORY_SECTOR as usize][256..384],
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
        &mut sectors[FIRST_DIRECTORY_SECTOR as usize][384..512],
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
    sectors[second_directory_sector as usize] = [0u8; SECTOR_SIZE];
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

/// Write MiniFAT sectors for streams stored inside the root mini stream.
fn write_minifat_sectors(
    sectors: &mut [[u8; SECTOR_SIZE]],
    first_mini_fat_sector: u32,
    mini_fat_sector_count: usize,
    mini_fat: &[u32],
) {
    if first_mini_fat_sector == END_OF_CHAIN {
        return;
    }
    for sector_offset in 0..mini_fat_sector_count {
        let sector = first_mini_fat_sector as usize + sector_offset;
        for entry_index in 0..(SECTOR_SIZE / 4) {
            let source_index = sector_offset * (SECTOR_SIZE / 4) + entry_index;
            let value = mini_fat.get(source_index).copied().unwrap_or(FREE_SECTOR);
            put_u32(&mut sectors[sector], entry_index * 4, value);
        }
    }
}

/// Write the FAT chains for directory, MiniFAT, root mini stream, and large streams.
fn write_fat_sector(
    sectors: &mut [[u8; SECTOR_SIZE]],
    streams: &[Stream<'_>],
    first_mini_fat_sector: u32,
    mini_fat_sector_count: usize,
    root_mini_stream_sector: u32,
    root_mini_stream_sector_count: usize,
    second_directory_sector: u32,
    total_sectors: u32,
) {
    let fat = &mut sectors[FAT_SECTOR_ID as usize];
    for sector in 0..128 {
        put_u32(fat, sector * 4, FREE_SECTOR);
    }
    put_u32(fat, (FAT_SECTOR_ID * 4) as usize, FAT_SECTOR);
    put_u32(
        fat,
        (FIRST_DIRECTORY_SECTOR * 4) as usize,
        second_directory_sector,
    );
    put_u32(fat, (second_directory_sector * 4) as usize, END_OF_CHAIN);

    write_regular_fat_chain(fat, first_mini_fat_sector, mini_fat_sector_count);
    write_regular_fat_chain(fat, root_mini_stream_sector, root_mini_stream_sector_count);
    for stream in streams {
        if !use_mini_stream(stream.data) {
            write_regular_fat_chain(
                fat,
                stream.start_sector,
                sectors_needed_bytes(stream.data.len()),
            );
        }
    }
    for sector in total_sectors..128 {
        put_u32(fat, (sector * 4) as usize, FREE_SECTOR);
    }
}

/// Mark one regular-sector chain in the FAT.
fn write_regular_fat_chain(fat: &mut [u8], start_sector: u32, sector_count: usize) {
    if start_sector == END_OF_CHAIN || sector_count == 0 {
        return;
    }
    for offset in 0..sector_count {
        let sector = start_sector + offset as u32;
        let next = if offset + 1 == sector_count {
            END_OF_CHAIN
        } else {
            sector + 1
        };
        put_u32(fat, (sector * 4) as usize, next);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfb::read_regular_stream;

    /// Read a little-endian u32 from test bytes.
    fn read_u32(buf: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ])
    }

    /// Read a little-endian u64 from test bytes.
    fn read_u64(buf: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
            buf[offset + 4],
            buf[offset + 5],
            buf[offset + 6],
            buf[offset + 7],
        ])
    }

    /// Verify the writer uses the same standard MiniFAT skeleton as set-data output.
    #[test]
    fn writes_set_data_like_minifat_container() {
        let native = vec![0x42; 300];
        let ole = write_compound_file(&native).expect("compound file writes");

        assert_eq!(ole.len(), 3072);
        assert_eq!(read_u32(&ole, 40), 0);
        assert_eq!(read_u32(&ole, 44), 1);
        assert_eq!(read_u32(&ole, 48), 1);
        assert_eq!(read_u32(&ole, 52), 0);
        assert_eq!(read_u32(&ole, 56), MINI_STREAM_CUTOFF);
        assert_eq!(read_u32(&ole, 60), 2);
        assert_eq!(read_u32(&ole, 64), 1);
        assert_eq!(read_u32(&ole, 68), END_OF_CHAIN);
        assert_eq!(read_u32(&ole, 72), 0);
        assert_eq!(read_u32(&ole, 76), FAT_SECTOR_ID);

        let fat_offset = SECTOR_SIZE;
        assert_eq!(read_u32(&ole, fat_offset), FAT_SECTOR);
        assert_eq!(read_u32(&ole, fat_offset + 4), 4);
        assert_eq!(read_u32(&ole, fat_offset + 8), END_OF_CHAIN);
        assert_eq!(read_u32(&ole, fat_offset + 12), END_OF_CHAIN);
        assert_eq!(read_u32(&ole, fat_offset + 16), END_OF_CHAIN);

        let dir_offset = (FIRST_DIRECTORY_SECTOR as usize + 1) * SECTOR_SIZE;
        assert_eq!(read_u32(&ole, dir_offset + 116), 3);
        assert_eq!(read_u64(&ole, dir_offset + 120), 512);
        assert_eq!(read_u32(&ole, dir_offset + 128 + 116), 0);
        assert_eq!(read_u32(&ole, dir_offset + 256 + 116), 1);
        assert_eq!(read_u32(&ole, dir_offset + 384 + 116), 6);

        let roundtrip = read_regular_stream(&ole, "Equation Native").expect("native stream reads");
        assert_eq!(roundtrip, native);
    }

    /// Keep large formulas readable by storing oversized Equation Native data in regular FAT.
    #[test]
    fn stores_large_equation_native_as_regular_stream() {
        let native = vec![0x7f; MINI_STREAM_CUTOFF as usize + 10];
        let ole = write_compound_file(&native).expect("compound file writes");

        let dir_offset = (FIRST_DIRECTORY_SECTOR as usize + 1) * SECTOR_SIZE;
        let equation_start = read_u32(&ole, dir_offset + 256 + 116);
        assert!(equation_start >= 5);

        let roundtrip = read_regular_stream(&ole, "Equation Native").expect("native stream reads");
        assert_eq!(roundtrip, native);
    }
}
