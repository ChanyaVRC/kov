use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Context;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::seek_table::{write_seek_table, SeekEntry};

pub struct CompressOptions {
    pub level: i32,
    pub frame_size: usize,
    pub threads: Option<usize>,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self { level: 3, frame_size: 1024 * 1024, threads: None }
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

    // Configure rayon thread pool if requested
    if let Some(n) = opts.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok(); // ignore error if pool already initialised
    }

    let data: &[u8] = &mmap;
    let uncompressed_len = data.len() as u64;

    // Split into frame-sized chunks
    let chunks: Vec<&[u8]> = if data.is_empty() {
        vec![]
    } else {
        data.chunks(opts.frame_size).collect()
    };

    let level = opts.level;

    // Compress all chunks in parallel
    let compressed: Vec<Vec<u8>> = chunks
        .par_iter()
        .map(|chunk| zstd::encode_all(*chunk, level).expect("zstd encode failed"))
        .collect();

    let out_file = File::create(output).context("create output")?;
    let mut writer = BufWriter::new(out_file);

    let mut entries: Vec<SeekEntry> = Vec::with_capacity(compressed.len());
    let mut total_compressed: u64 = 0;

    for frame in &compressed {
        let cs = frame.len() as u32;
        // decompressed size for last frame may be smaller than frame_size
        entries.push(SeekEntry {
            compressed_size: cs,
            decompressed_size: 0, // filled below
        });
        writer.write_all(frame).context("write frame")?;
        total_compressed += cs as u64;
    }

    // Fill decompressed sizes from original chunk sizes
    let frame_size = opts.frame_size;
    for (i, entry) in entries.iter_mut().enumerate() {
        let start = i * frame_size;
        let end = (start + frame_size).min(data.len());
        entry.decompressed_size = (end - start) as u32;
    }

    write_seek_table(&mut writer, &entries).context("write seek table")?;
    writer.flush().context("flush output")?;

    // seek table size = 8 (skippable header) + 8*N + 9
    let seek_table_size = 8 + entries.len() as u64 * 8 + 9;
    total_compressed += seek_table_size;

    Ok((uncompressed_len, total_compressed, entries.len() as u32))
}
