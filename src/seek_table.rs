use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::{bail, Context};

// zstd seekable format constants
const SKIPPABLE_MAGIC: u32 = 0x184D2A5E;
const SEEKABLE_MAGIC: u32 = 0x8F92EAB1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeekEntry {
    pub compressed_size: u32,
    pub decompressed_size: u32,
}

/// Write the seek table as a zstd skippable frame at the current position.
pub fn write_seek_table<W: Write>(w: &mut W, entries: &[SeekEntry]) -> anyhow::Result<()> {
    let n = entries.len() as u32;
    // frame_size = per-frame entries (8B each) + footer (9B)
    let frame_size: u32 = n * 8 + 9;

    write_u32_le(w, SKIPPABLE_MAGIC)?;
    write_u32_le(w, frame_size)?;

    for e in entries {
        write_u32_le(w, e.compressed_size)?;
        write_u32_le(w, e.decompressed_size)?;
    }

    // footer
    write_u32_le(w, n)?;
    w.write_all(&[0x00])?; // descriptor: no checksum
    write_u32_le(w, SEEKABLE_MAGIC)?;

    Ok(())
}

/// Read the seek table from the end of a seekable stream.
pub fn read_seek_table<R: Read + Seek>(r: &mut R) -> anyhow::Result<Vec<SeekEntry>> {
    // Footer is the last 9 bytes of the skippable frame payload.
    // Skippable frame: magic(4) + frame_size(4) + payload(frame_size)
    // Last 9 bytes of payload = footer: num_frames(4) + descriptor(1) + seekable_magic(4)
    // Total from end: 9 bytes into the payload, plus the 8-byte skippable header prefix = 17 bytes from end.
    // But we need to find the frame_size first.

    // Read from end: last 4 bytes = SEEKABLE_MAGIC, before that 1 byte descriptor, before that 4 bytes = num_frames
    r.seek(SeekFrom::End(-9)).context("seek to footer")?;

    let num_frames = read_u32_le(r)?;
    let descriptor = read_u8(r)?;
    let magic = read_u32_le(r)?;

    if magic != SEEKABLE_MAGIC {
        bail!("invalid seekable magic: 0x{magic:08X}");
    }
    if descriptor != 0x00 {
        bail!("unsupported seek table descriptor: 0x{descriptor:02X}");
    }

    // Seek to start of skippable frame payload (entries start here)
    // Layout from end: [entries: 8*N][footer: 9] + skippable header [magic:4][frame_size:4]
    let entries_size = num_frames as i64 * 8;
    let payload_size = entries_size + 9;
    r.seek(SeekFrom::End(-(payload_size + 8))).context("seek to entries")?;

    // Validate skippable frame magic and frame_size
    let sk_magic = read_u32_le(r)?;
    let sk_frame_size = read_u32_le(r)?;
    if sk_magic != SKIPPABLE_MAGIC {
        bail!("invalid skippable frame magic: 0x{sk_magic:08X}");
    }
    if sk_frame_size != num_frames * 8 + 9 {
        bail!("seek table frame_size mismatch");
    }

    let mut entries = Vec::with_capacity(num_frames as usize);
    for _ in 0..num_frames {
        entries.push(SeekEntry {
            compressed_size: read_u32_le(r)?,
            decompressed_size: read_u32_le(r)?,
        });
    }

    Ok(entries)
}

/// Compute per-frame start offsets (compressed byte offsets from file start).
pub fn frame_offsets(entries: &[SeekEntry]) -> Vec<u64> {
    let mut offsets = Vec::with_capacity(entries.len());
    let mut pos: u64 = 0;
    for e in entries {
        offsets.push(pos);
        pos += e.compressed_size as u64;
    }
    offsets
}

fn write_u32_le<W: Write>(w: &mut W, v: u32) -> anyhow::Result<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn read_u32_le<R: Read>(r: &mut R) -> anyhow::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u8<R: Read>(r: &mut R) -> anyhow::Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn round_trip_empty() {
        let entries: Vec<SeekEntry> = vec![];
        let mut buf = Vec::new();
        write_seek_table(&mut buf, &entries).unwrap();

        let mut cur = Cursor::new(&buf);
        let got = read_seek_table(&mut cur).unwrap();
        assert_eq!(got, entries);
    }

    #[test]
    fn round_trip_multiple() {
        let entries = vec![
            SeekEntry { compressed_size: 512, decompressed_size: 1024 },
            SeekEntry { compressed_size: 300, decompressed_size: 1024 },
            SeekEntry { compressed_size: 200, decompressed_size: 512 },
        ];
        let mut buf = Vec::new();
        write_seek_table(&mut buf, &entries).unwrap();

        // Simulate real file: prepend dummy compressed data
        let dummy = vec![0xABu8; 1012];
        let mut file = dummy.clone();
        file.extend_from_slice(&buf);

        let mut cur = Cursor::new(&file);
        let got = read_seek_table(&mut cur).unwrap();
        assert_eq!(got, entries);
    }

    #[test]
    fn frame_offsets_correct() {
        let entries = vec![
            SeekEntry { compressed_size: 100, decompressed_size: 200 },
            SeekEntry { compressed_size: 150, decompressed_size: 200 },
            SeekEntry { compressed_size: 50, decompressed_size: 100 },
        ];
        let offsets = frame_offsets(&entries);
        assert_eq!(offsets, vec![0, 100, 250]);
    }
}
