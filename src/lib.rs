struct TgaHeader {
    id_length: u8,
    color_map_type: u8,
    data_type_code: u8,
    color_map_origin: [u8; 2],
    colour_map_length: [u8; 2],
    colour_map_depth: u8,
    x_origin: [u8; 2],
    y_origin: [u8; 2],
    width: [u8; 2],
    height: [u8; 2],
    bits_per_pixel: u8,
    image_descriptor: u8,
}
