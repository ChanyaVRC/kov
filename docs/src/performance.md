# Performance

## Benchmark results

**Environment:** 50 MiB per dataset · x86_64 Linux · 32 cores · 1 MiB frame size  
**Tools:** gzip 1.10 · bzip2 1.0.8 · lz4 1.9.4 · zstd 1.5.5 · kov 0.1.0  
Each dataset is freshly generated immediately before measurement (no page-cache warm-up).

---

### Mixed (source code + logs + binary)

| Tool | Time | Output | Ratio | Speed |
|---|---|---|---|---|
| gzip -6 (default) | 1119 ms | 37.5 MiB | 1.33x | 45 MB/s |
| bzip2 | 2477 ms | 32.3 MiB | 1.55x | 20 MB/s |
| lz4 (default) | 77 ms | 43.5 MiB | 1.15x | 649 MB/s |
| lz4 -9 | 939 ms | 39.2 MiB | 1.28x | 53 MB/s |
| zstd -3 (default, 1T) | 185 ms | 37.1 MiB | 1.35x | 270 MB/s |
| zstd -9 (1T) | 731 ms | 36.1 MiB | 1.38x | 68 MB/s |
| zstd -3 -T0 (all cores) | 54 ms | 37.1 MiB | 1.35x | 926 MB/s |
| zstd -9 -T0 (all cores) | 380 ms | 36.1 MiB | 1.38x | 132 MB/s |
| kov -l 3 -t 1 (zstd, 1T) | 162 ms | 37.2 MiB | 1.34x | 309 MB/s |
| **kov -l 3 (zstd, 32T)** | **45 ms** | 37.2 MiB | 1.34x | **1111 MB/s** |
| kov -l 9 (zstd, 32T) | 195 ms | 36.5 MiB | 1.37x | 256 MB/s |

---

### Random text (high compressibility)

| Tool | Time | Output | Ratio | Speed |
|---|---|---|---|---|
| gzip -6 (default) | 636 ms | 18.1 MiB | 2.76x | 79 MB/s |
| bzip2 | 999 ms | 11.4 MiB | **4.38x** | 50 MB/s |
| lz4 (default) | 105 ms | 33.3 MiB | 1.50x | 476 MB/s |
| lz4 -9 | 522 ms | 20.4 MiB | 2.45x | 96 MB/s |
| zstd -3 (default, 1T) | 185 ms | 16.6 MiB | 3.02x | 270 MB/s |
| zstd -9 (1T) | 936 ms | 16.5 MiB | 3.03x | 53 MB/s |
| zstd -3 -T0 (all cores) | 55 ms | 16.6 MiB | 3.02x | 909 MB/s |
| zstd -9 -T0 (all cores) | 417 ms | 16.5 MiB | 3.03x | 120 MB/s |
| kov -l 3 -t 1 (zstd, 1T) | 189 ms | 16.6 MiB | 3.01x | 265 MB/s |
| **kov -l 3 (zstd, 32T)** | **40 ms** | 16.6 MiB | 3.01x | **1250 MB/s** |
| kov -l 9 (zstd, 32T) | 166 ms | 16.7 MiB | 3.00x | 301 MB/s |

---

### Random bytes (incompressible)

| Tool | Time | Output | Ratio | Speed |
|---|---|---|---|---|
| gzip -6 (default) | 810 ms | 50.0 MiB | 1.00x | 62 MB/s |
| bzip2 | 2974 ms | **50.2 MiB** | <1.00x | 17 MB/s |
| lz4 (default) | 39 ms | 50.0 MiB | 1.00x | 1282 MB/s |
| lz4 -9 | 713 ms | 50.0 MiB | 1.00x | 70 MB/s |
| zstd -3 (default, 1T) | 41 ms | 50.0 MiB | 1.00x | 1220 MB/s |
| zstd -9 (1T) | 77 ms | 50.0 MiB | 1.00x | 649 MB/s |
| **zstd -3 -T0 (all cores)** | **33 ms** | 50.0 MiB | 1.00x | **1515 MB/s** |
| zstd -9 -T0 (all cores) | 66 ms | 50.0 MiB | 1.00x | 758 MB/s |
| kov -l 3 -t 1 (zstd, 1T) | 42 ms | 50.0 MiB | 1.00x | 1190 MB/s |
| kov -l 3 (zstd, 32T) | 43 ms | 50.0 MiB | 1.00x | 1163 MB/s |
| kov -l 9 (zstd, 32T) | 82 ms | 50.0 MiB | 1.00x | 610 MB/s |

---

## Observations

**Compressible data (mixed / text):**
- kov -l 3 (parallel) is the **fastest overall** — 45 ms vs zstd -T0's 54 ms on mixed data. The per-frame rayon dispatch has lower synchronisation overhead than zstd's internal thread pool at this file size.
- lz4 (default) is the fastest single-threaded option but trades ratio (1.15x vs 1.34x on mixed).
- bzip2 achieves the best ratio on text (4.38x) at the cost of being 25× slower than kov parallel.
- zstd -9 -T0 scales poorly (380 ms) — the level-9 work-unit is too fine-grained to parallelise efficiently at 1 MiB/frame.

**Incompressible data (random bytes):**
- All tools produce output the same size as input — ratio is 1.00x.
- gzip spends 810 ms searching for patterns before giving up. zstd and lz4 detect incompressibility early and exit in 33–42 ms.
- bzip2 **expands** the file (50.2 MiB) due to block headers, and still takes 3 seconds.
- kov shows no parallel speedup here: compression is I/O-bound, and the seek table overhead (~850 B) is negligible.

---

## Decompression speed

Measured on the mixed dataset:

| Tool | Time | Speed |
|---|---|---|
| gzip -d | 236 ms | 212 MB/s |
| **kov decode** (full) | **96 ms** | **521 MB/s** |
| **kov decode --range 0:1048576** | **12 ms** | 1 frame only |

---

## Parallel compression

kov splits the input into frames and compresses them concurrently using all available CPU cores (via [rayon](https://github.com/rayon-rs/rayon)).

```sh
kov encode input.bin output.kov --threads 4
```

---

## Frame size trade-offs

| Frame size | Compression ratio | Random access granularity | Peak memory |
|---|---|---|---|
| 256 KiB | Lower | Fine | Lower |
| 1 MiB (default) | Good | Medium | Medium |
| 4 MiB | Better | Coarse | Higher |

---

## Random access cost

`decode --range OFFSET:LEN` decompresses at most `⌈LEN / frame_size⌉ + 1` frames regardless of total file size.

For a 10 GiB file with 1 MiB frames, reading any 4 KiB window requires decompressing at most 2 frames (2 MiB of work), not 10 GiB.
