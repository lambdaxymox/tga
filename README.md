# TGA Image Format Library
This package is a Rust implementation of the Truevision TGA image format. In particular, this repository implements the features necessary to read and write 24 bit RGB TGA image files. Further details about the TGA format can be found [here](http://paulbourke.net/dataformats/tga/), and also [here](https://www.loc.gov/preservation/digital/formats/fdd/fdd000180.shtml). This repository supports both run length encoded RGB images as well as uncompressed ones. The primary intention of this library is for working with textures for computer graphics applications.

## Usage
To use `tga`, add the following line to your `Cargo.toml` file.
```toml
[dependencies]
# ...
tga = "0.2.17"
# ...
```
Then in your `lib.rs`, `main.rs` or wherever your main source file is, add the crate declaration
```rust
extern crate tga;
```
and you are ready to use `tga`.

## Dependencies
The TGA image format library has no external dependencies in release. It requires `rust` version 1.24 or later as well as a recent version of `cargo` to build the library.