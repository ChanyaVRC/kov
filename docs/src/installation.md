# Installation

## Pre-built binaries

Download the binary for your platform from the [Releases page](https://github.com/YOUR_USERNAME/kov/releases) and place it somewhere on your `PATH`.

| Platform | File |
|---|---|
| Linux x86\_64 | `kov-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Linux aarch64 | `kov-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86\_64 | `kov-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `kov-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| Windows x86\_64 | `kov-vX.Y.Z-x86_64-pc-windows-msvc.zip` |

## Build from source

Requires [Rust](https://rustup.rs) 1.75 or later.

```sh
git clone https://github.com/YOUR_USERNAME/kov
cd kov
cargo install --path .
```

Verify the installation:

```sh
kov --version
```
