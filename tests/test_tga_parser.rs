extern crate tga;

use std::fs::File;
use std::io::Read;
use tga::TgaImage;

const SAMPLE_IMAGE: &str = "sample/lena.tga";


#[test]
fn test_files_exist() {
    let file = File::open(SAMPLE_IMAGE);
    assert!(file.is_ok());
}

#[test]
fn test_parse_from_file_succeeds() {
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let image = TgaImage::parse_from_file(&mut file);
    assert!(image.is_ok());
}

#[test]
fn test_parse_from_file() {
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let image = TgaImage::parse_from_file(&mut file).unwrap();
    assert_eq!(image.width(), 256);
    assert_eq!(image.height(), 256);
    assert_eq!(image.bits_per_pixel(), 24);
    assert_eq!(image.color_map_type(), 0);
    assert_eq!(image.data_type_code(), 2);
}

#[test]
fn test_parse_from_buffer_image_data_should_be_non_empty() {
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let image = TgaImage::parse_from_file(&mut file).unwrap();
    assert_ne!(image.image_data_length(), 0);
    assert_eq!(image.image_data_length(), image.width() * image.height());
}

#[test]
fn test_parse_from_buffer() {
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let image = TgaImage::parse_from_buffer(&buffer);
    assert!(image.is_ok());
}

#[test]
fn test_parse_from_buffer_and_parse_from_file_should_yield_same_file() {
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let image_from_file = TgaImage::parse_from_file(&mut file).unwrap();
    
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let image_from_buffer = TgaImage::parse_from_buffer(&buffer).unwrap();

    assert_eq!(image_from_file, image_from_buffer);
}

#[test]
fn test_tga_image_iterator() {
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let image_from_file = TgaImage::parse_from_file(&mut file).unwrap();
    
    let mut file = File::open(SAMPLE_IMAGE).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let image_from_buffer = TgaImage::parse_from_buffer(&buffer).unwrap();

    let pixels_from_file = image_from_file.pixels();
    let pixels_from_buffer = image_from_buffer.pixels();

    for (pixel_ff, pixel_fb) in pixels_from_file.zip(pixels_from_buffer) {
        assert_eq!(pixel_ff, pixel_fb);
    } 
}
