//! # TGA Library
//!
//! The library `tga` is a pure Rust library for reading and writing Truevision
//! TARGA format images. The TGA format is capable of representing multiple 
//! types of bitmapped images including  black and white, indexed colour, RGB
//! colour, and various compressed representations. The minimal implementation
//! is a 24 bit unmapped RGB colour image. This library presently implements 24
//! bit unmapped uncompressed RBG images only.
use std::error;
use std::fmt;
use std::io;
use std::rc::Rc;


/// The length of a TGA Header is always 18 bytes.
pub const TGA_HEADER_LENGTH: usize = 18;

const TGA_FOOTER: [u8; 26] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x54, 0x52, 0x55, 0x45, 0x56, 0x49, 0x53, 0x49,
    0x4F, 0x4E, 0x2D, 0x58, 0x46, 0x49, 0x4C, 0x45, 
    0x2E, 0x00
];

/// A `TgaHeader` type is a structure containing all the infomation about
/// a TGA file.
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
    /// Parse a TGA header from a buffer. We assume that the header to be parsed
    /// starts at the beginning of the buffer. If this is not the case, the
    /// parser will most likely reject the input since it cannot identify a 
    /// correct header.
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

    /// The width of a TGA image, in pixels.
    #[inline]
    fn width(&self) -> usize {
        (((self.width[1] as u16) << 8) | (self.width[0] as u16)) as usize
    }

    /// The height of a TGA image, in pixels.
    #[inline]
    fn height(&self) -> usize {
        ((((self.height[1] as u16) << 8) as u16) | (self.height[0] as u16)) as usize
    }

    /// The bit depth for each pixel. By default this will be 24 bits as the most
    /// common TGA image type is a 24 bit unmapped uncompressed RGB image.
    #[inline]
    fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel as usize
    }

    #[inline]
    fn bytes_per_pixel(&self) -> usize {
        (self.bits_per_pixel / 8) as usize
    }

    #[inline]
    fn colour_map_length(&self) -> usize {
        (((self.colour_map_length[1] as u16) << 8) | (self.colour_map_length[0] as u16)) as usize
    }

    #[inline]
    fn colour_map_depth(&self) -> usize {
        self.colour_map_depth as usize
    }

    #[inline]
    fn colour_map_size(&self) -> usize {
        let colour_map_length = ((self.colour_map_length[1] as u16) << 8) 
                              | self.colour_map_length[0] as u16;

        // From the TGA specification, the color map depth will be one of
        // 16, 24, or 32 bits; it is always a multiple of 8. Therefore
        // we can always safely divide by 8.
        let colour_map_depth_bytes = (self.colour_map_depth / 8) as u16;

        (colour_map_length * colour_map_depth_bytes) as usize
    }

    #[inline]
    fn id_length(&self) -> usize {
        self.id_length as usize
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

/// A `RawTgaImage` is a structure containing the underlying raw TGA image data.
#[derive(PartialEq, Eq, Debug)]
struct RawTgaImage {
    /// The TGA header.
    header: TgaHeader,
    /// The image identification data. This is typically omitted, but it can
    /// up to 255 character long. If more data is needed, it can be placed
    /// after the image data.
    image_identification: Rc<Vec<u8>>,
    /// The colour map data, as specified by the colour map specification.
    colour_map_data: Rc<Vec<u8>>,
    /// The raw pixels themselves.
    image_data: Rc<Vec<u8>>,
    /// The extended image identification data. This field is the spillover from
    /// the image identification field if the image identification data is too
    /// long to fit into the image indentification field.
    extended_image_identification: Rc<Vec<u8>>,
}

impl RawTgaImage {
    /// Construct a new TGA image.
    fn new(
        header: TgaHeader, 
        image_identification: Rc<Vec<u8>>, 
        colour_map_data: Rc<Vec<u8>>, 
        image_data: Rc<Vec<u8>>,
        extended_image_identification: Rc<Vec<u8>>
    ) -> RawTgaImage {
        RawTgaImage {
            header: header, 
            image_identification: image_identification, 
            colour_map_data: colour_map_data,
            image_data: image_data,
            extended_image_identification: extended_image_identification,
        }
    }

    /// The function `width` returns the width of a TGA image, in pixels.
    #[inline]
    fn width(&self) -> usize {
        self.header.width()
    }

    /// Return the height of a TGA image, in pixels.
    #[inline]
    fn height(&self) -> usize {
        self.header.height()
    }

    /// Return the bit depth per pixel in a TGA Image.
    #[inline]
    fn bits_per_pixel(&self) -> usize {
        self.header.bits_per_pixel()
    }

    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
    #[inline]
    fn color_map_type(&self) -> usize {
        self.header.color_map_type as usize
    }

    #[inline]
    fn data_type_code(&self) -> usize {
        self.header.data_type_code as usize
    }

    /// The function `header` produces a copy of the TGA header.
    #[inline]
    fn header(&self) -> TgaHeader {
        self.header
    }

    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    #[inline]
    fn pixels(&self) -> PixelIter {
        PixelIter {
            inner: self.image_data.as_slice(),
            current: [0; 3],
            index: 0,
        }
    }

    #[inline]
    fn scanlines(&self) -> ScanlineIter {
        ScanlineIter {
            inner: self.image_data.as_slice(),
            height: self.height(), 
            width:  self.width(),
            row: 0,
        }
    }

    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```ignore
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    #[inline]
    fn image_data_length(&self) -> usize {
        self.image_data.len() / 3
    }

    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    #[inline]
    fn image_data_length_bytes(&self) -> usize {
        self.image_data.len()
    }

    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    #[inline]
    fn image_identification(&self) -> &[u8] {
        &self.image_identification
    }

    #[inline]
    fn image_data(&self) -> &[u8] {
        &self.image_data
    }

    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
    #[inline]
    fn extended_image_identification(&self) -> &[u8] {
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Scanline(Vec<[u8; 3]>);

impl PartialEq<[u8]> for Scanline {
    fn eq(&self, rhs: &[u8]) -> bool {
        if rhs.len() == 3 * self.0.len() {
            let pixels = &self.0;
            let rhs_pixels = rhs.chunks(3);
            return pixels.iter().zip(rhs_pixels).all(
                |(pixel, rhs_pixel)| { pixel == rhs_pixel }
            );
        }

        false
    }
}

pub struct ScanlineIter<'a> {
    inner: &'a [u8],
    height: usize,
    width: usize,
    row: usize,
}

impl<'a> Iterator for ScanlineIter<'a> {
    type Item = Scanline;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < self.height {
            let mut scanline = vec![[0; 3]; self.width];
            for col in 0..self.width {
                let pixel = [
                    self.inner[self.row * (3 * self.width) + (3*col)], 
                    self.inner[self.row * (3 * self.width) + (3*col) + 1], 
                    self.inner[self.row * (3 * self.width) + (3*col) + 2],
                ];
                scanline[col] = pixel;
            }
            self.row += 1;

            return Some(Scanline(scanline));
        }

        None
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct UncompressedRgb {
    inner: RawTgaImage,
}

impl UncompressedRgb {
    /// Parse an unmapped uncompressed TGA image from a buffer in memory. 
    /// We assume that the image to be parsed starts at the beginning of the buffer. 
    /// In order to parse correctly, the bytes of the buffer must conform to the TGA 
    /// image format.
    pub fn parse_from_buffer(buf: &[u8]) -> Result<Self, TgaError> {
        let header = TgaHeader::parse_from_buffer(buf)?;

        // Check that we were passed the correct TGA header.
        if header.data_type_code != 2 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        // Targa 24 images can also be embedded in Targa 32 images, but we do
        // not implement that (yet).
        if header.bits_per_pixel != 24 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        if buf.len() < header.id_length() + TGA_HEADER_LENGTH {
            return Err(TgaError::CorruptTgaHeader);
        }

        // Parse the image identification.
        let slice = &buf[TGA_HEADER_LENGTH..buf.len()];
        let image_identification = Rc::new(
            slice[0..header.id_length()].iter().map(|&x| x).collect::<Vec<u8>>()
        );

        // Parse the colour map data.
        let slice = &slice[header.id_length()..slice.len()];
        if slice.len() < header.colour_map_size() {
            return Err(TgaError::IncompleteColourMap(
                slice.len(), header.colour_map_size()
            ));
        }

        let colour_map_data = Rc::new(
            slice[0..header.colour_map_size()].iter().map(|&x| x).collect::<Vec<u8>>()
        );

        // Parse the image data.
        let slice = &slice[header.colour_map_size()..slice.len()];
        let image_size = header.width() * header.height() * header.bytes_per_pixel();
        if slice.len() < image_size {
            return Err(TgaError::IncompleteImageData(slice.len(), image_size));
        }

        let image_data = Rc::new(
            slice[0..image_size].iter().map(|&x| x).collect::<Vec<u8>>()
        );

        // Parse the extended image identification information from the end
        // of the image data field.
        let mut slice = &slice[image_size..slice.len()];
        
        // Check whether the end of the remaining bytes is a TGA image footer.
        if slice[(slice.len() - 26)..slice.len()] == TGA_FOOTER {
            slice = &slice[0..(slice.len() - 26)];
        }

        let extended_image_identification = Rc::new(
            slice.iter().map(|&x| x).collect::<Vec<u8>>()
        );

        let inner = RawTgaImage::new(
            header, image_identification, colour_map_data, image_data, extended_image_identification
        );

        Ok(UncompressedRgb { inner: inner })
    }

    /// The function `width` returns the width of a TGA image, in pixels.
    #[inline]
    pub fn width(&self) -> usize {
        self.inner.width()
    }

    /// Return the height of a TGA image, in pixels.
    #[inline]
    pub fn height(&self) -> usize {
        self.inner.height()
    }

    /// Return the bit depth per pixel in a TGA Image.
    #[inline]
    pub fn bits_per_pixel(&self) -> usize {
        self.inner.bits_per_pixel()
    }

    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
    #[inline]
    pub fn color_map_type(&self) -> usize {
        self.inner.color_map_type()
    }

    #[inline]
    pub fn data_type_code(&self) -> usize {
        self.inner.data_type_code()
    }

    /// The function `header` produces a copy of the TGA header.
    #[inline]
    pub fn header(&self) -> TgaHeader {
        self.inner.header()
    }

    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    #[inline]
    pub fn pixels(&self) -> PixelIter {
        self.inner.pixels()
    }

    #[inline]
    pub fn scanlines(&self) -> ScanlineIter {
        self.inner.scanlines()
    }

    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```ignore
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    #[inline]
    pub fn image_data_length(&self) -> usize {
        self.inner.image_data_length()
    }

    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    #[inline]
    pub fn image_data_length_bytes(&self) -> usize {
        self.inner.image_data_length_bytes()
    }

    #[inline]
    fn image_data(&self) -> &[u8] {
        self.inner.image_data()
    }

    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    #[inline]
    pub fn image_identification(&self) -> &[u8] {
        self.inner.image_identification()
    }

    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
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
    pub fn parse_from_buffer(buf: &[u8]) -> Result<RunLengthEncodedRgb, TgaError> {
        let header = TgaHeader::parse_from_buffer(buf)?;
        
        // Determine whether we support the image format. We presently
        // support 24 bit unmapped RGB images only. They can either be 
        // uncompressed (type code 2) or run length encoded (type code 10).
        if header.data_type_code != 10 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        // Targa 24 images can also be embedded in Targa 32 images, but we do
        // not implement that.
        if header.bits_per_pixel != 24 {
            return Err(TgaError::Not24BitRgb(header.data_type_code as usize));
        }

        if buf.len() < header.id_length() + TGA_HEADER_LENGTH {
            return Err(TgaError::CorruptTgaHeader);
        }

        // Parse the image identification.
        let slice = &buf[TGA_HEADER_LENGTH..buf.len()];
        let image_identification = Rc::new(
            slice[0..header.id_length()].iter().map(|&x| x).collect::<Vec<u8>>()
        );

        // Parse the colour map data.
        let slice = &slice[header.id_length()..slice.len()];
        if slice.len() < header.colour_map_size() {
            return Err(
                TgaError::IncompleteColourMap(slice.len(), header.colour_map_size()
            ));
        }

        let colour_map_data = Rc::new(
            slice[0..header.colour_map_size()].iter().map(|&x| x).collect::<Vec<u8>>()
        );

        // Parse the image data.
        let slice = &slice[header.colour_map_size()..slice.len()];
        let image_size = header.width() * header.height() * header.bytes_per_pixel();

        // Search the buffer for all the data packets. Here we count
        // the number of bytes of image data we have available, and compare it
        // against the size of the image that the TGA image header claims it is.
        // Simultaneously, we find where the end of the image data is.
        let mut slice_i = 0;
        let mut image_data_found = 0;
        while (slice_i < slice.len()) && (image_data_found < image_size) {
            let packet_header = slice[slice_i];
            // A run length encoded packet never represents a run of zero.
            // Hence, we add 1 to get the true run length.
            let packet_length = (packet_header & 0x7F) as usize + 1;
            if packet_header & 0x80 != 0 {
                // We have a run length packet.
                image_data_found += 3 * packet_length;
                slice_i += 4;
            } else {
                // We have a raw packet.
                image_data_found += 3 * packet_length;
                slice_i += 3 * packet_length + 1;
            }
        }

        if (image_data_found != image_size) || (slice_i > slice.len()) {
            // Either not enough image data was found, or too much was found.
            // Either way, the image data is corrupt.
            return Err(
                TgaError::IncompleteImageData(image_data_found, image_size)
            );
        }

        // The slice of the buffer that's the actual image data we
        // searched through above.
        let image_slice = &slice[0..slice_i];
        slice_i = 0;
        let mut image_data = vec![0; image_size];
        let mut i = 0;
        while i < image_size {
            let packet_header = image_slice[slice_i];
            // A run length encoded packet never represents a run of zero.
            // Hence, we add 1 to get the true run length.
            let packet_length = (packet_header & 0x7F) as usize + 1;
            if packet_header & 0x80 != 0 {
                // We have a run length packet.
                for _ in 0..packet_length {
                    image_data[i + 0] = image_slice[slice_i + 1];
                    image_data[i + 1] = image_slice[slice_i + 2];
                    image_data[i + 2] = image_slice[slice_i + 3];
                    i += 3;
                }
                // Jump to the next packet.
                slice_i += 4;
            } else {
                // We have a raw packet.
                for _ in 0..packet_length {
                    image_data[i + 0] = image_slice[slice_i + 1];
                    image_data[i + 1] = image_slice[slice_i + 2];
                    image_data[i + 2] = image_slice[slice_i + 3];

                    i += 3;
                    // Jump to the next element in the raw packet.
                    slice_i += 3;
                }
                // Jump to the next packet.
                slice_i += 1;
            }
        }

        // Parse the extended image identification information from the end
        // of the image data field.
        let mut slice = &slice[slice_i..slice.len()];
        
        // Check whether the end of the remaining bytes is a TGA image footer.
        if slice[(slice.len() - 26)..slice.len()] == TGA_FOOTER {
            slice = &slice[0..(slice.len() - 26)];
        }

        let extended_image_identification = Rc::new(
            slice.iter().map(|&x| x).collect::<Vec<u8>>()
        );

        let inner = RawTgaImage::new(
            header, image_identification, colour_map_data, Rc::new(image_data), extended_image_identification
        );

        Ok(RunLengthEncodedRgb { inner: inner })
    }

    /// The function `width` returns the width of a TGA image, in pixels.
    #[inline]
    pub fn width(&self) -> usize {
        self.inner.width()
    }

    /// Return the height of a TGA image, in pixels.
    #[inline]
    pub fn height(&self) -> usize {
        self.inner.height()
    }

    /// Return the bit depth per pixel in a TGA Image.
    #[inline]
    pub fn bits_per_pixel(&self) -> usize {
        self.inner.bits_per_pixel()
    }

    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
    #[inline]
    pub fn color_map_type(&self) -> usize {
        self.inner.color_map_type()
    }

    #[inline]
    pub fn data_type_code(&self) -> usize {
        self.inner.data_type_code()
    }

    /// The function `header` produces a copy of the TGA header.
    #[inline]
    pub fn header(&self) -> TgaHeader {
        self.inner.header()
    }

    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    #[inline]
    pub fn pixels(&self) -> PixelIter {
        self.inner.pixels()
    }

    #[inline]
    pub fn scanlines(&self) -> ScanlineIter {
        self.inner.scanlines()
    }

    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    #[inline]
    pub fn image_data_length(&self) -> usize {
        self.inner.image_data_length()
    }

    #[inline]
    fn image_data(&self) -> &[u8] {
        self.inner.image_data()
    }

    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    #[inline]
    pub fn image_data_length_bytes(&self) -> usize {
        self.inner.image_data_length_bytes()
    }

    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    #[inline]
    pub fn image_identification(&self) -> &[u8] {
        self.inner.image_identification()
    }

    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
    #[inline]
    pub fn extended_image_identification(&self) -> &[u8] {
        self.inner.extended_image_identification()
    }
}


/// A `TgaImage` is a structure containing a TGA image. This data type 
/// can represent either 24 bit uncompressed RGB images, or 24 bit
/// run-length encoded RGB images.
#[derive(PartialEq, Eq, Debug)]
pub enum TgaImage {
    Type02(UncompressedRgb),
    Type10(RunLengthEncodedRgb),
}

impl TgaImage {
    pub fn parse_from_buffer(buf: &[u8]) -> Result<TgaImage, TgaError> {
        let header = TgaHeader::parse_from_buffer(buf)?;

        // Determine whether we support the image format. We presently
        // support 24 bit unmapped RGB images only. They can either be 
        // uncompressed (type code 2) or run length encoded (type code 10).
        match header.data_type_code {
            2 => UncompressedRgb::parse_from_buffer(buf).map(|image| { 
                TgaImage::Type02(image)
            }),
            10 => RunLengthEncodedRgb::parse_from_buffer(buf).map(|image| {
                TgaImage::Type10(image)
            }),
            _ => Err(TgaError::Not24BitRgb(header.data_type_code as usize))
        }
    }

    pub fn parse_from_file<F: io::Read>(f: &mut F) -> Result<TgaImage, TgaError> {
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        Self::parse_from_buffer(&buf)
    }

    /// The function `width` returns the width of a TGA image, in pixels.
    pub fn width(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.width(),
            &TgaImage::Type10(ref image) => image.width()
        }
    }

    /// Return the height of a TGA image, in pixels.
    pub fn height(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.height(),
            &TgaImage::Type10(ref image) => image.height()
        }
    }

    /// Return the bit depth per pixel in a TGA Image.
    pub fn bits_per_pixel(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.bits_per_pixel(),
            &TgaImage::Type10(ref image) => image.bits_per_pixel()
        }
    }

    /// Compute the colour map type. The colour map type is either `0` or `1`.
    /// A `0` indicates that there is no colour map; a `1` indicates that a 
    /// colour map is included.
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

    /// The function `header` produces a copy of the TGA header.
    pub fn header(&self) -> TgaHeader {
        match self {
            &TgaImage::Type02(ref image) => image.header(),
            &TgaImage::Type10(ref image) => image.header()
        }
    }

    /// The function `pixels` generates an iterator over the pixels of the image.
    /// It sweeps through the TGA image going from left to right in each row, and 
    /// going from bottom to top. The first pixel returned is the bottom left corner;
    /// the last pixel returned is the top right corner. 
    pub fn pixels(&self) -> PixelIter {
        match self {
            &TgaImage::Type02(ref image) => image.pixels(),
            &TgaImage::Type10(ref image) => image.pixels()
        }
    }

    pub fn scanlines(&self) -> ScanlineIter {
        match self {
            &TgaImage::Type02(ref image) => image.scanlines(),
            &TgaImage::Type10(ref image) => image.scanlines()
        }
    }

    /// The function `image_data_length` returns the size of the image,
    /// in the total number of pixels. This satisfies the following invariant.
    /// ```ignore
    /// self.image_data_length() == self.width() * self.height()
    /// ```
    pub fn image_data_length(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.image_data_length(),
            &TgaImage::Type10(ref image) => image.image_data_length()
        }
    }

    /// The function `image_data_length_bytes` computes the size of the 
    /// image data, in the number of bytes. For an unmapped RGB image, this will
    /// simply be `3 * image_data_length()`, since each RGB pixel is 3 bytes long.
    pub fn image_data_length_bytes(&self) -> usize {
        match self {
            &TgaImage::Type02(ref image) => image.image_data_length_bytes(),
            &TgaImage::Type10(ref image) => image.image_data_length_bytes()
        }
    }

    #[inline]
    pub fn image_data(&self) -> &[u8] {
        match self {
            &TgaImage::Type02(ref image) => image.image_data(),
            &TgaImage::Type10(ref image) => image.image_data()
        }
    }

    /// The function `image_identification` returns a slice into the 
    /// image identification field. This is a free-form field that immediately
    /// follows the header.
    pub fn image_identification(&self) -> &[u8] {
        match self {
            &TgaImage::Type02(ref image) => image.image_identification(),
            &TgaImage::Type10(ref image) => image.image_identification()
        }
    }

    /// The function `extended_image_identification` returns a slice to the 
    /// extended image identification data. This is the data that follows after
    /// the image data that is too large for the image identification field.
    pub fn extended_image_identification(&self) -> &[u8] {
        match self {
            &TgaImage::Type02(ref image) => image.extended_image_identification(),
            &TgaImage::Type10(ref image) => image.extended_image_identification()
        }
    }

    fn raw_tga_image(&self) -> &RawTgaImage {
        match self {
            &TgaImage::Type02(ref image) => &image.inner,
            &TgaImage::Type10(ref image) => &image.inner
        }
    }
}


pub struct TgaReader {
    buffer: [Rc<Vec<u8>>; 6],
    bytes_read_from_buffer: [usize; 6],
    index: usize,
    total_bytes_read: usize,
}

impl TgaReader{
    pub fn new(image: &TgaImage) -> TgaReader {
        let header = image.header();
        let header_array = Rc::new(vec![
            header.id_length,
            header.color_map_type,
            header.data_type_code,
            header.colour_map_origin[0], header.colour_map_origin[1],
            header.colour_map_length[0], header.colour_map_length[1],
            header.colour_map_depth,
            header.x_origin[0], header.x_origin[1],
            header.y_origin[0], header.y_origin[1],
            header.width[0], header.width[1],
            header.height[0], header.height[1],
            header.bits_per_pixel,
            header.image_descriptor,
        ]);

        let inner = image.raw_tga_image();
        let footer = Rc::new(TGA_FOOTER.to_vec());

        TgaReader {
            buffer: [
                header_array, 
                inner.image_identification.clone(),
                inner.colour_map_data.clone(),
                inner.image_data.clone(),
                inner.extended_image_identification.clone(),
                footer,
            ],
            bytes_read_from_buffer: [0; 6],
            index: 0,
            total_bytes_read: 0,
        }
    }
}

impl io::Read for TgaReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut bytes_written = 0;
        while (self.index < self.buffer.len()) && (bytes_written < buf.len()) {
            let diff = self.buffer[self.index].len() - self.bytes_read_from_buffer[self.index];
            let bytes_read_from_buffer = self.bytes_read_from_buffer[self.index];
            let bytes_to_be_written = match diff <= buf.len() {
                true => diff,
                false => buf.len(),
            };

            for i in 0..bytes_to_be_written {
                buf[bytes_written + i] = self.buffer[self.index][bytes_read_from_buffer + i];
            }

            bytes_written += bytes_to_be_written;
            self.bytes_read_from_buffer[self.index] += bytes_to_be_written;
            self.total_bytes_read += bytes_to_be_written;

            if self.bytes_read_from_buffer[self.index] >= self.buffer[self.index].len() {
                self.index += 1;
            }
        }

        Ok(bytes_written)
    }
}
