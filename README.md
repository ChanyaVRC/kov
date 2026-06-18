# kov — Keyed Offset Vault

Parallel file compressor using [Zstd seekable format](https://github.com/facebook/zstd/blob/dev/contrib/seekable_format/zstd_seekable_compression_format.md).

Each file is split into independent frames and compressed in parallel with rayon. An offset table appended at the end of the file makes every frame directly addressable — decompress any byte range without touching the rest.

## Install

```sh
cargo install --path .
```

Or download a pre-built binary from [Releases](../../releases).

## Usage

### Encode

```sh
kov encode input.bin output.kov
kov encode input.bin output.kov --level 9 --frame-size 2097152 --threads 4
```

| Flag | Default | Description |
|---|---|---|
| `-l, --level` | `3` | Zstd compression level (1–22) |
| `-f, --frame-size` | `1048576` | Frame size in bytes (1 MiB) |
| `-t, --threads` | CPU count | Compression thread count |

### Decode

```sh
# Full decompression
kov decode input.kov output.bin

# Partial decompression — only decompress 512 bytes starting at offset 4096
kov decode input.kov output.bin --range 4096:512
```

### Info

```sh
kov info input.kov
```

```
Frames:            8
Uncompressed size: 8.0 MiB

Frame          Compressed     Decompressed
--------------------------------------------
0               382.0 KiB        1.0 MiB
1               381.8 KiB        1.0 MiB
...
```

## Performance

50 MiB mixed dataset (source code + logs + binary), 32-core x86_64 Linux.

| Tool | Time | Ratio | Speed |
|---|---|---|---|
| gzip -6 | 1119 ms | 1.33x | 45 MB/s |
| lz4 (default) | 77 ms | 1.15x | 649 MB/s |
| zstd -3 (1T) | 185 ms | 1.35x | 270 MB/s |
| zstd -3 -T0 | 54 ms | 1.35x | 926 MB/s |
| **kov -l 3 (32T)** | **45 ms** | 1.34x | **1111 MB/s** |

→ [Full benchmark details](https://YOUR_USERNAME.github.io/kov/performance.html) (3 datasets including random text and random bytes)

## Output format

```
[zstd Frame 0][zstd Frame 1]...[zstd Frame N][Seek Table]
```

The seek table is a standard zstd skippable frame (`0x184D2A5E`) compatible with any tool that understands the seekable format spec. Each frame is independently decompressible with any standard zstd decoder.

## Memory usage

All compressed frames are held in memory before writing. Peak memory ≈ compressed size of the input (typically 20–80% of the original). For files larger than available RAM, reduce `--frame-size` so frames are streamed individually — or contact us for a streaming write path.

## Build

```sh
cargo build --release
cargo test
cargo clippy -- -D warnings
```
