# info

Show seek table metadata for a compressed file.

```sh
kov info <INPUT>
```

## Example

```sh
kov info archive.kov
```

```
Frames:            8
Uncompressed size: 8.0 MiB

Frame          Compressed     Decompressed
--------------------------------------------
0               382.0 KiB        1.0 MiB
1               381.8 KiB        1.0 MiB
2               380.2 KiB        1.0 MiB
3               379.5 KiB        1.0 MiB
4               381.1 KiB        1.0 MiB
5               380.7 KiB        1.0 MiB
6               381.3 KiB        1.0 MiB
7               161.1 KiB      512.0 KiB
```

The seek table is read from the end of the file without decompressing any data frames.
Use this to verify the frame count and locate the byte range you need before running `decode --range`.
