//! # TGA Library
//!
//! The library `libtga` is a pure Rust library for reading an writing Truevision
//! TARGA format images. The TGA format is capable of representing multiple 
//! types of bitmapped images including  black and white, indexed colour, RGB
//! colour, and various compressed representations. The minimal implementation
//! is a 24 bit unmapped RGB colour image. This library presently implements 24
//! bit unmapped uncompressed RBG images only.
//!
use std::error;
use std::fmt;
use std::io::Read;
use std::io;


/// The length of a TGA Header is always 18 bytes.
pub const TGA_HEADER_LENGTH: usize = 18;

///
/// A `TgaHeader` type is a structure containing all the infomation about
/// a TGA file.
///
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TgaHeader {
    /// The length of the image identification string, in bytes. The 
    /// image identification data immediately follows the header itself. Further
    /// identification data can be placed after the image data at the end of the
    /// file.
    id_length: u8,
    /// The type of a colour map. This field contains either a `0` or a `1`.
    /// Here, `0` means that no colour map is included, and a `1` indicates that 
    /// a colour map is included. 
    color_map_type: u8,
    /// The type of image. The most common case is an image type of `2` which
    /// indicates an unmapped uncompressed RGB file.
    data_type_code: u8,
    /// The integer index of the first colour map entry.
    colour_map_origin: [u8; 2],
    /// The number of colour map entries.
    colour_map_length: [u8; 2],
    /// The number of bits in a colour map entry. There are 24 bits for a Targa 24 image.
    colour_map_depth: u8,
    /// The X coordinate of the lower left corner of an image, in little endian byte order.
    x_origin: [u8; 2],
    /// The Y coordinate of the lower left corner of an image, in little endian byte order.
    y_origin: [u8; 2],
    /// The width of an image in pixels.
    width: [u8; 2],
    /// The height of an image in pixels.
    height: [u8; 2],
    /// The number of bits per pixel. For an unmapped RGB file this is 24 bits.
    bits_per_pixel: u8,
    /// Image descriptor byte.
    image_descriptor: u8,
}

impl TgaHeader {
    ///
    /// Parse a TGA header from a buffer. We assume that the header to be parsed
    /// starts at the beginning of the buffer.
    ///
    #[inline]
    fn parse_from_buffer(buf: &[u8]) -> Result<TgaHeader, TgaError> {
        // The buffer must be at least the length (in bytes) of a TGA header.
        if buf.len() >= TGA_HEADER_LENGTH {
            let header = TgaHeader {
                id_length: buf[0],
                color_map_type: buf[1],
                data_type_code: buf[2],
                colour_map_origin: [buf[3], buf[4]],
                colour_map_length: [buf[5], buf[6]],
                colour_map_depth: buf[7],
                x_origin: [buf[8], buf[9]],
                y_origin: [buf[10], buf[11]],
                width: [buf[12], buf[13]],
                height: [buf[14], buf[15]],
                bits_per_pixel: buf[16],
                image_descriptor: buf[17],
            };

            return Ok(header);
        }

        Err(TgaError::IncompleteTgaHeader(buf.len(), TGA_HEADER_LENGTH))
    }

    ///
    /// Parse a TGA header from a file or other kind of stream. We assume that the header 
    /// to be parsed starts at the beginning of the buffer. If this is not the case, the
    /// parser will most likely reject the input since it cannot identify a correct header.
    ///
    fn parse_from_file<F: Read>(f: &mut F) -> Result<TgaHeader, TgaError> {
        let mut buf = [0; TGA_HEADER_LENGTH];
        let offset = match f.read(&mut buf) {
            Ok(val) => val as usize,
            Err(_) => return Err(TgaError::CorruptTgaHeader)
        };

        if offset != TGA_HEADER_LENGTH {
            return Err(
                TgaError::IncompleteTgaHeader(offset, TGA_HEADER_LENGTH)
            );
        }
        Self::parse_from_buffer(&buf)
    }

    fn colour_map_size(&self) -> usize {
        let colour_map_length = ((self.colour_map_length[1] as u16) << 8) 
                              | self.colour_map_length[0] as u16;

        // From the TGA specification, the color map depth will be one of
        // 16, 24, or 32 bits; it is always a multiple of 8. Therefore
        // we can always safely divide by 8.
        let colour_map_depth_bytes = (self.colour_map_depth / 8) as u16;

        (colour_map_length * colour_map_depth_bytes) as usize
    }

    ///
    /// The width of a TGA image, in pixels.
    ///
    fn width(&self) -> usize {
        (((self.width[1] as u16) << 8) | (self.width[0] as u16)) as usize
    }

    ///
    /// The height of a TGA image, in pixels.
    ///
    fn height(&self) -> usize {
        ((((self.height[1] as u16) << 8) as u16) | (self.height[0] as u16)) as usize
    }

    ///
    /// The bit depth for each pixel. By default this will be 24 bits as the most
    /// common TGA image type is a 24 bit unmapped uncompressed RGB image.
    ///
    fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel as usize
    }
}

#[derive(Debug)]
pub enum TgaError {
    CorruptTgaHeader,
    Not24BitRgb(usize),
    CorruptIdString(Box<io::Error>),
    CorruptColourMap(Box<io::Error>),
    CorruptImageData(Box<io::Error>),
    IncompleteTgaHeader(usize, usize),
    IncompleteIdString(usize, usize),
    IncompleteColourMap(usize, usize),
    IncompleteImageData(usize, usize),
}

impl fmt::Display for TgaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TgaError::CorruptTgaHeader => {
                write!(f, "CorruptTgaHeader")
            }
            TgaError::Not24BitRgb(got_type_code) => {
                write!(f, "Not24BitRgb(got_type_code={})", got_type_code)
            }
            TgaError::CorruptIdString(_) => {
                write!(f, "CorruptIdString")
            }
            TgaError::CorruptColourMap(_) => {
                write!(f, "CorruptColourMap")
            }
            TgaError::CorruptImageData(_) => {
                write!(f, "CorruptImageData")
            }
            TgaError::IncompleteTgaHeader(have, need) => {
                write!(f, "IncompleteTgaHeader(have={}, need={})", have, need)
            }
            TgaError::IncompleteIdString(have, need) => {
                write!(f, "IncompleteIdString(have={}, need={})", have, need)
            }
            TgaError::IncompleteColourMap(have, need) => {
                write!(f, "IncompleteColourMap(have={}, need={})", have, need)
            }
            TgaError::IncompleteImageData(have, need) => {
                write!(f, "IncompleteImageData(have={}, need={})", have, need)
            }
        }
    }
}

impl error::Error for TgaError {
    fn description(&self) -> &str {
        match *self {
            TgaError::CorruptTgaHeader => {
                "The TGA header data is corrupted."
            }
            TgaError::Not24BitRgb(_) => {
                "The TGA image is not a 24 bit TGA format RGB image."
            }
            TgaError::CorruptIdString(_) => {
                "The image identification is either corrupted, or it is the wrong length."
            }
            TgaError::CorruptColourMap(_) => {
                "The colour map is either corrupted, or it is the wrong length."
            }
            TgaError::CorruptImageData(_) => {
                "The TGA image data is either corrupted, or it is the wrong length."
            }
            TgaError::IncompleteTgaHeader(_,_) => {
                "The file is too small to contain a complete TGA header."
            }
            TgaError::IncompleteIdString(_,_) => {
                "The ID string is too short."
            }
            TgaError::IncompleteColourMap(_,_) => {
                "The TGA image colour map is too short."
            }
            TgaError::IncompleteImageData(_,_) => {
                "The number of pixels in the TGA image does not equal what was reported in the header."
            }
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            TgaError::CorruptTgaHeader => None,
            TgaError::Not24BitRgb(_) => None,
            TgaError::CorruptIdString(ref err) => Some(err),
            TgaError::CorruptColourMap(ref err) => Some(err),
            TgaError::CorruptImageData(ref err) => Some(err),
            TgaError::IncompleteTgaHeader(_,_) => None,
            TgaError::IncompleteIdString(_,_) => None,
            TgaError::IncompleteColourMap(_,_) => None,
            TgaError::IncompleteImageData(_,_) => None,
        }
    }
}

///
/// A `TgaImage` is a structure containing a TGA image. This data type 
/// can represent either 24 bit uncompressed RGB images, or 24 bit
/// run-length encoded RGB images.
///
#[derive(PartialEq, Eq, Debug)]
pub enum TgaImage {
    Type02(UncompressedRgb),
    Type10(RunLengthEncodedRgb),
}

impl TgaImage {
    pub fn parse_from_buffer(buf: &[u8]) -> Result<TgaImage, TgaError> {
        let header = try!(TgaHeader::parse_from_buffer(buf));

        // Determine whether we support the image format. We presently
        // support 24 bit unmapped RGB images only. They can either be 
        // uncompressed (type code 2) or run length encoded (type code 10).
        match header.data_type_code {
            2 => UncompressedRgb::parse_from_buffer(buf, header).map(|image| { 
                TgaImage::Type02(image)
            }),
            10 => RunLengthEncodedRgb::parse_from_buffer(buf, header).map(|image| {
                TgaImage::Type10(image)
            }),
            _ => Err(TgaError::Not24BitRgb(header.data_type_code as usize))
        }
    }

    pub fn parse_from_file<F: Read>(f: &mut F) -> Result<TgaImage, TgaError> {
        let header = try!(TgaHeader::parse_from_file(f));

        // Determine whether we support the image format. We presently
        // support 24 bit unmapped RGB images only. They can either be 
        // uncompressed (type code 2) or run length encoded (type code 10).
        match header.data_type_code {
            2 => UncompressedRgb::parse_from_file(f, header).map(|image| { 
                TgaImage::Type02(image)
            }),
            10 => RunLengthEncodedRgb::parse_from_file(f, header).map(|image| {
                TgaImage::Type10(image)
            }),
            _ => Err(TgaError::Not24BitRgb(header.data_type_code as usize))
        }
    }

    ///
    /// The function `width` returns the width of a TGA image, in pixels.
    ///
    pub fn width(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.width(),
            &TgaImage::Type10(ref image) => image.width()
        }
    }

    ///
    /// Return the height of a TGA image, in pixels.
    ///
    pub fn height(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.height(),
            &TgaImage::Type10(ref image) => image.height()
        }
    }

    ///
    /// Return the bit depth per pixel in a TGA Image.
    ///
    pub fn bits_per_pixel(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.bits_per_pixel(),
            &TgaImage::Type10(ref image) => image.bits_per_pixel()
        }
    }

    ///
    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
    ///
    pub fn color_map_type(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.color_map_type(),
            &TgaImage::Type10(ref image) => image.color_map_type()
        }
    }

    pub fn data_type_code(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.data_type_code(),
            &TgaImage::Type10(ref image) => image.data_type_code()
        }
    }

    ///
    /// The function `header` produces a copy of the TGA header.
    ///
    pub fn header(&self) -> TgaHeader {
        match self {
            &TgaImage::Type02(ref image) => image.header(),
            &TgaImage::Type10(ref image) => image.header()
        }
    }

    ///
    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    ///
    pub fn pixels(&self) -> PixelIter {
        match self {
            &TgaImage::Type02(ref image) => image.pixels(),
            &TgaImage::Type10(ref image) => image.pixels()
        }
    }

    ///
    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    ///
    pub fn image_data_length(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.image_data_length(),
            &TgaImage::Type10(ref image) => image.image_data_length()
        }
    }

    ///
    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    ///
    pub fn image_data_length_bytes(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.image_data_length_bytes(),
            &TgaImage::Type10(ref image) => image.image_data_length_bytes()
        }
    }

    ///
    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    ///
    pub fn image_identification(&self) -> &[u8] {
        match self {
            &TgaImage::Type02(ref image) => image.image_identification(),
            &TgaImage::Type10(ref image) => image.image_identification()
        }
    }

    ///
    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
    ///
    pub fn extended_image_identification(&self) -> &[u8] {
        match self {
            &TgaImage::Type02(ref image) => image.extended_image_identification(),
            &TgaImage::Type10(ref image) => image.extended_image_identification()
        }
    }
}

///
/// A `RawTgaImage` is a structure containing the underlying raw TGA image data.
///
#[derive(PartialEq, Eq, Debug)]
pub struct RawTgaImage {
    /// The TGA header.
    header: TgaHeader,
    /// The image identification data. This is typically omitted, but it can
    /// up to 255 character long. If more data is needed, it can be placed
    /// after the image data.
    image_identification: Box<Vec<u8>>,
    /// The colour map data, as specified by the colour map specification.
    colour_map_data: Box<Vec<u8>>,
    /// The raw pixels themselves.
    image_data: Box<Vec<u8>>,
    /// The extended image identification data. This field is the spillover from
    /// the image identification field if the image identification data is too
    /// long to fit into the image indentification field.
    extended_image_identification: Box<Vec<u8>>,
}

impl RawTgaImage {
    ///
    /// Construct a new TGA image.
    ///
    pub fn new(
        header: TgaHeader, 
        image_identification: Box<Vec<u8>>, 
        colour_map_data: Box<Vec<u8>>, 
        image_data: Box<Vec<u8>>,
        extended_image_identification: Box<Vec<u8>>
    ) -> RawTgaImage {
        RawTgaImage {
            header: header, 
            image_identification: image_identification, 
            colour_map_data: colour_map_data, 
            image_data: image_data,
            extended_image_identification: extended_image_identification,
        }
    }

    ///
    /// The function `width` returns the width of a TGA image, in pixels.
    ///
    #[inline]
    pub fn width(&self) -> usize {
        self.header.width()
    }

    ///
    /// Return the height of a TGA image, in pixels.
    ///
    #[inline]
    pub fn height(&self) -> usize {
        self.header.height()
    }

    ///
    /// Return the bit depth per pixel in a TGA Image.
    ///
    #[inline]
    pub fn bits_per_pixel(&self) -> usize {
        self.header.bits_per_pixel()
    }

    ///
    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
    ///
    #[inline]
    pub fn color_map_type(&self) -> usize {
        self.header.color_map_type as usize
    }

    #[inline]
    pub fn data_type_code(&self) -> usize {
        self.header.data_type_code as usize
    }

    ///
    /// The function `header` produces a copy of the TGA header.
    ///
    #[inline]
    pub fn header(&self) -> TgaHeader {
        self.header
    }

    ///
    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    ///
    #[inline]
    pub fn pixels(&self) -> PixelIter {
        PixelIter {
            inner: self.image_data.as_slice(),
            current: [0; 3],
            index: 0,
        }
    }

    ///
    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    ///
    #[inline]
    pub fn image_data_length(&self) -> usize {
        self.image_data.len() / 3
    }

    ///
    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    ///
    #[inline]
    pub fn image_data_length_bytes(&self) -> usize {
        self.image_data.len()
    }

    ///
    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    ///
    #[inline]
    pub fn image_identification(&self) -> &[u8] {
        &self.image_identification
    }

    ///
    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
    ///
    #[inline]
    pub fn extended_image_identification(&self) -> &[u8] {
        &self.extended_image_identification
    }
}

pub struct PixelIter<'a> {
    inner: &'a [u8],
    current: [u8; 3],
    index: usize,
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = [u8; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.inner.len() {
            self.current[0] = self.inner[self.index];
            self.current[1] = self.inner[self.index + 1];
            self.current[2] = self.inner[self.index + 2];
            self.index += 3;
            
            return Some(self.current);
        }

        None
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct UncompressedRgb {
    inner: RawTgaImage,
}

impl UncompressedRgb {
    ///
    /// Parse a unmapped uncompressed TGA image from a buffer in memory. 
    /// We assume that the image to be parsed starts at the beginning of the buffer. 
    /// In order to parse correctly, the bytes of the buffer must conform to the TGA 
    /// image format.
    ///
    pub fn parse_from_buffer(buf: &[u8], header: TgaHeader) -> Result<Self, TgaError> {
        // Check that we were passed the correct TGA header.
        if header.data_type_code != 2 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        // Targa 24 images can also be embedded in Targa 32 images, but we do
        // not implement that (yet).
        if header.bits_per_pixel != 24 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        if buf.len() < header.id_length as usize + TGA_HEADER_LENGTH {
            return Err(TgaError::CorruptTgaHeader);
        }

        let slice = &buf[TGA_HEADER_LENGTH..buf.len()];
        let mut bytes = slice.bytes();
        let mut image_identification = Box::new(Vec::new());
        for i in 0..header.id_length {
            let byte = bytes.next();
            match byte {
                Some(Ok(val)) => image_identification.push(val),
                Some(Err(err)) => {
                    return Err(
                        TgaError::CorruptIdString(Box::new(err))
                    );
                }
                None => {
                    return Err(
                        TgaError::IncompleteIdString(i as usize, header.id_length as usize)
                    );
                }
            }
        }

        let colour_map_size = header.colour_map_size();
        let mut colour_map_data = Box::new(Vec::new());
        for i in 0..colour_map_size {
            let byte = bytes.next();
            match byte {
                Some(Ok(val)) => colour_map_data.push(val),
                Some(Err(err)) => {
                    return Err(
                        TgaError::CorruptColourMap(Box::new(err))
                    );
                }
                None => {
                    return Err(
                        TgaError::IncompleteColourMap(i as usize, colour_map_size)
                    );
                }
            }
        }

        let width = header.width();
        let height = header.height();
        let bytes_per_pixel = (header.bits_per_pixel / 8) as usize;
        let image_size = width * height * bytes_per_pixel;
        
        let mut image_data = Box::new(Vec::new());
        for i in 0..image_size {
            let byte = bytes.next();
            match byte {
                Some(Ok(val)) => image_data.push(val),
                Some(Err(err)) => {
                    return Err(
                        TgaError::CorruptImageData(Box::new(err))
                    );
                }
                None => {
                    return Err(
                        TgaError::IncompleteImageData(i, image_size)
                    );
                }
            }
        }

        // Parse the extended image identification information from the end
        // of the image data field.
        let extended_image_identification = Box::new(
            bytes.map(|byte| byte.unwrap()).collect::<Vec<u8>>()
        );

        let inner = RawTgaImage::new(
            header, image_identification, colour_map_data, image_data, extended_image_identification
        );

        let image = UncompressedRgb { inner: inner };

        Ok(image)
    }

    ///
    /// Parse a TGA image from a file or stream. We assume that the image to be parsed
    /// starts at the beginning of the buffer. In order to parse correctly,
    /// the bytes of the file or stream must conform to the TGA image format. The 
    /// first 18 bytes of the image should be the TGA header.
    ///
    pub fn parse_from_file<F: Read>(f: &mut F, header: TgaHeader) -> Result<Self, TgaError> {
        // Determine whether we support the image format. We presently
        // support 24 bit unmapped RGB images only. They can either be 
        // uncompressed (type code 2) or run length encoded (type code 10).
        if header.data_type_code != 2 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        // Targa 24 images can also be embedded in Targa 32 images, but we do
        // not implement that (yet).
        if header.bits_per_pixel != 24 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        let mut bytes = f.bytes();
        let mut image_identification = Box::new(Vec::new());
        for i in 0..header.id_length {
            let byte = bytes.next();
            match byte {
                Some(Ok(val)) => image_identification.push(val),
                Some(Err(err)) => {
                    return Err(
                        TgaError::CorruptIdString(Box::new(err))
                    );
                }
                None => {
                    return Err(
                        TgaError::IncompleteIdString(i as usize, header.id_length as usize)
                    );
                }
            }
        }

        let colour_map_size = header.colour_map_size();
        let mut colour_map_data = Box::new(Vec::new());
        for i in 0..colour_map_size {
            let byte = bytes.next();
            match byte {
                Some(Ok(val)) => colour_map_data.push(val),
                Some(Err(err)) => {
                    return Err(
                        TgaError::CorruptColourMap(Box::new(err))
                    );
                }
                None => {
                    return Err(
                        TgaError::IncompleteColourMap(i as usize, colour_map_size)
                    );
                }
            }
        }

        let width = header.width();
        let height = header.height();
        let bytes_per_pixel = (header.bits_per_pixel / 8) as usize;
        let image_size = width * height * bytes_per_pixel;
        
        let mut image_data = Box::new(Vec::new());
        for i in 0..image_size {
            let byte = bytes.next();
            match byte {
                Some(Ok(val)) => image_data.push(val),
                Some(Err(err)) => {
                    return Err(
                        TgaError::CorruptImageData(Box::new(err))
                    );
                }
                None => {
                    return Err(
                        TgaError::IncompleteImageData(i, image_size)
                    );
                }
            }
        }

        // Parse the extended image identification information from the end
        // of the image data field.
        let extended_image_identification = Box::new(
            bytes.map(|byte| byte.unwrap()).collect::<Vec<u8>>()
        );

        let inner = RawTgaImage::new(
            header, image_identification, colour_map_data, image_data, extended_image_identification
        );

        let image = UncompressedRgb { inner: inner };

        Ok(image)        
    }

    ///
    /// The function `width` returns the width of a TGA image, in pixels.
    ///
    #[inline]
    pub fn width(&self) -> usize {
        self.inner.width()
    }

    ///
    /// Return the height of a TGA image, in pixels.
    ///
    #[inline]
    pub fn height(&self) -> usize {
        self.inner.height()
    }

    ///
    /// Return the bit depth per pixel in a TGA Image.
    ///
    #[inline]
    pub fn bits_per_pixel(&self) -> usize {
        self.inner.bits_per_pixel()
    }

    ///
    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
    ///
    #[inline]
    pub fn color_map_type(&self) -> usize {
        self.inner.color_map_type()
    }

    #[inline]
    pub fn data_type_code(&self) -> usize {
        self.inner.data_type_code()
    }

    ///
    /// The function `header` produces a copy of the TGA header.
    ///
    #[inline]
    pub fn header(&self) -> TgaHeader {
        self.inner.header()
    }

    ///
    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    ///
    #[inline]
    pub fn pixels(&self) -> PixelIter {
        self.inner.pixels()
    }

    ///
    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    ///
    #[inline]
    pub fn image_data_length(&self) -> usize {
        self.inner.image_data_length()
    }

    ///
    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    ///
    #[inline]
    pub fn image_data_length_bytes(&self) -> usize {
        self.inner.image_data_length_bytes()
    }

    ///
    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    ///
    #[inline]
    pub fn image_identification(&self) -> &[u8] {
        self.inner.image_identification()
    }

    ///
    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
    ///
    #[inline]
    pub fn extended_image_identification(&self) -> &[u8] {
        self.inner.extended_image_identification()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct RunLengthEncodedRgb {
    inner: RawTgaImage,
}

impl RunLengthEncodedRgb {
    pub fn parse_from_buffer(buf: &[u8], header: TgaHeader) -> Result<RunLengthEncodedRgb, TgaError> {
        unimplemented!()
    }

    pub fn parse_from_file<F: Read>(f: &mut F, header: TgaHeader)-> Result<RunLengthEncodedRgb, TgaError> {
        unimplemented!()
    }

    ///
    /// The function `width` returns the width of a TGA image, in pixels.
    ///
    #[inline]
    pub fn width(&self) -> usize {
        self.inner.width()
    }

    ///
    /// Return the height of a TGA image, in pixels.
    ///
    #[inline]
    pub fn height(&self) -> usize {
        self.inner.height()
    }

    ///
    /// Return the bit depth per pixel in a TGA Image.
    ///
    #[inline]
    pub fn bits_per_pixel(&self) -> usize {
        self.inner.bits_per_pixel()
    }

    ///
    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
    ///
    #[inline]
    pub fn color_map_type(&self) -> usize {
        self.inner.color_map_type()
    }

    #[inline]
    pub fn data_type_code(&self) -> usize {
        self.inner.data_type_code()
    }

    ///
    /// The function `header` produces a copy of the TGA header.
    ///
    #[inline]
    pub fn header(&self) -> TgaHeader {
        self.inner.header()
    }

    ///
    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    ///
    #[inline]
    pub fn pixels(&self) -> PixelIter {
        self.inner.pixels()
    }

    ///
    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    ///
    #[inline]
    pub fn image_data_length(&self) -> usize {
        self.inner.image_data_length()
    }

    ///
    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    ///
    #[inline]
    pub fn image_data_length_bytes(&self) -> usize {
        self.inner.image_data_length_bytes()
    }

    ///
    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    ///
    #[inline]
    pub fn image_identification(&self) -> &[u8] {
        self.inner.image_identification()
    }

    ///
    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
    ///
    #[inline]
    pub fn extended_image_identification(&self) -> &[u8] {
        self.inner.extended_image_identification()
    }
}

