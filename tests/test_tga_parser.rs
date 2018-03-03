extern crate tga;

use std::fs::File;
use std::io::Read;
use tga::TgaImage;

const SAMPLE_IMAGE1: &str = "sample/lena.tga";
const SAMPLE_IMAGE2: &str = "sample/color.tga";


#[test]
fn test_files_exist() {
    let file = File::open(SAMPLE_IMAGE1);
    assert!(file.is_ok());
    let file = File::open(SAMPLE_IMAGE2);
    assert!(file.is_ok());
}

#[test]
fn test_parse_from_file_succeeds() {
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let image = TgaImage::parse_from_file(&mut file);
    assert!(image.is_ok());
}

#[test]
fn test_parse_from_file() {
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let image = TgaImage::parse_from_file(&mut file).unwrap();
    assert_eq!(image.width(), 512);
    assert_eq!(image.height(), 512);
    assert_eq!(image.bits_per_pixel(), 24);
    assert_eq!(image.color_map_type(), 0);
    assert_eq!(image.data_type_code(), 2);
}

#[test]
fn test_parse_from_buffer_image_data_should_be_non_empty() {
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let image = TgaImage::parse_from_file(&mut file).unwrap();

    assert_ne!(image.image_data_length(), 0);
    assert_eq!(image.image_data_length(), image.width() * image.height());
}

#[test]
fn test_parse_from_buffer() {
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let image = TgaImage::parse_from_buffer(&buffer);
    assert!(image.is_ok());
}

#[test]
fn test_parse_from_buffer_and_parse_from_file_should_yield_same_file() {
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let image_from_file = TgaImage::parse_from_file(&mut file).unwrap();
    
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let image_from_buffer = TgaImage::parse_from_buffer(&buffer).unwrap();

    assert_eq!(image_from_file, image_from_buffer);
}

#[test]
fn test_tga_image_iterator() {
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let image_from_file = TgaImage::parse_from_file(&mut file).unwrap();
    
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let image_from_buffer = TgaImage::parse_from_buffer(&buffer).unwrap();

    let pixels_from_file = image_from_file.pixels();
    let pixels_from_buffer = image_from_buffer.pixels();

    for (pixel_ff, pixel_fb) in pixels_from_file.zip(pixels_from_buffer) {
        assert_eq!(pixel_ff, pixel_fb);
    }
}

#[test]
fn test_tga_image_iterator_should_return_every_pixel_in_image() {
    let mut file = File::open(SAMPLE_IMAGE1).unwrap();
    let image = TgaImage::parse_from_file(&mut file).unwrap();
    let pixels = image.pixels().collect::<Vec<[u8; 3]>>();

    assert_eq!(pixels.len(), image.image_data_length());
}

#[test]
fn test_tga_image_should_with_one_color_should_have_every_pixel_the_same_color() {
    let mut file = File::open(SAMPLE_IMAGE2).unwrap();
    let image = TgaImage::parse_from_file(&mut file).unwrap();

    let mut pixels = image.pixels();
    let first_pixel = pixels.next().unwrap();

    assert!(pixels.all(|pixel| pixel == first_pixel));
}
