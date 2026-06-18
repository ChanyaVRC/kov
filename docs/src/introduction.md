# kov — Keyed Offset Vault

**kov** (**K**eyed **O**ffset **V**ault) is a command-line file compressor that uses [Zstd seekable format](https://github.com/facebook/zstd/blob/dev/contrib/seekable_format/zstd_seekable_compression_format.md) to enable fast parallel compression and byte-range decompression.

## Why kov?

Standard compression tools (gzip, plain zstd) require decompressing from the beginning of a file to reach any given offset. kov solves this by dividing files into independent frames:

```
[Frame 0][Frame 1][Frame 2]...[Frame N][Seek Table]
```

- **Parallel compression** — frames are compressed concurrently with rayon across all available CPU cores
- **Random access** — decompress any byte range using `--range` without touching other frames
- **Standard format** — the output is valid Zstd seekable format, readable by any compatible tool

## Quick start

```sh
# Compress
kov encode large_file.bin large_file.kov

# Decompress
kov decode large_file.kov large_file.bin

# Read only bytes 1 MiB–1 MiB+4 KiB
kov decode large_file.kov chunk.bin --range 1048576:4096
```
