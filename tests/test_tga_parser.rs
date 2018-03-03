extern crate tga;

use std::fs::File;
use std::io::Read;
use tga::TgaImage;

mod sample;


struct TestCase<'a> {
    filename: &'a str,
    width: usize,
    height: usize,
    bits_per_pixel: usize,
    color_map_type: usize,
    data_type_code: usize,
}

struct TestCases<'a> {
    tests: Vec<TestCase<'a>>,
}

fn test_cases<'a>() -> TestCases<'a> {
    TestCases {
        tests: vec![
            TestCase {
                filename: sample::LENA_TGA,
                width: sample::LENA_TGA_WIDTH,
                height: sample::LENA_TGA_HEIGHT,
                bits_per_pixel: sample::LENA_TGA_BITS_PER_PIXEL,
                color_map_type: sample::LENA_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::LENA_TGA_DATA_TYPE_CODE,
            },
            TestCase {
                filename: sample::COLOR_TGA,
                width: sample::COLOR_TGA_WIDTH,
                height: sample::COLOR_TGA_HEIGHT,
                bits_per_pixel: sample::COLOR_TGA_BITS_PER_PIXEL,
                color_map_type: sample::COLOR_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::COLOR_TGA_DATA_TYPE_CODE,
            },
        ]
    }
}


///
/// The TGA image parser should be able to take a valid existing TGA
/// file, and then parse it into an image.
///
#[test]
fn test_parse_from_file_succeeds() {
    for test_case in test_cases().tests {
        let filename = test_case.filename;
        let mut file = File::open(filename).unwrap();
        let image = TgaImage::parse_from_file(&mut file);
        
        assert!(image.is_ok());
    }
}

///
/// The TGA parser should be able to take a buffer in memory
/// containing valid TGA data, and parse it into an image.
///
#[test]
fn test_parse_from_buffer() {
    for test_case in test_cases().tests {
        let filename = test_case.filename;
        let mut file = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        
        file.read_to_end(&mut buffer).unwrap();
        let image = TgaImage::parse_from_buffer(&buffer);
        
        assert!(image.is_ok());
    }
}

///
/// The TGA image parser should correctly parse the TGA header data.
/// Given a TGA image with a known height, width, pixel depth, etc.,
/// The TGA image parser should recognize the exact same parameters from
/// the file. If there is a failure, either the image is wrong, or the parser
/// is incorrectly reading the data.
///
#[test]
fn test_parsed_tga_image_matches_expected_header_data() {
    for test_case in test_cases().tests {
        let filename = test_case.filename;
        let mut file = File::open(filename).unwrap();
        let image = TgaImage::parse_from_file(&mut file).unwrap();

        assert_eq!(image.width(), test_case.width);
        assert_eq!(image.height(), test_case.height);
        assert_eq!(image.bits_per_pixel(), test_case.bits_per_pixel);
        assert_eq!(image.color_map_type(), test_case.color_map_type);
        assert_eq!(image.data_type_code(), test_case.data_type_code);
    }
}

///
/// The parsed TGA image should satsify the following invariant.
/// ```
/// image.image_data_length() == image.width() * image.height()
/// ```
///
/// That is, the image data field of a TGA image should contain every pixel
/// from the image file, and no more.
///
#[test]
fn test_tga_image_should_have_correct_width_and_height() {
    for test_case in test_cases().tests {
        let filename = test_case.filename;
        let mut file = File::open(filename).unwrap();
        let image = TgaImage::parse_from_file(&mut file).unwrap();

        assert_eq!(image.image_data_length(), image.width() * image.height());
    }
}

///
/// The TGA image parser should get the same contents from a TGA image regardless
/// of whether the image came directly from a file, or if it came from a buffer
/// in memory.
///
#[test]
fn test_parse_from_buffer_and_parse_from_file_should_have_the_same_contents() {
    for test_case in test_cases().tests {
        let filename = test_case.filename;
        let mut file = File::open(filename).unwrap();
        let image_from_file = TgaImage::parse_from_file(&mut file).unwrap();
    
        let mut file = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let image_from_buffer = TgaImage::parse_from_buffer(&buffer).unwrap();

        assert_eq!(image_from_file, image_from_buffer);
    }
}

///
/// The TGA image parser should get the same contents from a TGA image regardless
/// of whether the image came directly from a file, or if it came from a buffer
/// in memory. Here we do it in another way using an iterator.
///
#[test]
fn test_tga_image_iterator() {
    let mut file = File::open(sample::LENA_TGA).unwrap();
    let image_from_file = TgaImage::parse_from_file(&mut file).unwrap();
    
    let mut file = File::open(sample::LENA_TGA).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let image_from_buffer = TgaImage::parse_from_buffer(&buffer).unwrap();

    let pixels_from_file = image_from_file.pixels();
    let pixels_from_buffer = image_from_buffer.pixels();

    for (pixel_ff, pixel_fb) in pixels_from_file.zip(pixels_from_buffer) {
        assert_eq!(pixel_ff, pixel_fb);
    }
}

///
/// The TGA image pixel iterator should return every pixel in the image.
///
#[test]
fn test_tga_image_iterator_should_return_every_pixel_in_image() {
    for test_case in test_cases().tests {
        let filename = test_case.filename;
        let mut file = File::open(filename).unwrap();
        let image = TgaImage::parse_from_file(&mut file).unwrap();
        let pixels = image.pixels().collect::<Vec<[u8; 3]>>();

        assert_eq!(pixels.len(), image.image_data_length());
    }
}

///
/// In a TGA image where every pixel has one color, each pixel in the image data
/// should be exactly the same value.
///
#[test]
fn test_tga_image_should_with_one_color_should_have_every_pixel_the_same_color() {
    let mut file = File::open(sample::COLOR_TGA).unwrap();
    let image = TgaImage::parse_from_file(&mut file).unwrap();

    let mut pixels = image.pixels();
    let first_pixel = pixels.next().unwrap();

    assert!(pixels.all(|pixel| pixel == first_pixel));
}
