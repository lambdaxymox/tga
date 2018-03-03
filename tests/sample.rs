pub const LENA_TGA: &str = "sample/lena.tga";
pub const COLOR_TGA: &str = "sample/color.tga";


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
}
