use std::io::Read;
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
    fn parse_tga_header(buf: &[u8]) -> Option<TgaHeader> {
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
}
