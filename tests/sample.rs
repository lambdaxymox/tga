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


#[cfg(test)]
mod tests {
    use std::fs::File;

    fn file_exists(file: &str) -> bool {
        File::open(file).is_ok()
    }

    #[test]
    fn test_file_exists1() {
        assert!(file_exists(super::LENA_TGA));
    }

    #[test]
    fn test_file_exists2() {
        assert!(file_exists(super::COLOR_TGA));
    }

    #[test]
    fn test_file_exists3() {
        assert!(file_exists(super::ONE_TGA));
    }
}
