extern crate tga;

use std::fs::File;
use std::slice;
use std::io::Read;

mod sample;


#[derive(Copy, Clone, Debug)]
struct TestCaseConfig<'a> {
    filename: &'a str,
    width: usize,
    height: usize,
    bits_per_pixel: usize,
    color_map_type: usize,
    data_type_code: usize,
}

#[derive(Clone, Debug)]
struct TestCase<'a> {
    filename: &'a str,
    width: usize,
    height: usize,
    bits_per_pixel: usize,
    color_map_type: usize,
    data_type_code: usize,
    image: Vec<u8>,
}

impl<'a> TestCase<'a> {
    fn new(config: TestCaseConfig) -> TestCase {
        let mut file = File::open(config.filename).unwrap();
        let mut image = Vec::new();
        file.read_to_end(&mut image).unwrap();

        TestCase {
            filename: config.filename,
            width: config.width,
            height: config.height,
            bits_per_pixel: config.bits_per_pixel,
            color_map_type: config.color_map_type,
            data_type_code: config.data_type_code,
            image: image,
        }
    }

    fn as_slice(&self) -> &[u8] {
        self.image.as_slice()
    }

}

#[derive(Clone, Debug)]
struct Test<'a> {
    tests: Vec<TestCase<'a>>,
}

impl<'a> Test<'a> {
    fn iter(&self) -> TestIter {
        TestIter {
            inner: self.tests.iter(),
        }
    }
}

struct TestIter<'a> {
    inner: slice::Iter<'a, TestCase<'a>>,
}

impl<'a> Iterator for TestIter<'a> {
    type Item = &'a TestCase<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

fn test_cases<'a>() -> Test<'a> {
    Test {
        tests: vec![
            TestCase::new(TestCaseConfig {
                filename: sample::LENA_TGA,
                width: sample::LENA_TGA_WIDTH,
                height: sample::LENA_TGA_HEIGHT,
                bits_per_pixel: sample::LENA_TGA_BITS_PER_PIXEL,
                color_map_type: sample::LENA_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::LENA_TGA_DATA_TYPE_CODE,
            }),
            TestCase::new(TestCaseConfig {
                filename: sample::COLOR_TGA,
                width: sample::COLOR_TGA_WIDTH,
                height: sample::COLOR_TGA_HEIGHT,
                bits_per_pixel: sample::COLOR_TGA_BITS_PER_PIXEL,
                color_map_type: sample::COLOR_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::COLOR_TGA_DATA_TYPE_CODE,
            }),
            TestCase::new(TestCaseConfig {
                filename: sample::ONE_TGA,
                width: sample::ONE_TGA_WIDTH,
                height: sample::ONE_TGA_HEIGHT,
                bits_per_pixel: sample::ONE_TGA_BITS_PER_PIXEL,
                color_map_type: sample::ONE_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::ONE_TGA_DATA_TYPE_CODE,
            }),
        ]
    }
}

fn test_cases_rle<'a>() -> Test<'a> {
    Test {
        tests: vec![
            TestCase::new(TestCaseConfig {
                filename: sample::LENA_RLE_TGA,
                width: sample::LENA_RLE_TGA_WIDTH,
                height: sample::LENA_RLE_TGA_HEIGHT,
                bits_per_pixel: sample::LENA_RLE_TGA_BITS_PER_PIXEL,
                color_map_type: sample::LENA_RLE_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::LENA_RLE_TGA_DATA_TYPE_CODE,
            }),
            TestCase::new(TestCaseConfig {
                filename: sample::COLOR_RLE_TGA,
                width: sample::COLOR_RLE_TGA_WIDTH,
                height: sample::COLOR_RLE_TGA_HEIGHT,
                bits_per_pixel: sample::COLOR_RLE_TGA_BITS_PER_PIXEL,
                color_map_type: sample::COLOR_RLE_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::COLOR_RLE_TGA_DATA_TYPE_CODE,
            }),
            TestCase::new(TestCaseConfig {
                filename: sample::ONE_RLE_TGA,
                width: sample::ONE_RLE_TGA_WIDTH,
                height: sample::ONE_RLE_TGA_HEIGHT,
                bits_per_pixel: sample::ONE_RLE_TGA_BITS_PER_PIXEL,
                color_map_type: sample::ONE_RLE_TGA_COLOR_MAP_TYPE,
                data_type_code: sample::ONE_RLE_TGA_DATA_TYPE_CODE,
            }),
        ]
    }
}

#[cfg(test)]
mod tests_unmapped_rgb {
    use std::fs::File;
    use tga::TgaImage;
    use super::sample;


    /// The TGA image parser should be able to take a valid existing TGA
    /// file, and then parse it into an image.
    #[test]
    fn test_parse_from_file_succeeds() {
        for test_case in super::test_cases().iter() {
            let image = TgaImage::parse_from_file(&mut test_case.as_slice());
        
            assert!(image.is_ok());
        }
    }

    /// The TGA parser should be able to take a buffer in memory
    /// containing valid TGA data, and parse it into an image.
    #[test]
    fn test_parse_from_buffer_succeeds() {
        for test_case in super::test_cases().iter() {
            let image = TgaImage::parse_from_buffer(test_case.as_slice());
        
            assert!(image.is_ok());
        }
    }

    /// The TGA image parser should correctly parse the TGA header data.
    /// Given a TGA image with a known height, width, pixel depth, etc.,
    /// The TGA image parser should recognize the exact same parameters from
    /// the file. If there is a failure, either the image is wrong, or the parser
    /// is incorrectly reading the data.
    #[test]
    fn test_parsed_tga_image_matches_expected_header_data() {
        for test_case in super::test_cases().iter() {
            let image = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();

            assert_eq!(image.width(), test_case.width);
            assert_eq!(image.height(), test_case.height);
            assert_eq!(image.bits_per_pixel(), test_case.bits_per_pixel);
            assert_eq!(image.color_map_type(), test_case.color_map_type);
            assert_eq!(image.data_type_code(), test_case.data_type_code);
        }
    }

    /// The parsed TGA image should satsify the following invariant.
    /// ```
    /// image.image_data_length() == image.width() * image.height()
    /// ```
    ///
    /// That is, the image data field of a TGA image should contain every pixel
    /// from the image file, and no more.
    #[test]
    fn test_tga_image_should_have_correct_width_and_height() {
        for test_case in super::test_cases().iter() {
            let image = TgaImage::parse_from_file(&mut test_case.as_slice()).unwrap();

            assert_eq!(image.image_data_length(), image.width() * image.height());
        }
    }

    /// The TGA image parser should get the same contents from a TGA image regardless
    /// of whether the image came directly from a file, or if it came from a buffer
    /// in memory.
    #[test]
    fn test_parse_from_buffer_and_parse_from_file_should_be_equal() {
        for test_case in super::test_cases().iter() {
            let image_from_file = TgaImage::parse_from_file(&mut test_case.as_slice()).unwrap();
            let image_from_buffer = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();

            assert_eq!(image_from_file, image_from_buffer);
        }
    }

    /// The TGA image parser should get the same contents from a TGA image regardless
    /// of whether the image came directly from a file, or if it came from a buffer
    /// in memory. Here we do it in another way using an iterator.
    #[test]
    fn test_tga_image_pixel_iterator() {
        for test_case in super::test_cases().iter() {
            let image_from_file = TgaImage::parse_from_file(&mut test_case.as_slice()).unwrap();
            let image_from_buffer = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();

            let pixels_ff = image_from_file.pixels();
            let pixels_fb = image_from_buffer.pixels();
            
            assert!(pixels_ff.zip(pixels_fb).all(
                |(pixel_ff, pixel_fb)| {
                    pixel_ff == pixel_fb 
                }
            ));
        }
    }

    #[test]
    fn test_tga_image_scanline_iterator() {
        let mut file = File::open(sample::LENA_TGA).unwrap();
        let image = TgaImage::parse_from_file(&mut file).unwrap();
    
        let scanlines = image.scanlines();
        let slice = image.image_data();
        let scanlines_from_pixels = slice.chunks(3 * image.width());

        assert!(scanlines.zip(scanlines_from_pixels).all(
            |(scanline, scanline_from_pixels)| { scanline == *scanline_from_pixels }
        ));
    }

    /// The TGA image pixel iterator should return every pixel in the image.
    #[test]
    fn test_tga_image_iterator_should_return_every_pixel_in_image() {
        for test_case in super::test_cases().iter() {
            let image = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();
            let pixels = image.pixels().collect::<Vec<[u8; 3]>>();

            assert_eq!(pixels.len(), image.image_data_length());
        }
    }

    /// In a TGA image where every pixel has one color, each pixel in the image data
    /// should have exactly the same value.
    #[test]
    fn test_tga_image_should_with_one_color_should_return_the_same_color_with_every_pixel() {
        let mut file = File::open(sample::COLOR_TGA).unwrap();
        let image = TgaImage::parse_from_file(&mut file).unwrap();

        let mut pixels = image.pixels();
        let first_pixel = pixels.next().unwrap();

        assert!(pixels.all(|pixel| pixel == first_pixel));
    }

}

#[cfg(test)]
mod tests_rle_rgb {
    use std::fs::File;
    use tga::TgaImage;
    use super::sample;

    /// The TGA image parser should be able to take a valid existing TGA
    /// file, and then parse it into an image.
    #[test]
    fn test_parse_from_file_succeeds() {
        for test_case in super::test_cases_rle().iter() {
            let image = TgaImage::parse_from_file(&mut test_case.as_slice());
        
            assert!(image.is_ok());
        }
    }

    /// The TGA parser should be able to take a buffer in memory
    /// containing valid TGA data, and parse it into an image.
    #[test]
    fn test_parse_from_buffer() {
        for test_case in super::test_cases_rle().iter() {
            let image = TgaImage::parse_from_buffer(test_case.as_slice());
        
            assert!(image.is_ok());
        }
    }

    /// The TGA image parser should correctly parse the TGA header data.
    /// Given a TGA image with a known height, width, pixel depth, etc.,
    /// The TGA image parser should recognize the exact same parameters from
    /// the file. If there is a failure, either the image is wrong, or the parser
    /// is incorrectly reading the data.
    #[test]
    fn test_parsed_tga_image_matches_expected_header_data() {
        for test_case in super::test_cases_rle().iter() {
            let image = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();

            assert_eq!(image.width(), test_case.width);
            assert_eq!(image.height(), test_case.height);
            assert_eq!(image.bits_per_pixel(), test_case.bits_per_pixel);
            assert_eq!(image.color_map_type(), test_case.color_map_type);
            assert_eq!(image.data_type_code(), test_case.data_type_code);
        }
    }

    /// The parsed TGA image should satsify the following invariant.
    /// ```
    /// image.image_data_length() == image.width() * image.height()
    /// ```
    ///
    /// That is, the image data field of a TGA image should contain every pixel
    /// from the image file, and no more.
    #[test]
    fn test_tga_image_should_have_correct_width_and_height() {
        for test_case in super::test_cases_rle().iter() {
            let image = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();

            assert_eq!(image.image_data_length(), image.width() * image.height());
        }
    }

    /// The TGA image parser should get the same contents from a TGA image regardless
    /// of whether the image came directly from a file, or if it came from a buffer
    /// in memory.
    #[test]
    fn test_parse_from_buffer_and_parse_from_file_should_have_the_same_contents() {
        for test_case in super::test_cases_rle().iter() {
            let image_from_file = TgaImage::parse_from_file(&mut test_case.as_slice()).unwrap();
            let image_from_buffer = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();

            assert_eq!(image_from_file, image_from_buffer);
        }
    }

    /// The TGA image parser should get the same contents from a TGA image regardless
    /// of whether the image came directly from a file, or if it came from a buffer
    /// in memory. Here we do it in another way using an iterator.
    #[test]
    fn test_tga_image_pixel_iterator() {
        for test_case in super::test_cases().iter() {
            let image_from_file = TgaImage::parse_from_file(&mut test_case.as_slice()).unwrap();
            let image_from_buffer = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();

            let pixels_ff = image_from_file.pixels();
            let pixels_fb = image_from_buffer.pixels();

            assert!(pixels_ff.zip(pixels_fb).all(
                |(pixel_ff, pixel_fb)| {
                    pixel_ff == pixel_fb
                }
            ));
        }
    }

    /// The TGA image pixel iterator should return every pixel in the image.
    #[test]
    fn test_tga_image_iterator_should_return_every_pixel_in_image() {
        for test_case in super::test_cases_rle().iter() {
            let image = TgaImage::parse_from_file(&mut test_case.as_slice()).unwrap();
            let pixels = image.pixels().collect::<Vec<[u8; 3]>>();

            assert_eq!(pixels.len(), image.image_data_length());
        }
    }

    /// In a TGA image where every pixel has one color, each pixel in the image data
    /// should have exactly the same value.
    #[test]
    fn test_tga_image_should_with_one_color_should_return_the_same_color_with_every_pixel() {
        let mut file = File::open(sample::COLOR_RLE_TGA).unwrap();
        let image = TgaImage::parse_from_file(&mut file).unwrap();

        let mut pixels = image.pixels();
        let first_pixel = pixels.next().unwrap();

        assert!(pixels.all(|pixel| pixel == first_pixel));
    }

    /// An decoded RLE encoded TGA image should be exactly the same as an uncompressed
    /// RGB image.
    #[test]
    fn test_a_decoded_rle_rgb_tga_image_should_be_the_same_as_an_unmapped_one() {
        let mut file_rle = File::open(sample::LENA_RLE_TGA).unwrap();
        let image_rle = TgaImage::parse_from_file(&mut file_rle).unwrap();

        let mut file = File::open(sample::LENA_TGA).unwrap();
        let image = TgaImage::parse_from_file(&mut file).unwrap();

        assert_eq!(image_rle.image_data(), image.image_data());
    }
}

#[cfg(test)]
mod tests_tga_reader {
    use tga::TgaImage;
    use tga::TgaReader;
    use std::io::Read;

    #[test]
    fn test_tga_reader_output_should_match_raw_unmapped_rgb_file() {
        for test_case in super::test_cases().iter() {
            let image = TgaImage::parse_from_buffer(test_case.as_slice()).unwrap();
            let mut reader = TgaReader::new(&image);
            let mut buf = vec![0; test_case.as_slice().len()];
            reader.read(&mut buf).unwrap();

            assert_eq!(buf.as_slice(), test_case.as_slice());
        }
    }
}
