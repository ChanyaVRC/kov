mod compress;
mod decompress;
mod seek_table;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

use compress::{compress_file, CompressOptions};
use decompress::{decompress_full, decompress_range, read_info};

#[derive(Parser)]
#[command(name = "kov", about = "Parallel zstd seekable file compressor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a file
    Encode {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long, default_value = "3", value_parser = clap::value_parser!(i32).range(1..=22))]
        level: i32,
        #[arg(short, long, default_value = "1048576")]
        frame_size: usize,
        #[arg(short, long)]
        threads: Option<usize>,
    },
    /// Decompress a file (full or partial range)
    Decode {
        input: PathBuf,
        output: PathBuf,
        /// Partial decompress: OFFSET:LEN in bytes (e.g. 4096:512)
        #[arg(short, long, value_name = "OFFSET:LEN")]
        range: Option<String>,
    },
    /// Show seek table info for a compressed file
    Info {
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode { input, output, level, frame_size, threads } => {
            let opts = CompressOptions { level, frame_size, threads };
            let (uncompressed, compressed, frames) =
                compress_file(&input, &output, &opts).context("compression failed")?;
            let ratio = uncompressed as f64 / compressed as f64;
            println!(
                "Encoded: {} bytes → {} bytes ({frames} frames, {ratio:.2}x ratio)",
                fmt_size(uncompressed),
                fmt_size(compressed),
            );
        }

        Commands::Decode { input, output, range } => {
            let written = if let Some(r) = range {
                let (offset, len) = parse_range(&r)?;
                decompress_range(&input, &output, offset, len).context("partial decompress failed")?
            } else {
                decompress_full(&input, &output).context("decompress failed")?
            };
            println!("Decoded: {} bytes", fmt_size(written));
        }

        Commands::Info { input } => {
            let info = read_info(&input).context("read info failed")?;
            println!("Frames:            {}", info.frame_count());
            println!("Uncompressed size: {}", fmt_size(info.uncompressed_size));
            println!();
            println!("{:<8} {:>16} {:>16}", "Frame", "Compressed", "Decompressed");
            println!("{}", "-".repeat(44));
            for (i, e) in info.entries.iter().enumerate() {
                println!(
                    "{:<8} {:>16} {:>16}",
                    i,
                    fmt_size(e.compressed_size as u64),
                    fmt_size(e.decompressed_size as u64),
                );
            }
        }
    }

    Ok(())
}

fn parse_range(s: &str) -> anyhow::Result<(u64, u64)> {
    let (a, b) = s
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--range must be OFFSET:LEN, got '{s}'"))?;
    let offset: u64 = a.parse().context("invalid offset")?;
    let len: u64 = b.parse().context("invalid len")?;
    Ok((offset, len))
}

fn fmt_size(n: u64) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{:.1} KiB", n as f64 / 1024.0)
    } else if n < 1024 * 1024 * 1024 {
        format!("{:.1} MiB", n as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GiB", n as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
