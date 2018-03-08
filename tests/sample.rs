pub const LENA_TGA: &str = "sample/lena.tga";
pub const LENA_TGA_WIDTH: usize = 512;
pub const LENA_TGA_HEIGHT: usize = 512;
pub const LENA_TGA_BITS_PER_PIXEL: usize = 24;
pub const LENA_TGA_COLOR_MAP_TYPE: usize = 0;
pub const LENA_TGA_DATA_TYPE_CODE: usize = 2;

pub const COLOR_TGA: &str = "sample/color.tga";
pub const COLOR_TGA_WIDTH: usize = 640;
pub const COLOR_TGA_HEIGHT: usize = 480;
pub const COLOR_TGA_BITS_PER_PIXEL: usize = 24;
pub const COLOR_TGA_COLOR_MAP_TYPE: usize = 0;
pub const COLOR_TGA_DATA_TYPE_CODE: usize = 2;

pub const ONE_TGA: &str = "sample/one.tga";
pub const ONE_TGA_WIDTH: usize = 1;
pub const ONE_TGA_HEIGHT: usize = 1;
pub const ONE_TGA_BITS_PER_PIXEL: usize = 24;
pub const ONE_TGA_COLOR_MAP_TYPE: usize = 0;
pub const ONE_TGA_DATA_TYPE_CODE: usize = 2;

pub const LENA_RLE_TGA: &str = "sample/lena_rle.tga";
pub const LENA_RLE_TGA_WIDTH: usize = 512;
pub const LENA_RLE_TGA_HEIGHT: usize = 512;
pub const LENA_RLE_TGA_BITS_PER_PIXEL: usize = 24;
pub const LENA_RLE_TGA_COLOR_MAP_TYPE: usize = 0;
pub const LENA_RLE_TGA_DATA_TYPE_CODE: usize = 2;

pub const COLOR_RLE_TGA: &str = "sample/color_rle.tga";
pub const COLOR_RLE_TGA_WIDTH: usize = 640;
pub const COLOR_RLE_TGA_HEIGHT: usize = 480;
pub const COLOR_RLE_TGA_BITS_PER_PIXEL: usize = 24;
pub const COLOR_RLE_TGA_COLOR_MAP_TYPE: usize = 0;
pub const COLOR_RLE_TGA_DATA_TYPE_CODE: usize = 2;


#[cfg(test)]
mod tests {
    use std::fs::File;

    struct TestCase<'a> {
        data: Vec<&'a str>,
    }

    fn test_cases<'a>() -> TestCase<'a> {
        TestCase {
            data: vec![
                super::LENA_TGA,     super::COLOR_TGA,     super::ONE_TGA, 
                super::LENA_RLE_TGA, super::COLOR_RLE_TGA,
            ]
        }
    }

    fn file_exists(file: &str) -> bool {
        File::open(file).is_ok()
    }

    #[test]
    fn test_files_exist() {
        for test_case in test_cases().data {
            assert!(file_exists(test_case));
        }
    }
}
