# TGA Image Format Library
This package is a Rust implementation of the minimal implementation of the Truevision TGA image format. In particular, this repository implements the features necessary to read and write 24 bit RGB TGA image files, as detailed [here](http://paulbourke.net/dataformats/tga/). This repository supports both run length encoded RGB images as well as uncompressed ones.

## Usage
To use `libtga`, add the following line to your `Cargo.toml` file.
```toml
[dependencies]
# ...
libtga = "0.2.8"
# ...
```
Then in your `lib.rs`, `main.rs` or wherever your main source file is, add the crate declaration
```rust
extern crate tga;
```
and you are ready to use `libtga`.

## Dependencies
The TGA image format library has no external dependencies in release. It requires `rust` version 1.24 or later as well as a recent version of `cargo` to build the library.