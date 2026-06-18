# decode

Decompress a kov file — either fully or a specific byte range.

```sh
kov decode [OPTIONS] <INPUT> <OUTPUT>
```

## Options

| Flag | Description |
|---|---|
| `-r, --range <OFFSET:LEN>` | Decompress only `LEN` bytes starting at uncompressed offset `OFFSET`. |

## Examples

```sh
# Full decompression
kov decode archive.kov original.bin

# Read 512 bytes starting at offset 4096
kov decode archive.kov chunk.bin --range 4096:512

# Read the first 1 MiB
kov decode archive.kov first_mib.bin --range 0:1048576
```

## How partial decompression works

kov finds the frames that overlap the requested range and decompresses only those, skipping all other frames entirely. A 10 GiB file with 1 MiB frames requires decompressing at most 1–2 frames to satisfy any `--range` request.

```
Requested: [offset, offset+len)
                    ┌───────────┐
[Frame 0][Frame 1]  │[Frame 2]  │[Frame 3]...[Frame N][Seek Table]
                    └───────────┘
                    Only this frame is decompressed
```
