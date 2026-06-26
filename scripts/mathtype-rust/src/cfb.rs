const SECTOR_SIZE: usize = 512;
const FREE_SECTOR: u32 = 0xffff_ffff;
const END_OF_CHAIN: u32 = 0xffff_fffe;
const FAT_SECTOR: u32 = 0xffff_fffd;
const MINI_SECTOR_SIZE: usize = 64;

#[derive(Debug)]
struct DirectoryEntry {
    start_sector: u32,
    size: u64,
}

/// Read a named regular or mini CFB stream from a MathType OLE file.
pub(crate) fn read_regular_stream(file: &[u8], stream_name: &str) -> Result<Vec<u8>, String> {
    if file.len() < SECTOR_SIZE {
        return Err("compound file is shorter than one header sector".to_string());
    }
    if file.get(..8) != Some(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]) {
        return Err("not a compound file binary object".to_string());
    }
    let sector_shift = read_u16_at(file, 30)? as usize;
    let sector_size = 1usize
        .checked_shl(sector_shift as u32)
        .ok_or_else(|| format!("invalid CFB sector shift: {sector_shift}"))?;
    if sector_size != SECTOR_SIZE {
        return Err(format!("unsupported CFB sector size: {sector_size}"));
    }

    let first_directory_sector = read_u32_at(file, 48)?;
    let mini_stream_cutoff = read_u32_at(file, 56)? as u64;
    let first_mini_fat_sector = read_u32_at(file, 60)?;
    let mini_fat_sector_count = read_u32_at(file, 64)? as usize;
    let fat_sector_count = read_u32_at(file, 44)? as usize;
    let fat_sectors = read_header_difat(file, fat_sector_count)?;
    let fat = read_fat(file, &fat_sectors, sector_size)?;
    let directory = read_sector_chain(file, first_directory_sector, &fat, sector_size)?;
    let stream = find_directory_entry(&directory, stream_name)?;
    let root = find_directory_entry(&directory, "Root Entry")?;

    let mut data = if stream.size < mini_stream_cutoff {
        let mini_fat = read_mini_fat(
            file,
            first_mini_fat_sector,
            mini_fat_sector_count,
            &fat,
            sector_size,
        )?;
        let mini_stream = read_sector_chain(file, root.start_sector, &fat, sector_size)?;
        read_mini_stream_chain(&mini_stream, stream.start_sector, &mini_fat)?
    } else {
        read_sector_chain(file, stream.start_sector, &fat, sector_size)?
    };

    let wanted_len = usize::try_from(stream.size)
        .map_err(|_| format!("stream is too large to fit in memory: {}", stream.size))?;
    if data.len() < wanted_len {
        return Err(format!(
            "stream {stream_name} chain is shorter than declared size: {} < {wanted_len}",
            data.len()
        ));
    }
    data.truncate(wanted_len);
    Ok(data)
}

/// Decode the DIFAT entries stored directly in the CFB header.
fn read_header_difat(file: &[u8], fat_sector_count: usize) -> Result<Vec<u32>, String> {
    if fat_sector_count > 109 {
        return Err("CFB reader only supports header DIFAT entries".to_string());
    }
    let mut sectors = Vec::new();
    for index in 0..fat_sector_count {
        let sector = read_u32_at(file, 76 + index * 4)?;
        if sector != FREE_SECTOR {
            sectors.push(sector);
        }
    }
    Ok(sectors)
}

/// Load the FAT table from the listed FAT sectors.
fn read_fat(file: &[u8], fat_sectors: &[u32], sector_size: usize) -> Result<Vec<u32>, String> {
    let mut fat = Vec::new();
    for &sector in fat_sectors {
        let bytes = sector_bytes(file, sector, sector_size)?;
        for chunk in bytes.chunks_exact(4) {
            fat.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
    }
    Ok(fat)
}

/// Follow a regular FAT chain and concatenate all sector payloads.
fn read_sector_chain(
    file: &[u8],
    start_sector: u32,
    fat: &[u32],
    sector_size: usize,
) -> Result<Vec<u8>, String> {
    if start_sector == END_OF_CHAIN {
        return Ok(Vec::new());
    }
    let mut data = Vec::new();
    let mut sector = start_sector;
    let mut guard = 0usize;
    while sector != END_OF_CHAIN {
        if sector == FREE_SECTOR || sector == FAT_SECTOR {
            return Err(format!("invalid sector in chain: 0x{sector:08x}"));
        }
        data.extend_from_slice(sector_bytes(file, sector, sector_size)?);
        sector = *fat
            .get(sector as usize)
            .ok_or_else(|| format!("sector {sector} is outside the FAT"))?;
        guard += 1;
        if guard > fat.len() {
            return Err("sector chain appears to contain a cycle".to_string());
        }
    }
    Ok(data)
}

/// Locate a named stream entry in the decoded directory bytes.
fn find_directory_entry(directory: &[u8], name: &str) -> Result<DirectoryEntry, String> {
    for entry in directory.chunks_exact(128) {
        let entry_name = directory_entry_name(entry)?;
        if entry_name == name {
            return Ok(DirectoryEntry {
                start_sector: read_u32_at(entry, 116)?,
                size: read_u64_at(entry, 120)?,
            });
        }
    }
    Err(format!("stream not found in compound file: {name}"))
}

/// Load the MiniFAT chain, which MathType uses for small Equation Native streams.
fn read_mini_fat(
    file: &[u8],
    first_sector: u32,
    sector_count: usize,
    fat: &[u32],
    sector_size: usize,
) -> Result<Vec<u32>, String> {
    if sector_count == 0 || first_sector == END_OF_CHAIN {
        return Ok(Vec::new());
    }
    let mut bytes = Vec::new();
    let mut sector = first_sector;
    for _ in 0..sector_count {
        bytes.extend_from_slice(sector_bytes(file, sector, sector_size)?);
        sector = *fat
            .get(sector as usize)
            .ok_or_else(|| format!("MiniFAT sector {sector} is outside the FAT"))?;
        if sector == END_OF_CHAIN {
            break;
        }
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

/// Follow a MiniFAT chain inside the root mini stream.
fn read_mini_stream_chain(
    mini_stream: &[u8],
    start_sector: u32,
    mini_fat: &[u32],
) -> Result<Vec<u8>, String> {
    if start_sector == END_OF_CHAIN {
        return Ok(Vec::new());
    }
    let mut data = Vec::new();
    let mut sector = start_sector;
    let mut guard = 0usize;
    while sector != END_OF_CHAIN {
        if sector == FREE_SECTOR {
            return Err("invalid free mini sector in chain".to_string());
        }
        let start = sector as usize * MINI_SECTOR_SIZE;
        let end = start + MINI_SECTOR_SIZE;
        let bytes = mini_stream
            .get(start..end)
            .ok_or_else(|| format!("mini sector {sector} is outside the mini stream"))?;
        data.extend_from_slice(bytes);
        sector = *mini_fat
            .get(sector as usize)
            .ok_or_else(|| format!("mini sector {sector} is outside the MiniFAT"))?;
        guard += 1;
        if guard > mini_fat.len() {
            return Err("mini sector chain appears to contain a cycle".to_string());
        }
    }
    Ok(data)
}

/// Decode a CFB directory entry name from UTF-16LE.
fn directory_entry_name(entry: &[u8]) -> Result<String, String> {
    let name_len = read_u16_at(entry, 64)? as usize;
    if name_len < 2 || name_len > 64 {
        return Ok(String::new());
    }
    let raw_name = &entry[..name_len - 2];
    let units = raw_name
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    String::from_utf16(&units).map_err(|err| format!("invalid directory UTF-16 name: {err}"))
}

/// Return one CFB sector by sector number.
fn sector_bytes(file: &[u8], sector: u32, sector_size: usize) -> Result<&[u8], String> {
    let start = (sector as usize + 1)
        .checked_mul(sector_size)
        .ok_or_else(|| format!("sector offset overflow for sector {sector}"))?;
    let end = start
        .checked_add(sector_size)
        .ok_or_else(|| format!("sector end overflow for sector {sector}"))?;
    file.get(start..end)
        .ok_or_else(|| format!("sector {sector} is outside the compound file"))
}

/// Read a little-endian u16 from a fixed offset.
fn read_u16_at(data: &[u8], offset: usize) -> Result<u16, String> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| format!("u16 offset {offset} is outside the buffer"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

/// Read a little-endian u32 from a fixed offset.
fn read_u32_at(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 offset {offset} is outside the buffer"))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Read a little-endian u64 from a fixed offset.
fn read_u64_at(data: &[u8], offset: usize) -> Result<u64, String> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or_else(|| format!("u64 offset {offset} is outside the buffer"))?;
    Ok(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}
