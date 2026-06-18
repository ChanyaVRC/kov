# encode

Compress a file into kov format.

```sh
kov encode [OPTIONS] <INPUT> <OUTPUT>
```

## Options

| Flag | Default | Description |
|---|---|---|
| `-l, --level <1-22>` | `3` | Zstd compression level. Higher = smaller file, slower. |
| `-f, --frame-size <BYTES>` | `1048576` | Uncompressed bytes per frame (1 MiB). Larger frames compress better; smaller frames allow finer-grained random access. |
| `-t, --threads <N>` | CPU count | Number of parallel compression threads. |

## Examples

```sh
# Default settings
kov encode input.bin output.kov

# Maximum compression, 2 MiB frames
kov encode input.bin output.kov --level 19 --frame-size 2097152

# Limit to 4 threads
kov encode input.bin output.kov --threads 4
```

## Output

```
Encoded: 8.0 MiB bytes → 3.2 MiB bytes (8 frames, 2.50x ratio)
```

## Memory usage

All compressed frames are buffered in memory before writing.
Peak memory ≈ compressed size of the input (typically 20–80% of the original).
Reduce `--frame-size` if memory is constrained.
