use std::{
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::Context;

use crate::seek_table::{frame_offsets, read_seek_table, SeekEntry};

pub struct DecompressInfo {
    pub entries: Vec<SeekEntry>,
    pub uncompressed_size: u64,
}

impl DecompressInfo {
    pub fn frame_count(&self) -> u32 {
        self.entries.len() as u32
    }
}

/// Read the seek table from a compressed file.
pub fn read_info(path: &Path) -> anyhow::Result<DecompressInfo> {
    let mut f = File::open(path).context("open file")?;
    let entries = read_seek_table(&mut f).context("read seek table")?;
    let uncompressed_size = entries.iter().map(|e| e.decompressed_size as u64).sum();
    Ok(DecompressInfo { entries, uncompressed_size })
}

/// Open a .kov file and return (file handle, seek entries, per-frame compressed offsets).
fn open_kov(path: &Path) -> anyhow::Result<(File, Vec<SeekEntry>, Vec<u64>)> {
    let mut f = File::open(path).context("open input")?;
    let entries = read_seek_table(&mut f).context("read seek table")?;
    let offsets = frame_offsets(&entries);
    Ok((f, entries, offsets))
}

/// Decompress the full file from `input` to `output`.
pub fn decompress_full(input: &Path, output: &Path) -> anyhow::Result<u64> {
    let (mut f, entries, offsets) = open_kov(input)?;
    let out = File::create(output).context("create output")?;
    let mut writer = BufWriter::new(out);
    let mut total = 0u64;

    for (i, entry) in entries.iter().enumerate() {
        f.seek(SeekFrom::Start(offsets[i])).context("seek to frame")?;
        let mut frame_buf = vec![0u8; entry.compressed_size as usize];
        f.read_exact(&mut frame_buf).context("read frame")?;
        let decompressed = zstd::decode_all(frame_buf.as_slice()).context("zstd decode")?;
        writer.write_all(&decompressed).context("write decompressed")?;
        total += decompressed.len() as u64;
    }

    writer.flush()?;
    Ok(total)
}

/// Decompress only the byte range [offset, offset+len) to `output`.
pub fn decompress_range(
    input: &Path,
    output: &Path,
    offset: u64,
    len: u64,
) -> anyhow::Result<u64> {
    let (mut f, entries, offsets) = open_kov(input)?;

    let uncompressed_total: u64 = entries.iter().map(|e| e.decompressed_size as u64).sum();
    let end = offset.checked_add(len).context("offset + len overflows u64")?;
    anyhow::ensure!(
        end <= uncompressed_total,
        "range [{offset}, {end}) exceeds file size {uncompressed_total}"
    );

    let out = File::create(output).context("create output")?;
    let mut writer = BufWriter::new(out);

    let mut decompressed_cursor: u64 = 0;
    let mut written = 0u64;

    for (i, entry) in entries.iter().enumerate() {
        let frame_start = decompressed_cursor;
        let frame_end = frame_start + entry.decompressed_size as u64;

        if frame_end <= offset {
            decompressed_cursor = frame_end;
            continue;
        }
        if frame_start >= end {
            break;
        }

        f.seek(SeekFrom::Start(offsets[i])).context("seek to frame")?;
        let mut frame_buf = vec![0u8; entry.compressed_size as usize];
        f.read_exact(&mut frame_buf).context("read frame")?;
        let decompressed = zstd::decode_all(frame_buf.as_slice()).context("zstd decode")?;

        let slice_start = (offset.saturating_sub(frame_start)) as usize;
        // Clamp to actual decompressed buffer length, not the seek table's declared size
        let slice_end = (end - frame_start).min(decompressed.len() as u64) as usize;
        writer.write_all(&decompressed[slice_start..slice_end])?;
        written += (slice_end - slice_start) as u64;

        decompressed_cursor = frame_end;
    }

    writer.flush()?;
    Ok(written)
}
