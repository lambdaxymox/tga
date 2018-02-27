use std::error;
use std::fmt;
use std::io::{Read, Seek, SeekFrom};
use std::io;

const TGA_HEADER_LENGTH: usize = 18;


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TgaHeader {
    id_length: u8,
    color_map_type: u8,
    data_type_code: u8,
    colour_map_origin: [u8; 2],
    colour_map_length: [u8; 2],
    colour_map_depth: u8,
    x_origin: [u8; 2],
    y_origin: [u8; 2],
    width: [u8; 2],
    height: [u8; 2],
    bits_per_pixel: u8,
    image_descriptor: u8,
}

impl TgaHeader {
    fn parse_from_buffer(buf: &[u8]) -> Option<TgaHeader> {
        if buf.len() >= TGA_HEADER_LENGTH {
            // The buffer must be at least the length (in bytes) of a TGA header.
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

            return Some(header);
        }

        None
    }

    fn parse_from_file<F: Read>(f: &mut F) -> Option<TgaHeader> {
        let mut buf = [0; TGA_HEADER_LENGTH];
        f.read(&mut buf).unwrap();
        Self::parse_from_buffer(&buf)
    }

    fn colour_map_size(&self) -> usize {
        let colour_map_length = ((self.colour_map_length[1] as u16) << 8) 
                              | self.colour_map_length[0] as u16;

        // From the TGA specification, the color map depth will be one of
        // 16, 24, or 32 bits. That is, it is always a multiple of 8.
        let colour_map_depth_bytes = (self.colour_map_depth / 8) as u16;

        (colour_map_length * colour_map_depth_bytes) as usize
    }

    fn width(&self) -> usize {
        (((self.width[1] as u16) << 8) | (self.width[0] as u16)) as usize
    }

    fn height(&self) -> usize {
        ((((self.height[1] as u16) << 8) as u16) | (self.height[0] as u16)) as usize
    }

    fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel as usize
    }
}

#[derive(Debug)]
pub enum TgaError {
    CorruptTgaHeader,
    Not24BitUncompressedRgb,
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
            TgaError::Not24BitUncompressedRgb => {
                write!(f, "Not24BitUncompressedRgb")
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
            TgaError::Not24BitUncompressedRgb => {
                "The TGA image is not a 24 bit uncompressed RGB image."
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
            TgaError::Not24BitUncompressedRgb => None,
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

#[derive(PartialEq, Eq, Debug)]
pub struct TgaImage {
    header: TgaHeader,
    image_identification: Box<Vec<u8>>,
    colour_map_data: Box<Vec<u8>>,
    image_data: Box<Vec<u8>>,
}

impl TgaImage {
    pub fn new(
        header: TgaHeader, 
        image_identification: Box<Vec<u8>>, 
        colour_map_data: Box<Vec<u8>>, 
        image_data: Box<Vec<u8>>
    ) -> TgaImage {
        TgaImage {
            header: header, 
            image_identification: image_identification, 
            colour_map_data: colour_map_data, 
            image_data: image_data
        }
    }

    pub fn parse_from_buffer(buf: &[u8]) -> Result<TgaImage, TgaError> {
        let header = TgaHeader::parse_from_buffer(buf).unwrap();

        // Check the header.
        if header.data_type_code != 2 {
            return Err(TgaError::Not24BitUncompressedRgb);
        }

        if header.bits_per_pixel != 24 {
            return Err(TgaError::Not24BitUncompressedRgb);
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

        let image = TgaImage::new(
            header, image_identification, colour_map_data, image_data
        );

        Ok(image)
    }

    pub fn parse_from_file<F: Read + Seek>(f: &mut F) -> Result<TgaImage, TgaError> {
        let header = TgaHeader::parse_from_file(f).unwrap();
        let offset = match f.seek(SeekFrom::Start(TGA_HEADER_LENGTH as u64)) {
            Ok(val) => val as usize,
            Err(_) => return Err(TgaError::CorruptTgaHeader)
        };

        if offset != TGA_HEADER_LENGTH {
            return Err(
                TgaError::IncompleteTgaHeader(offset, TGA_HEADER_LENGTH)
            );
        }

        // Check the header.
        if header.data_type_code != 2 {
            return Err(TgaError::Not24BitUncompressedRgb);
        }

        if header.bits_per_pixel != 24 {
            return Err(TgaError::Not24BitUncompressedRgb);
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

        let image = TgaImage::new(
            header, image_identification, colour_map_data, image_data
        );

        Ok(image)        
    }

    pub fn width(&self) -> usize {
        self.header.width()
    }

    pub fn height(&self) -> usize {
        self.header.height()
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.header.bits_per_pixel()
    }

    pub fn color_map_type(&self) -> usize {
        self.header.color_map_type as usize
    }

    pub fn data_type_code(&self) -> usize {
        self.header.data_type_code as usize
    }

    pub fn header(&self) -> TgaHeader {
        self.header
    }

    pub fn pixels(&self) -> PixelIter {
        PixelIter {
            inner: self.image_data.as_slice(),
            current: [0; 3],
            index: 0,
        }
    }

    pub fn image_data_length(&self) -> usize {
        self.image_data.len() / 3
    }

    pub fn image_data_length_bytes(&self) -> usize {
        self.image_data.len()
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
