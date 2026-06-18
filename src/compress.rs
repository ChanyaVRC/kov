use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Context;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::seek_table::{SeekEntry, write_seek_table};

pub struct CompressOptions {
    pub level: i32,
    pub frame_size: usize,
    pub threads: Option<usize>,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            level: 3,
            frame_size: 1024 * 1024,
            threads: None,
        }
    }
}

/// Compress `input` to `output` using parallel zstd frame encoding.
/// Returns (uncompressed_bytes, compressed_bytes, frame_count).
pub fn compress_file(
    input: &Path,
    output: &Path,
    opts: &CompressOptions,
) -> anyhow::Result<(u64, u64, u32)> {
    let in_file = File::open(input).context("open input")?;
    let mmap = unsafe { Mmap::map(&in_file).context("mmap input")? };

    if let Some(n) = opts.threads
        && let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
    {
        eprintln!("warning: could not set thread count to {n}: {e}");
    }

    let data: &[u8] = &mmap;
    let uncompressed_len = data.len() as u64;

    let chunks: Vec<&[u8]> = if data.is_empty() {
        vec![]
    } else {
        data.chunks(opts.frame_size).collect()
    };

    let level = opts.level;

    // Compress all chunks in parallel; propagate errors via Result collection
    let compressed: Vec<Vec<u8>> = chunks
        .par_iter()
        .map(|chunk| zstd::encode_all(*chunk, level).context("zstd encode failed"))
        .collect::<anyhow::Result<_>>()?;

    let out_file = File::create(output).context("create output")?;
    let mut writer = BufWriter::new(out_file);

    let mut entries: Vec<SeekEntry> = Vec::with_capacity(compressed.len());
    let mut total_compressed: u64 = 0;

    // Single pass: decompressed_size taken directly from each original chunk length
    for (frame, chunk) in compressed.iter().zip(chunks.iter()) {
        let cs = u32::try_from(frame.len()).context("compressed frame exceeds 4 GiB")?;
        let ds = u32::try_from(chunk.len()).context("frame size exceeds 4 GiB")?;
        entries.push(SeekEntry {
            compressed_size: cs,
            decompressed_size: ds,
        });
        writer.write_all(frame).context("write frame")?;
        total_compressed += cs as u64;
    }

    write_seek_table(&mut writer, &entries).context("write seek table")?;
    writer.flush().context("flush output")?;

    // seek table size = 8 (skippable header) + 8*N + 9
    let seek_table_size = 8 + entries.len() as u64 * 8 + 9;
    total_compressed += seek_table_size;

    Ok((uncompressed_len, total_compressed, entries.len() as u32))
}
