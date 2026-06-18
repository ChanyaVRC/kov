# Output Format

kov produces standard [Zstd seekable format](https://github.com/facebook/zstd/blob/dev/contrib/seekable_format/zstd_seekable_compression_format.md) files.

## File layout

```
┌──────────────┐
│  zstd Frame 0│  ← independently decompressible
├──────────────┤
│  zstd Frame 1│
├──────────────┤
│     ...      │
├──────────────┤
│  zstd Frame N│
├──────────────┤
│  Seek Table  │  ← skippable frame at EOF
└──────────────┘
```

## Seek table (skippable frame)

The seek table is encoded as a zstd skippable frame with magic `0x184D2A5E`:

```
Magic:        0x184D2A5E  (4 bytes LE)
Frame size:   8*N + 9     (4 bytes LE)

Per-frame entries (N × 8 bytes):
  compressed_size:    u32 LE
  decompressed_size:  u32 LE

Footer (9 bytes):
  num_frames:         u32 LE
  descriptor:         0x00
  seekable_magic:     0x8F92EAB1  (4 bytes LE)
```

## Compatibility

Because the format follows the Zstd seekable spec exactly, frames can be decompressed individually by any standard `zstd` binary:

```sh
# Extract a single frame with standard zstd (after computing its offset)
dd if=archive.kov bs=1 skip=<frame_offset> count=<frame_compressed_size> | zstd -d -o frame.bin
```

The seek table skippable frame is silently ignored by tools that do not understand it.
