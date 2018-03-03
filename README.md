# TGA Image Format Library
This package is a Rust implementation of the minimal implementation of the Truevision TGA image format. In particular, this repository implements the features necessary to read and write 24 bit uncompressed RGB TGA image files, as detailed [here](http://paulbourke.net/dataformats/tga/).

## Usage
To use `libtga`, add the following line to your `Cargo.toml` file.
```
[dependencies]
libtga = "0.2.3"
```
Then in your `lib.rs`, `main.rs` or wherever your main source file is, add the crate declaration
```rust
extern crate tga;
```
and you are ready to roll.

## Dependencies
The TGA image format library has no external dependencies. It is designed to be a pure Rust implementation of the Truevision TGA image format.
